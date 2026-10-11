use android_activity::AndroidApp;
use jni::{
    JavaVM,
    objects::{GlobalRef, JObject, JString, JValue},
};
use photocraft_ui_egui::{FileDialogAnswer, FileDialogReply, FileDialogRequest, PhotocraftApp, Services};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::Duration,
};

struct Bridge {
    vm: JavaVM,
    activity: GlobalRef,
}

impl Bridge {
    // Android gives us a live VM and a global Activity reference. The owned GlobalRef below
    // keeps the Activity alive; no native pointers are exposed outside this small adapter.
    #[allow(unsafe_code)]
    fn new(app: &AndroidApp) -> Result<Self, String> {
        // SAFETY: android_main is called with the VM owned by Android for this process.
        let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) }.map_err(|e| e.to_string())?;
        let activity = {
            let env = vm.attach_current_thread().map_err(|e| e.to_string())?;
            // SAFETY: AndroidApp retains this global reference for its lifetime. We immediately
            // create our own global reference and never delete AndroidApp's reference.
            let activity = unsafe { JObject::from_raw(app.activity_as_ptr().cast()) };
            env.new_global_ref(activity).map_err(|e| e.to_string())?
        };
        Ok(Self { vm, activity })
    }

    fn call(&self, method: &str, arg: Option<&str>, returns_text: bool) -> Result<String, String> {
        let mut env = self.vm.attach_current_thread().map_err(|e| e.to_string())?;
        let result = env.with_local_frame(8, |env| -> Result<String, jni::errors::Error> {
            let text = arg.map(|s| env.new_string(s)).transpose()?;
            let args = text.as_ref().map(|s| vec![JValue::Object(s.as_ref())]).unwrap_or_default();
            let signature = match (arg.is_some(), returns_text) {
                (true, true) => "(Ljava/lang/String;)Ljava/lang/String;",
                (true, false) => "(Ljava/lang/String;)V",
                (false, true) => "()Ljava/lang/String;",
                (false, false) => "()V",
            };
            let result = env.call_method(self.activity.as_obj(), method, signature, &args)?;
            if returns_text {
                let object = JString::from(result.l()?);
                let value: String = env.get_string(&object)?.into();
                Ok(value)
            } else {
                Ok(String::new())
            }
        });
        if result.is_err() {
            let _ = env.exception_clear();
        }
        result.map_err(|e| format!("Android {method}: {e}"))
    }
    fn write(&self, path: &str, bytes: &[u8]) -> Result<(), String> {
        photocraft_format::atomic_write(Path::new(path), bytes).map_err(|e| e.to_string())?;
        let error = self.call("commitFile", Some(path), true)?;
        if error.is_empty() { Ok(()) } else { Err(error) }
    }
}

type Pending = Rc<RefCell<Option<FileDialogReply>>>;

fn guarded<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .unwrap_or_else(|_| Err("PhotoCraft could not complete this operation; the document remains open.".into()))
}

fn export(doc: &photocraft_doc::Document, path: &str, settings: &photocraft_ui_egui::ExportSettings) -> Result<(Vec<u8>, Vec<String>), String> {
    guarded(|| {
        let mut options = photocraft_io::ExportOptions::default();
        if let Some(q) = settings.jpeg_quality {
            options.encode.jpeg_quality = q;
        }
        options.encode.webp_lossless = settings.webp_lossless;
        if let Some(q) = settings.webp_quality {
            options.encode.webp_quality = q;
        }
        options.tiff_layers = settings.tiff_layers;
        options.tga_bits = settings.tga_bits;
        options.xmp = if settings.xmp_all { photocraft_io::XmpEmbed::All } else { photocraft_io::XmpEmbed::None };
        photocraft_io::export(doc, path, &options).map(|r| (r.bytes, r.warnings)).map_err(|e| e.to_string())
    })
}

fn services(bridge: Arc<Bridge>, pending: Pending, directory: &Path) -> Services {
    let picker = bridge.clone();
    let writer = bridge.clone();
    let saver = bridge.clone();
    let preferences = directory.join("preferences.json");
    let load_preferences = preferences.clone();
    let recovery_dir = directory.join("Recovery");
    let recovery = Rc::new(RefCell::new(photocraft_format::RecoveryStore::new(recovery_dir.clone())));
    let (autosave, results, discard, adopt) = (recovery.clone(), recovery.clone(), recovery.clone(), recovery.clone());
    Services {
        import: Some(Box::new(|name, bytes, depth| {
            guarded(|| photocraft_io::import_with_svg_group_depth(name, bytes, depth).map(|r| (r.document, r.warnings)).map_err(|e| e.to_string()))
        })),
        export: Some(Box::new(export)),
        write: Some(Box::new(move |path, bytes| writer.write(path, bytes))),
        save_file: Some(Arc::new(move |doc, path, settings, ctl| {
            ctl.progress(0.0, "Encoding");
            let (bytes, warnings) = export(doc, path, settings)?;
            ctl.commit()?;
            ctl.progress(0.0, "Writing");
            saver.write(path, &bytes)?;
            Ok(warnings)
        })),
        file_dialog: Some(Box::new(move |request, _, reply| {
            let request = match request {
                FileDialogRequest::Open { multiple, .. } => json!({"multiple": multiple}),
                FileDialogRequest::Save { suggested } => {
                    json!({"save": true, "name": Path::new(&suggested).file_name().map(|p| p.to_string_lossy()).unwrap_or_default()})
                }
            };
            if let Err(e) = picker.call("requestFile", Some(&request.to_string()), false) {
                log::error!("{e}");
                reply.send(None);
            } else {
                *pending.borrow_mut() = Some(reply);
            }
        })),
        load_prefs: Some(Box::new(move || std::fs::read_to_string(&load_preferences).ok())),
        save_prefs: Some(Box::new(move |text| photocraft_format::atomic_write(&preferences, text.as_bytes()).map_err(|e| e.to_string()))),
        open_url: Some(Box::new(move |url| bridge.call("openLink", Some(url), false).map(|_| ()))),
        autosave: Some(Box::new(move |doc, revision, path| autosave.borrow_mut().autosave_checked(doc, revision, path.map(str::to_string)))),
        autosave_results: Some(Box::new(move || results.borrow_mut().take_completed())),
        discard_autosave: Some(Box::new(move |id| {
            let _ = discard.borrow_mut().discard(id);
        })),
        adopt_autosave: Some(Box::new(move |id, key| {
            adopt.borrow_mut().adopt(id, key);
        })),
        recover: Some(Box::new(move || {
            photocraft_format::list_recovery(&recovery_dir)
                .into_iter()
                .map(|entry| photocraft_ui_egui::Recoverable {
                    key: entry.info.key.clone(),
                    name: entry.info.document_name.clone(),
                    path: entry.info.original_path.clone(),
                    load: Box::new(move || photocraft_format::recover(&entry).map_err(|e| e.to_string())),
                })
                .collect()
        })),
        ..Default::default()
    }
}

struct AndroidShell {
    app: PhotocraftApp,
    bridge: Arc<Bridge>,
    pending: Pending,
    error: Option<String>,
    text_session: Option<String>,
    font_directory: PathBuf,
    font_pending: bool,
    font_theme: Option<photocraft_ui_egui::theme::ThemeKind>,
}

fn android_fonts(script: photocraft_text::cjk::CjkScript) -> Vec<photocraft_text::cjk::FontFile> {
    use photocraft_text::cjk::{CjkScript, FontFile};
    let family = match script {
        CjkScript::Japanese => "Noto Sans CJK JP",
        CjkScript::Korean => "Noto Sans CJK KR",
        CjkScript::SimplifiedChinese => "Noto Sans CJK SC",
        CjkScript::TraditionalChinese => "Noto Sans CJK TC",
    };
    vec![FontFile { path: PathBuf::from("/system/fonts/NotoSansCJK-Regular.ttc"), family }]
}

impl AndroidShell {
    fn poll(&mut self) -> Result<(), String> {
        let text = self.bridge.call("pollReply", None, true)?;
        if text.is_empty() {
            return Ok(());
        }
        let event: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if let Some(text) = event.get("text").and_then(Value::as_str) {
            let layer = event.get("layer").and_then(Value::as_u64).ok_or("Missing text layer")?;
            let session = event.get("session").and_then(Value::as_str).ok_or("Missing text session")?;
            if self.app.ui.text_edit.as_ref().is_some_and(|ed| ed.layer == layer && ed.session == session) {
                self.app.run("type.edit", json!({"layer": layer, "text": text, "coalesce": session}))?;
                if let Some(ed) = self.app.ui.text_edit.as_mut() {
                    ed.anchor = 0;
                    ed.caret = text.chars().count();
                }
            }
            return Ok(());
        }
        if event.get("textCancelled").is_some() {
            return Ok(());
        }
        if self.font_pending {
            self.font_pending = false;
            if let Some(error) = event.get("error").and_then(Value::as_str) {
                return Err(error.into());
            }
            if let Some(files) = event.get("files").and_then(Value::as_array) {
                for file in files {
                    let path = file.get("path").and_then(Value::as_str).ok_or("Missing font file")?;
                    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
                    let _ = std::fs::remove_file(path);
                    if bytes.len() > 32 * 1024 * 1024 {
                        return Err("字体文件不能超过 32 MiB".into());
                    }
                    let names = photocraft_text::shared().lock().unwrap_or_else(|e| e.into_inner()).fonts.register_font_data(bytes.clone());
                    let Some(name) = names.first() else {
                        return Err("无法识别字体，请选择 TTF、OTF 或 TTC 字体".into());
                    };
                    std::fs::create_dir_all(&self.font_directory).map_err(|e| e.to_string())?;
                    let filename = format!("font-{}.font", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
                    photocraft_format::atomic_write(&self.font_directory.join(filename), &bytes).map_err(|e| e.to_string())?;
                    self.app.ui.tool_options.type_font = name.clone();
                    if let Some(ed) = self.app.ui.text_edit.clone() {
                        self.app.run("type.setStyle", json!({"layer": ed.layer, "font": name, "coalesce": ed.session}))?;
                    }
                    self.app.ui.status = format!("已导入字体：{}", names.join("、"));
                }
            }
            return Ok(());
        }
        let reply = self.pending.borrow_mut().take();
        if let Some(error) = event.get("error").and_then(Value::as_str) {
            return Err(error.into());
        }
        let Some(reply) = reply else { return Ok(()) };
        let answer = if let Some(path) = event.get("save").and_then(Value::as_str) {
            Some(FileDialogAnswer::SaveTo(path.into()))
        } else if let Some(files) = event.get("files").and_then(Value::as_array) {
            let mut contents = Vec::new();
            for file in files {
                let path = file.get("path").and_then(Value::as_str).ok_or("Missing imported file")?;
                let name = file.get("name").and_then(Value::as_str).unwrap_or("image");
                let bytes = photocraft_format::read_file(Path::new(path)).map_err(|e| e.to_string());
                let _ = std::fs::remove_file(path);
                contents.push((name.to_string(), bytes?));
            }
            Some(FileDialogAnswer::ContentsMany(contents))
        } else {
            None
        };
        reply.send(answer);
        Ok(())
    }

    fn command(&mut self, ctx: &egui::Context, id: &str) {
        if let Err(error) = photocraft_ui_egui::menus::invoke(&mut self.app, ctx, id, json!({})) {
            self.error = Some(error);
        }
    }
}

impl eframe::App for AndroidShell {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Err(e) = self.poll() {
            self.error = Some(e);
        }
        self.app.logic(ctx, frame);
        if self.font_theme != Some(self.app.ui.theme) {
            let mut sources = photocraft_ui_egui::cjk_fonts::Sources::system();
            sources.files = android_fonts;
            photocraft_ui_egui::cjk_fonts::install_with(ctx, sources);
            self.font_theme = Some(self.app.ui.theme);
        }
        // The native file picker finishes off-thread, outside winit's input queue.
        if self.pending.borrow().is_some() || self.font_pending || self.app.ui.text_edit.is_some() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, input: &mut egui::RawInput) {
        self.app.raw_input_hook(ctx, input);
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        egui::Panel::top("android_actions").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (label, command) in [
                    ("New", "file.new"),
                    ("Open", "file.open"),
                    ("Save", "file.save"),
                    ("Save As", "file.saveAs"),
                    ("Undo", "edit.undo"),
                    ("Redo", "edit.redo"),
                    ("Fit", "view.fitOnScreen"),
                ] {
                    if ui.add_sized([50.0, 32.0], egui::Button::new(photocraft_ui_egui::i18n::t(label))).clicked() {
                        self.command(&ctx, command);
                    }
                }
                ui.toggle_value(&mut self.app.ui.panels.dock, "图层");
                ui.toggle_value(&mut self.app.ui.panels.options_bar, "选项");
            });
        });
        if let Some(error) = self.error.clone() {
            egui::Window::new("PhotoCraft").collapsible(false).show(&ctx, |ui| {
                ui.label(error);
                if ui.button("OK").clicked() {
                    self.error = None;
                }
            });
        }
        self.app.ui(ui, frame);
        if self.app.ui.tool.is_type() {
            egui::Window::new("文字工具")
                .default_pos([40.0, 90.0])
                .default_width((ctx.content_rect().width() - 64.0).clamp(240.0, 540.0))
                .resizable(false)
                .collapsible(true)
                .show(&ctx, |ui| {
                    photocraft_ui_egui::type_tool::mobile_options_bar(&mut self.app, ui);
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("输入 / 编辑文字").clicked() {
                            if self.app.ui.text_edit.is_none()
                                && photocraft_ui_egui::type_tool::edit_active(&mut self.app).is_err()
                                && let Some(st) = self.app.session.active()
                            {
                                let p = [f64::from(st.doc.size.width) / 4.0, f64::from(st.doc.size.height) / 2.0];
                                photocraft_ui_egui::type_tool::pointer_up(&mut self.app, p, p);
                            }
                            self.text_session = None;
                        }
                        if ui.add_enabled(!self.font_pending && self.pending.borrow().is_none(), egui::Button::new("上传字体")).clicked() {
                            match self.bridge.call("requestFile", Some("{\"multiple\":true}"), false) {
                                Ok(_) => self.font_pending = true,
                                Err(e) => self.error = Some(e),
                            }
                        }
                        ui.label("TTF / OTF / TTC");
                    });
                });
        }
        if let Some(ed) = self.app.ui.text_edit.clone() {
            if self.text_session.as_deref() != Some(&ed.session) {
                self.text_session = Some(ed.session.clone());
                if let Some(photocraft_doc::LayerContent::Text(t)) =
                    self.app.session.active().and_then(|st| st.doc.layer(photocraft_doc::LayerId(ed.layer))).map(|l| &l.content)
                {
                    let text = if ed.created && t.text == photocraft_ui_egui::type_tool::PLACEHOLDER { "" } else { &t.text };
                    if let Err(e) = self.bridge.call("editText", Some(&json!({"layer": ed.layer, "session": ed.session, "text": text}).to_string()), false) {
                        self.error = Some(e);
                    }
                }
            }
        } else {
            self.text_session = None;
        }
    }
}

fn start(android: AndroidApp) -> Result<(), String> {
    let bridge = Arc::new(Bridge::new(&android)?);
    let directory = PathBuf::from(bridge.call("dataDirectory", None, true)?);
    if bridge.call("selfTestRequested", None, true)? == "yes" {
        let result = super::smoke::run();
        let report = json!({"passed": result.is_ok(), "error": result.err()}).to_string();
        log::info!("PHOTOCRAFT_SELF_TEST {report}");
        std::fs::write(directory.join("self-test.json"), report).map_err(|e| e.to_string())?;
    }
    let mut options = eframe::NativeOptions { android_app: Some(android), ..Default::default() };
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(create) = &mut options.wgpu_options.wgpu_setup {
        // Mobile Vulkan implementations can expose smaller limits than desktop defaults
        // (for example a 16 KiB uniform buffer). Request only what this adapter supports.
        create.device_descriptor = Arc::new(|adapter| eframe::wgpu::DeviceDescriptor {
            label: Some("PhotoCraft Android GPU"),
            required_limits: adapter.limits(),
            ..Default::default()
        });
    }
    eframe::run_native(
        "PhotoCraft",
        options,
        Box::new(move |cc| {
            let mut session = photocraft_engine::Session::new();
            let _ = session.attach_preset_store(photocraft_engine::preset_store::open_dir(directory.join("Presets")));
            let pending = Rc::new(RefCell::new(None));
            PhotocraftApp::setup_context(&cc.egui_ctx, photocraft_ui_egui::theme::ThemeKind::Pro);
            let mut app = PhotocraftApp::new(session, services(bridge.clone(), pending.clone(), &directory));
            app.background_jobs = true;
            app.ui.tool_options.move_auto_select = true;
            app.ui.tool_options.move_show_transform = true;
            let font_directory = directory.join("Fonts");
            if let Ok(files) = std::fs::read_dir(&font_directory) {
                for file in files.flatten() {
                    if file.metadata().is_ok_and(|m| m.len() <= 32 * 1024 * 1024)
                        && let Ok(bytes) = std::fs::read(file.path())
                    {
                        photocraft_text::shared().lock().unwrap_or_else(|e| e.into_inner()).fonts.register_font_data(bytes);
                    }
                }
            }
            app.ui.panels.dock = false;
            app.ui.panels.rail = false;
            app.ui.panels.options_bar = false;
            if let Some(render) = cc.wgpu_render_state.clone() {
                app.set_wgpu(render);
            }
            Ok(Box::new(AndroidShell { app, bridge, pending, error: None, font_theme: None, text_session: None, font_directory, font_pending: false }))
        }),
    )
    .map_err(|e| e.to_string())
}

// SAFETY: android-activity resolves this single, uniquely named Rust-ABI entry point.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_tag("PhotoCraft").with_max_level(log::LevelFilter::Info));
    if let Err(error) = guarded(|| start(app)) {
        log::error!("PhotoCraft startup failed: {error}");
    }
}
