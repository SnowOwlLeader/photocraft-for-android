//! A touch-friendly entry to the existing pixel-grid pencil engine.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]
use photocraft_ui_egui::{PhotocraftApp, state::Tool};
#[cfg(test)]
use serde_json::json;

pub fn show(app: &mut PhotocraftApp, ctx: &egui::Context) -> Result<(), String> {
    if app.ui.tool != Tool::Pencil {
        return Ok(());
    }
    let mut size = app.session.tools.brush.size.round().clamp(1.0, 64.0) as u32;
    let before = size;
    egui::Window::new("像素铅笔").default_pos([48.0, 108.0]).resizable(false).collapsible(true).show(ctx, |ui| {
        ui.label("整像素绘制 · 无抗锯齿");
        ui.horizontal(|ui| {
            if ui.add_enabled(size > 1, egui::Button::new("－")).clicked() {
                size = size.saturating_sub(1);
            }
            ui.add(egui::DragValue::new(&mut size).range(1..=64).suffix(" px"));
            if ui.add_enabled(size < 64, egui::Button::new("＋")).clicked() {
                size = size.saturating_add(1).min(64);
            }
        });
        ui.horizontal(|ui| {
            for n in [1, 2, 4, 8, 16] {
                if ui.selectable_label(size == n, format!("{n}px")).clicked() {
                    size = n;
                }
            }
        });
        ui.checkbox(&mut app.ui.tool_options.pencil_auto_erase, "自动擦除");
        ui.small("双指放大画布后更方便逐像素绘制");
    });
    if size != before {
        app.run("tools.setBrush", serde_json::json!({"size":size}))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pixel_entry_resets_dynamics_and_draws_an_opaque_one_pixel_staircase() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        app.run("file.new", json!({"width":16,"height":16,"background":"transparent"})).unwrap();
        app.run("tools.setBrush", json!({"size":20,"opacity":0.2,"pressureSize":true,"smoothing":{"amount":0.7}})).unwrap();
        photocraft_ui_egui::panels::activate_pixel_pencil(&mut app).unwrap();
        assert_eq!(app.ui.tool, Tool::Pencil);
        assert_eq!(app.session.tools.brush.size, 1.0);
        assert!(!app.session.tools.brush.pressure_size);
        app.run("paint.pencil", json!({"points":[[2.2,2.2],[8.2,8.2]],"color":"#ff0000"})).unwrap();
        let bytes = photocraft_io::export(&app.session.active().unwrap().doc, "pixels.png", &Default::default()).unwrap().bytes;
        let image = photocraft_codecs::decode(&bytes).unwrap();
        assert_eq!(image.width(), 16);
        let pixels = image.to_rgba8();
        let painted: Vec<_> = pixels.chunks_exact(4).filter(|p| p[3] > 0).collect();
        assert_eq!(painted.len(), 7);
        assert!(painted.iter().all(|p| *p == [255, 0, 0, 255]));
    }
}
