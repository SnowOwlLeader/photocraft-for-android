//! Native egui file manager; Android provider work runs on the document worker.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]
use serde_json::{Value, json};

#[derive(Clone, Debug)]
pub struct Entry {
    pub uri: String,
    pub name: String,
    pub directory: bool,
    pub size: u64,
}

#[derive(Default)]
pub struct Browser {
    pub request: Option<Value>,
    pub busy: bool,
    pub error: String,
    pub current: String,
    pub label: String,
    pub history: Vec<(String, String)>,
    pub roots: Vec<Entry>,
    pub entries: Vec<Entry>,
    pub selected: Vec<String>,
    pub search: String,
    pub filename: String,
    pub edit: Option<(&'static str, String, String)>,
    pub name_input: String,
    pub confirm: Option<Value>,
}

fn entries(v: &Value) -> Vec<Entry> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|e| {
            Some(Entry {
                uri: e.get("uri")?.as_str()?.into(),
                name: e.get("name")?.as_str()?.into(),
                directory: e.get("directory").and_then(Value::as_bool).unwrap_or(false),
                size: e.get("size").and_then(Value::as_u64).unwrap_or(0),
            })
        })
        .collect()
}

pub fn valid_name(name: &str) -> bool {
    !name.trim().is_empty() && name != "." && name != ".." && name.chars().count() <= 240 && !name.chars().any(|c| c.is_control() || "/\\:*?\"<>|".contains(c))
}

impl Browser {
    pub fn begin(&mut self, request: Value) -> Value {
        self.filename = request.get("name").and_then(Value::as_str).unwrap_or("作品.psd").into();
        self.request = Some(request);
        self.selected.clear();
        self.error.clear();
        self.search.clear();
        self.edit = None;
        self.confirm = None;
        self.list()
    }

    pub fn list(&mut self) -> Value {
        self.busy = true;
        json!({"action":"list","uri":self.current})
    }

    pub fn receive(&mut self, response: &Value) {
        self.busy = false;
        if let Some(error) = response.get("error").and_then(Value::as_str) {
            self.error = error.into();
            return;
        }
        self.error.clear();
        if response.get("cancelled").is_some() {
            return;
        }
        if let Some(uri) = response.get("uri").and_then(Value::as_str) {
            self.current = uri.into();
        }
        if let Some(label) = response.get("name").and_then(Value::as_str) {
            self.label = label.into();
        }
        self.roots = entries(&response["roots"]);
        self.entries = entries(&response["entries"]);
        self.entries.sort_by(|a, b| b.directory.cmp(&a.directory).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
        self.selected.retain(|uri| self.entries.iter().any(|e| &e.uri == uri));
    }

    fn navigate(&mut self, uri: String, name: String) -> Value {
        self.history.push((self.current.clone(), self.label.clone()));
        self.current = uri;
        self.label = name;
        self.selected.clear();
        self.search.clear();
        self.list()
    }

    /// `cancel` is handled by the shell; every other action is sent to the platform worker.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<Value> {
        let request = self.request.clone()?;
        let save = request.get("save").and_then(Value::as_bool).unwrap_or(false);
        let fonts = request.get("transient").and_then(Value::as_bool).unwrap_or(false);
        let folder = request.get("folder").and_then(Value::as_bool).unwrap_or(false);
        let mut action = None;
        let area = ctx.content_rect();
        let width = (area.width() - 32.0).clamp(140.0, 660.0);
        let modal = egui::Modal::new(egui::Id::new("android-file-manager")).show(ctx, |ui| {
            ui.set_width(width);
            egui::ScrollArea::vertical().id_salt("native-files-dialog").max_height((area.height() - 112.0).max(80.0)).auto_shrink([false, false]).show(
                ui,
                |ui| {
                    ui.heading(if folder {
                        "选择文件夹"
                    } else if save {
                        "另存为"
                    } else if fonts {
                        "选择字体"
                    } else {
                        "文件管理器"
                    });
                    ui.horizontal_wrapped(|ui| {
                        if ui.add_enabled(!self.busy && !self.history.is_empty(), egui::Button::new("← 返回")).clicked()
                            && let Some((uri, name)) = self.history.pop()
                        {
                            self.current = uri;
                            self.label = name;
                            self.selected.clear();
                            self.search.clear();
                            action = Some(self.list());
                        }
                        if ui.add_enabled(!self.busy, egui::Button::new("刷新")).clicked() {
                            action = Some(self.list());
                        }
                        if ui.add_enabled(!self.busy, egui::Button::new("添加手机目录")).clicked() {
                            action = Some(json!({"action":"authorize"}));
                        }
                        if ui.add_enabled(!self.busy && !folder, egui::Button::new("系统选择器")).clicked() {
                            action = Some(json!({"action":"system","request":request}));
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        for root in self.roots.clone() {
                            if ui.add_enabled(!self.busy, egui::Button::new(&root.name).selected(self.current == root.uri)).clicked() {
                                self.history.clear();
                                action = Some(self.navigate(root.uri, root.name));
                            }
                        }
                    });
                    ui.label(egui::RichText::new(&self.label).strong());
                    if ui.add_enabled(!self.busy, egui::TextEdit::singleline(&mut self.search).hint_text("搜索当前目录").desired_width(f32::INFINITY)).clicked()
                    {
                        action = Some(json!({"action":"input","field":"search","title":"搜索当前目录","value":self.search}));
                    }
                    ui.horizontal_wrapped(|ui| {
                        if ui.add_enabled(!self.busy, egui::Button::new("新建文件夹")).clicked() {
                            self.edit = Some(("mkdir", self.current.clone(), "新建文件夹".into()));
                            self.name_input = "新建文件夹".into();
                        }
                        if !save {
                            ui.small(format!("已选 {} 个文件", self.selected.len()));
                        }
                    });
                    if !self.error.is_empty() {
                        ui.colored_label(ui.visuals().error_fg_color, &self.error);
                    }
                    if self.busy {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("正在读取…");
                        });
                    }
                    let rows = self.entries.clone();
                    let filter = self.search.to_lowercase();
                    egui::ScrollArea::vertical().id_salt("native-files-list").max_height((area.height() - 300.0).clamp(64.0, 420.0)).show(ui, |ui| {
                        ui.set_min_width(width);
                        let mut count = 0;
                        for entry in rows.iter().filter(|e| e.name.to_lowercase().contains(&filter)) {
                            count += 1;
                            ui.horizontal(|ui| {
                                let label = format!("{}  {}", if entry.directory { "目录" } else { "文件" }, entry.name);
                                let response = ui.add_enabled(
                                    !self.busy,
                                    egui::Button::new(label)
                                        .selected(self.selected.contains(&entry.uri))
                                        .min_size(egui::vec2((width - 68.0).max(40.0), 36.0))
                                        .wrap(),
                                );
                                if response.clicked() {
                                    if entry.directory {
                                        action = Some(self.navigate(entry.uri.clone(), entry.name.clone()));
                                    } else if save {
                                        self.filename = entry.name.clone();
                                    } else if let Some(i) = self.selected.iter().position(|s| s == &entry.uri) {
                                        self.selected.remove(i);
                                    } else if self.selected.len() < 16 {
                                        self.selected.push(entry.uri.clone());
                                    }
                                }
                                ui.add_enabled_ui(!self.busy, |ui| {
                                    ui.menu_button("⋯", |ui| {
                                        if !entry.directory {
                                            ui.small(format!("{} 字节", entry.size));
                                        }
                                        if ui.button("重命名").clicked() {
                                            self.edit = Some(("rename", entry.uri.clone(), entry.name.clone()));
                                            self.name_input = entry.name.clone();
                                            ui.close();
                                        }
                                        if ui.button("删除").clicked() {
                                            self.confirm = Some(json!({"action":"delete","uri":entry.uri,"name":entry.name,"directory":self.current}));
                                            ui.close();
                                        }
                                    });
                                });
                            });
                        }
                        if count == 0 && !self.busy {
                            ui.label("当前目录没有匹配的文件");
                        }
                    });
                    ui.separator();
                    if save {
                        ui.label("文件名（通过后缀选择格式）");
                        if ui.add_enabled(!self.busy, egui::TextEdit::singleline(&mut self.filename).desired_width(f32::INFINITY)).clicked() {
                            action = Some(json!({"action":"input","field":"filename","title":"文件名","value":self.filename}));
                        }
                    }
                    if let Some((kind, uri, _)) = self.edit.clone() {
                        ui.label(if kind == "mkdir" { "文件夹名称" } else { "新名称" });
                        if ui.add_enabled(!self.busy, egui::TextEdit::singleline(&mut self.name_input)).clicked() {
                            action = Some(json!({"action":"input","field":"name","title":"输入名称","value":self.name_input}));
                        }
                        ui.horizontal(|ui| {
                            if ui.add_enabled(!self.busy && valid_name(&self.name_input), egui::Button::new("确认名称")).clicked() {
                                action = Some(json!({"action":kind,"uri":uri,"name":self.name_input,"directory":self.current}));
                                self.edit = None;
                            }
                            if ui.button("取消修改").clicked() {
                                self.edit = None;
                            }
                        });
                    }
                    if let Some(confirm) = self.confirm.clone() {
                        ui.label(format!(
                            "{}「{}」？",
                            if confirm["action"] == "delete" { "确认删除" } else { "覆盖已有文件" },
                            confirm["name"].as_str().unwrap_or("")
                        ));
                        ui.horizontal(|ui| {
                            if ui.add_enabled(!self.busy, egui::Button::new("确认")).clicked() {
                                action = Some(confirm);
                                self.confirm = None;
                            }
                            if ui.button("取消操作").clicked() {
                                self.confirm = None;
                            }
                        });
                    }
                },
            );
            ui.separator();
            ui.horizontal(|ui| {
                if photocraft_ui_egui::widgets::secondary_button(ui, "关闭", 72.0).clicked() && !self.busy {
                    action = Some(json!({"action":"cancel"}));
                }
                let can = !self.busy
                    && self.edit.is_none()
                    && self.confirm.is_none()
                    && if folder {
                        !self.current.is_empty()
                    } else if save {
                        valid_name(&self.filename)
                    } else {
                        !self.selected.is_empty()
                    };
                ui.add_enabled_ui(can, |ui| {
                    if photocraft_ui_egui::widgets::primary_button(
                        ui,
                        if folder {
                            "选择此文件夹"
                        } else if save {
                            "保存到此目录"
                        } else {
                            "打开所选文件"
                        },
                        112.0,
                    )
                    .clicked()
                    {
                        if folder {
                            action = Some(json!({"action":"folder","uri":self.current}));
                        } else if save {
                            let save_action = json!({"action":"save","uri":self.current,"name":self.filename});
                            if self.entries.iter().any(|e| e.name == self.filename) {
                                self.confirm = Some(save_action);
                            } else {
                                action = Some(save_action);
                            }
                        } else {
                            action = Some(json!({"action":"open","uris":self.selected,"transient":fonts}));
                        }
                    }
                });
            });
        });
        if modal.should_close() && !self.busy {
            action = Some(json!({"action":"cancel"}));
        }
        if action.is_some() {
            self.busy = true;
        }
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_path_traversal_and_invalid_provider_names() {
        for n in ["", "..", ".", "/sdcard/a", "a\\b", "a\n", "a:b"] {
            assert!(!valid_name(n));
        }
        assert!(valid_name("我的作品.psd"));
    }
    #[test]
    fn provider_listing_sorts_directories_and_keeps_distinct_uris() {
        let mut b = Browser::default();
        b.begin(json!({"multiple":true}));
        b.receive(&json!({"uri":"content://tree/root","name":"作品","roots":[],"entries":[
            {"name":"a.psd","uri":"one"},{"name":"z","uri":"dir","directory":true},{"name":"a.psd","uri":"two"}]}));
        assert_eq!(b.entries.len(), 3);
        assert_eq!(b.entries[0].uri, "dir");
        b.selected = vec!["one".into(), "missing".into()];
        b.receive(&json!({"entries":[{"name":"a.psd","uri":"one"}]}));
        assert_eq!(b.selected, ["one"]);
        b.receive(&json!({"error":"permission expired"}));
        assert_eq!(b.error, "permission expired");
        assert_eq!(b.selected, ["one"]);
    }

    #[test]
    fn small_screen_footer_remains_inside_the_clip_for_long_list_and_save_name() {
        for size in [[320.0, 480.0], [360.0, 568.0], [540.0, 360.0]] {
            for (save, folder) in [(false, false), (true, false), (false, true)] {
                let ctx = egui::Context::default();
                photocraft_ui_egui::PhotocraftApp::setup_context(&ctx, photocraft_ui_egui::theme::ThemeKind::Pro);
                let mut browser = Browser::default();
                browser.begin(json!({"save":save,"folder":folder,"name":"作品.psd"}));
                browser.receive(
                    &json!({"name":"作品","uri":"root","entries":(0..40).map(|n|json!({"uri":n.to_string(),"name":format!("{n}.psd")})).collect::<Vec<_>>()}),
                );
                browser.selected.push("1".into());
                let mut output = Default::default();
                for _ in 0..4 {
                    output = ctx.run_ui(
                        egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(size[0], size[1]))), ..Default::default() },
                        |ui| {
                            browser.show(ui.ctx());
                        },
                    );
                    output.textures_delta.clear();
                }
                for label in [
                    "关闭",
                    if folder {
                        "选择此文件夹"
                    } else if save {
                        "保存到此目录"
                    } else {
                        "打开所选文件"
                    },
                ] {
                    let text = output
                        .shapes
                        .iter()
                        .find_map(|shape| match &shape.shape {
                            egui::epaint::Shape::Text(t) if t.galley.text() == label => Some((t, shape.clip_rect)),
                            _ => None,
                        })
                        .unwrap_or_else(|| panic!("missing {label}"));
                    let bounds = text.0.galley.rect.translate(text.0.pos.to_vec2());
                    assert!(text.1.contains_rect(bounds), "{label} clipped at {size:?}: {bounds:?} / {:?}", text.1);
                    assert!(ctx.content_rect().contains_rect(bounds), "footer offscreen");
                }
            }
        }
    }
}
