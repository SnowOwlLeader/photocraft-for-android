//! Generate the effective default shortcut reference without running commands or opening a window.
use photocraft_ui_egui::{PhotocraftApp, Services, hold_keys, menus, shortcut_dispatch, shortcuts};

fn main() {
    let app = PhotocraftApp::new(photocraft_engine::Session::new(), Services::default());
    let mut bindings = shortcut_dispatch::bindings(&app);
    bindings.sort_by(|a, b| a.0.cmp(&b.0));
    println!("# PhotoCraft 默认键盘快捷键\n");
    println!("由命令注册表生成，共 {} 个有效绑定。Cmd 在安卓和 Windows 上对应 Ctrl。", bindings.len());
    println!("菜单和工具是否可用取决于当前文档、选区、图层类型和编辑状态。用户自定义绑定优先。\n");
    println!("| 命令 | 快捷键 |\n| --- | --- |");
    for (id, sc) in bindings {
        let label =
            menus::UI_COMMANDS.iter().find(|c| c.0 == id).map(|c| c.1).or_else(|| photocraft_engine::commands::find(&id).map(|c| c.label)).unwrap_or(&id);
        let mut parts = Vec::new();
        if sc.modifiers.command || sc.modifiers.ctrl {
            parts.push("Ctrl");
        }
        if sc.modifiers.alt {
            parts.push("Alt");
        }
        if sc.modifiers.shift {
            parts.push("Shift");
        }
        parts.push(sc.logical_key.name());
        let key = parts.join("+");
        println!("| {} (`{}`) | {} |", label.replace('|', "\\|"), id, key.replace('|', "\\|"));
    }
    println!("\n## 按住临时工具\n\n| 操作 | 快捷键 |\n| --- | --- |");
    for (id, label, _) in photocraft_engine::prefs::TEMPORARY_TOOLS {
        if hold_keys::is_temporary(id) {
            println!("| {label} | {} |", shortcuts::shortcut_label(&app, id).unwrap_or_default());
        }
    }
}
