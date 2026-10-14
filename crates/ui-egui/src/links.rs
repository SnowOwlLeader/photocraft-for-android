//! Community and project links: the Help menu, the About dialog, the start screen and the title
//! bar all open these. URLs follow the crafting-app pattern (`getartcraft.com/apps/<app>`,
//! `github.com/storytold/<app>`).

use serde_json::{Value, json};

use crate::PhotocraftApp;

pub const COMMUNITY_LABEL: &str = "加入我们的 QQ 社群";
pub const COMMUNITY_GROUP: &str = "1129505510";
pub const COMMUNITY_URL: &str = "mqqapi://card/show_pslcard?src_type=internal&version=1&uin=1129505510&card_type=group&source=qrcode";
pub const ARTCRAFT_WEBSITE: &str = "https://getartcraft.com";
pub const APP_PAGE: &str = "https://getartcraft.com/apps/photocraft";
pub const GITHUB: &str = "https://github.com/storytold/photocraft";
pub const ISSUES: &str = "https://github.com/storytold/photocraft/issues";

/// Help-menu link commands: (id, url). Labels live in `menus::UI_COMMANDS`.
pub const COMMANDS: &[(&str, &str)] = &[
    ("help.discord", COMMUNITY_URL),
    ("help.website", APP_PAGE),
    ("help.artcraftWebsite", ARTCRAFT_WEBSITE),
    ("help.github", GITHUB),
    ("help.reportIssue", ISSUES),
];

pub fn url_for(id: &str) -> Option<&'static str> {
    COMMANDS.iter().find(|c| c.0 == id).map(|c| c.1)
}

/// Open `url` in the system browser (a new tab on the web) and note it in the status bar. Prefers
/// the platform `open_url` service (reliable on Windows/macOS/Linux); falls back to `ctx.open_url`.
pub fn open(app: &mut PhotocraftApp, ctx: &egui::Context, url: &str) -> Value {
    let opened = app.services.open_url.as_ref().map(|f| f(url).is_ok()).unwrap_or(false);
    if !opened {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
    let template = "Opened {url}";
    app.ui.status = crate::i18n::fmt(tl!(template), &[("url", url)]);
    json!({"url": url})
}

/// The community entry shared by the start screen and About dialog.
pub fn community_button(app: &mut PhotocraftApp, ui: &mut egui::Ui, min_width: f32) -> egui::Response {
    let r = crate::widgets::primary_button(ui, COMMUNITY_LABEL, min_width).on_hover_text(COMMUNITY_URL);
    ui.label(format!("QQ群号：{COMMUNITY_GROUP}（可在 QQ 中搜索加入）"));
    if r.clicked() {
        open(app, ui.ctx(), COMMUNITY_URL);
    }
    r
}

/// "PhotoCraft website · GitHub · ArtCraft" as links, centred. Clicks route through [`open`] (the
/// platform browser service) rather than `ui.hyperlink_to`, which uses the unreliable `ctx.open_url`.
pub fn link_row(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let t = crate::theme::Tokens::get(ui.ctx());
    let links = [(tl!("PhotoCraft website"), APP_PAGE), (tl!("GitHub"), GITHUB), (tl!("ArtCraft"), ARTCRAFT_WEBSITE)];
    let font = egui::FontId::proportional(12.5);
    let sep = "  ·  ";
    let width: f32 = links.iter().map(|(l, _)| ui.painter().layout_no_wrap((*l).into(), font.clone(), t.text).size().x).sum::<f32>()
        + 2.0 * ui.painter().layout_no_wrap(sep.into(), font.clone(), t.text).size().x;
    let mut clicked: Option<&str> = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
        for (i, (label, url)) in links.iter().enumerate() {
            if i > 0 {
                ui.label(egui::RichText::new(sep).font(font.clone()).color(t.text_faint));
            }
            if ui.link(egui::RichText::new(*label).font(font.clone()).color(t.accent)).on_hover_text(*url).clicked() {
                clicked = Some(url);
            }
        }
    });
    if let Some(url) = clicked {
        open(app, ui.ctx(), url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_open_through_the_platform_service() {
        // Help, community and start-page links must reach the platform service.
        // They route through the `open_url` service rather than the unreliable `ctx.open_url`.
        use std::sync::{Arc, Mutex};
        let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let rec = opened.clone();
        let services = crate::Services {
            open_url: Some(Box::new(move |u: &str| {
                rec.lock().unwrap().push(u.to_string());
                Ok(())
            })),
            ..Default::default()
        };
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), services);
        let ctx = egui::Context::default();
        // Every Help-menu link id reaches the service with its URL.
        for (id, url) in COMMANDS {
            crate::menus::invoke(&mut app, &ctx, id, json!({})).unwrap();
            assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(*url), "{id}");
        }
        // The direct helper used by community and start-page links also uses it.
        open(&mut app, &ctx, COMMUNITY_URL);
        assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(COMMUNITY_URL));
    }

    #[test]
    fn help_menu_lists_links_then_separator_then_system_info_and_about() {
        let app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let help: Vec<String> = crate::menus::menu_items(&app).into_iter().filter(|i| i.path == ["Help"]).map(|i| i.id).collect();
        assert_eq!(help, ["help.discord", "help.website", "help.artcraftWebsite", "help.github", "help.reportIssue", "---", "help.systemInfo", "help.about"]);
        for (id, _) in COMMANDS {
            assert!(crate::menus::is_live(id) && crate::menus::is_enabled(&app, id), "{id}");
        }
    }

    #[test]
    fn link_commands_open_their_urls() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let ctx = egui::Context::default();
        for (id, url) in [
            ("help.discord", COMMUNITY_URL),
            ("help.website", "https://getartcraft.com/apps/photocraft"),
            ("help.github", "https://github.com/storytold/photocraft"),
        ] {
            let r = crate::menus::invoke(&mut app, &ctx, id, serde_json::json!({})).unwrap();
            assert_eq!(r["url"], url);
            assert_eq!(app.ui.status, format!("Opened {url}"));
        }
    }

    #[test]
    fn community_button_opens_the_current_qq_group() {
        use egui_kittest::kittest::Queryable;
        use std::sync::{Arc, Mutex};
        let urls = Arc::new(Mutex::new(Vec::new()));
        let recorded = urls.clone();
        let mut app = PhotocraftApp::new(
            photocraft_engine::Session::new(),
            crate::Services {
                open_url: Some(Box::new(move |url| {
                    recorded.lock().unwrap().push(url.to_owned());
                    Ok(())
                })),
                ..Default::default()
            },
        );
        let mut h = egui_kittest::Harness::new_ui(|ui| {
            if ui.ctx().fonts(|f| f.families().contains(&egui::FontFamily::Name("medium".into()))) {
                community_button(&mut app, ui, 160.0);
            }
        });
        PhotocraftApp::setup_context(&h.ctx, Default::default());
        h.run_steps(2);
        h.get_by_label(COMMUNITY_LABEL).click();
        h.run();
        assert_eq!(*urls.lock().unwrap(), [COMMUNITY_URL]);
        assert!(COMMUNITY_URL.contains("uin=1129505510&card_type=group"));
        h.get_by_label_contains("1129505510");
    }
}
