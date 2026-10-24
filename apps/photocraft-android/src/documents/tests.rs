use super::*;
use photocraft_engine::Session;
use photocraft_ui_egui::{FileDialogReply, FileDialogRequest, PhotocraftApp, Services, menus};
use serde_json::json;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

struct Files(PathBuf);

impl Files {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "photocraft-save-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn psd(&self, folder: &str) -> String {
        let dir = self.0.join(folder);
        std::fs::create_dir_all(&dir).unwrap();
        let path = std::fs::canonicalize(dir).unwrap().join("原作品.PSD");
        let mut session = Session::new();
        session.execute("file.new", json!({"width": 4, "height": 4})).unwrap();
        let bytes = photocraft_io::export(&session.active().unwrap().doc, "source.psd", &Default::default()).unwrap().bytes;
        std::fs::write(&path, bytes).unwrap();
        path.to_string_lossy().into_owned()
    }
}

impl Drop for Files {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

type Dialogs = Rc<RefCell<Vec<(FileDialogRequest, FileDialogReply)>>>;
type Writes = Rc<RefCell<Vec<String>>>;

fn app(fail: bool) -> (PhotocraftApp, Dialogs, Writes) {
    let dialogs: Dialogs = Default::default();
    let writes: Writes = Default::default();
    let (asked, written) = (dialogs.clone(), writes.clone());
    let app = PhotocraftApp::new(
        Session::new(),
        Services {
            import: Some(Box::new(|name, bytes, options| {
                photocraft_io::import_with_options(name, bytes, options).map(|r| (r.document, r.warnings)).map_err(|e| e.to_string())
            })),
            export: Some(Box::new(|doc, path, _| {
                photocraft_io::export(doc, path, &Default::default()).map(|r| (r.bytes, r.warnings)).map_err(|e| e.to_string())
            })),
            write: Some(Box::new(move |path, bytes| {
                if fail {
                    return Err("Provider denied write".into());
                }
                photocraft_format::atomic_write(std::path::Path::new(path), bytes).map_err(|e| e.to_string())?;
                written.borrow_mut().push(path.into());
                Ok(())
            })),
            file_dialog: Some(Box::new(move |request, _, reply| asked.borrow_mut().push((request, reply)))),
            ..Default::default()
        },
    );
    (app, dialogs, writes)
}

fn open(app: &mut PhotocraftApp, dialogs: &Dialogs, ctx: &egui::Context, paths: &[String]) {
    menus::invoke(app, ctx, "file.open", json!({})).unwrap();
    app.poll_file_dialog(ctx, None);
    let files: Vec<_> = paths.iter().map(|path| json!({"path": path, "name": "原作品.PSD"})).collect();
    let (_, reply) = dialogs.borrow_mut().pop().unwrap();
    reply.send(Some(picked_files(&files).unwrap()));
    app.poll_file_dialog(ctx, None);
    assert!(!app.ui.status_error, "{}", app.ui.status);
}

#[test]
fn opened_psd_save_overwrites_and_save_as_retargets_subsequent_saves() {
    let files = Files::new();
    let source = files.psd("imports/source");
    let destination = files.psd("exports/target");
    let (mut app, dialogs, writes) = app(false);
    let ctx = egui::Context::default();
    open(&mut app, &dialogs, &ctx, std::slice::from_ref(&source));
    assert_eq!(app.session.active().unwrap().path.as_deref(), Some(source.as_str()));
    assert_eq!(app.session.active().unwrap().doc.name, "原作品.PSD");
    let original = std::fs::read(&source).unwrap();
    for layers in [2, 3] {
        app.run("layer.new.layer", json!({})).unwrap();
        menus::invoke(&mut app, &ctx, "file.save", json!({})).unwrap();
        assert!(!app.file_dialog_open(), "Save must not invoke CREATE_DOCUMENT");
        assert_eq!(app.session.active().unwrap().path.as_deref(), Some(source.as_str()));
        assert!(!app.session.active().unwrap().is_dirty());
        let bytes = std::fs::read(&source).unwrap();
        assert_ne!(bytes, original);
        assert_eq!(photocraft_io::import("source.psd", &bytes).unwrap().document.layers.len(), layers);
    }
    let saved_original = std::fs::read(&source).unwrap();
    menus::invoke(&mut app, &ctx, "file.saveAs", json!({})).unwrap();
    app.poll_file_dialog(&ctx, None);
    let (_, reply) = dialogs.borrow_mut().pop().unwrap();
    reply.send(None);
    app.poll_file_dialog(&ctx, None);
    assert_eq!(app.session.active().unwrap().path.as_deref(), Some(source.as_str()), "cancel keeps original destination");
    menus::invoke(&mut app, &ctx, "file.saveAs", json!({})).unwrap();
    app.poll_file_dialog(&ctx, None);
    let (_, reply) = dialogs.borrow_mut().pop().unwrap();
    reply.send(Some(FileDialogAnswer::SaveTo(destination.clone())));
    app.poll_file_dialog(&ctx, None);
    // The save dialog lowercases the extension; the provider bridge uses the same spelling.
    let target = std::path::Path::new(&destination).with_extension("psd").to_string_lossy().into_owned();
    assert_eq!(app.session.active().unwrap().path.as_deref(), Some(target.as_str()));
    app.run("layer.new.layer", json!({})).unwrap();
    menus::invoke(&mut app, &ctx, "file.save", json!({})).unwrap();
    assert!(!app.file_dialog_open());
    assert_eq!(std::fs::read(&source).unwrap(), saved_original, "Save As and later Save leave the old PSD intact");
    assert_eq!(photocraft_io::import("target.psd", &std::fs::read(&target).unwrap()).unwrap().document.layers.len(), 4);
    assert_eq!(*writes.borrow(), [source.clone(), source, target.clone(), target]);
}

#[test]
fn failed_provider_save_keeps_edits_dirty_and_does_not_create_a_copy() {
    let files = Files::new();
    let source = files.psd("source");
    let original = std::fs::read(&source).unwrap();
    let (mut app, dialogs, writes) = app(true);
    let ctx = egui::Context::default();
    open(&mut app, &dialogs, &ctx, std::slice::from_ref(&source));
    app.run("layer.new.layer", json!({})).unwrap();
    assert!(menus::invoke(&mut app, &ctx, "file.save", json!({})).unwrap_err().contains("denied"));
    assert!(app.session.active().unwrap().is_dirty());
    assert_eq!(app.session.active().unwrap().path.as_deref(), Some(source.as_str()));
    assert_eq!(std::fs::read(&source).unwrap(), original);
    assert!(writes.borrow().is_empty() && !app.file_dialog_open());
}

#[test]
fn identically_named_picked_psds_keep_separate_destinations() {
    let files = Files::new();
    let paths = vec![files.psd("first"), files.psd("second")];
    let (mut app, dialogs, writes) = app(false);
    let ctx = egui::Context::default();
    open(&mut app, &dialogs, &ctx, &paths);
    for (index, path) in paths.iter().enumerate() {
        app.session.set_active(index);
        app.run("layer.new.layer", json!({})).unwrap();
        menus::invoke(&mut app, &ctx, "file.save", json!({})).unwrap();
        assert_eq!(app.session.active().unwrap().path.as_deref(), Some(path.as_str()));
    }
    assert_eq!(*writes.borrow(), paths);
    assert!(!app.file_dialog_open());
}

#[test]
fn background_open_preserves_original_psd_destination() {
    let files = Files::new();
    let source = files.psd("background");
    let (mut app, dialogs, writes) = app(false);
    app.background_jobs = true;
    let ctx = egui::Context::default();
    open(&mut app, &dialogs, &ctx, std::slice::from_ref(&source));
    let started = std::time::Instant::now();
    while !app.jobs.opens.is_empty() && started.elapsed() < std::time::Duration::from_secs(30) {
        photocraft_ui_egui::jobs_ui::tick(&mut app, &ctx);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(app.jobs.opens.is_empty());
    assert_eq!(app.session.active().unwrap().path.as_deref(), Some(source.as_str()));
    app.run("layer.new.layer", json!({})).unwrap();
    menus::invoke(&mut app, &ctx, "file.save", json!({})).unwrap();
    assert!(!app.file_dialog_open());
    assert_eq!(*writes.borrow(), [source]);
}

#[test]
fn invalid_provider_reply_is_an_error() {
    for file in [json!({"name":"source.psd"}), json!({"path":""}), json!({"path":32})] {
        assert!(picked_files(&[file]).is_err());
    }
}
