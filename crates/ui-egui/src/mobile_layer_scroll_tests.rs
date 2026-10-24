use super::*;

struct TouchList {
    app: PhotocraftApp,
    ctx: egui::Context,
    time: f64,
    scroll: egui::Id,
}

impl TouchList {
    fn new() -> Self {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({"width": 16, "height": 16})).unwrap();
        for i in 0..40 {
            app.run("layer.new.layer", json!({"name": format!("Row {i:02}")})).unwrap();
        }
        let ctx = egui::Context::default();
        PhotocraftApp::setup_context(&ctx, crate::theme::ThemeKind::ProMedium);
        let mut list = Self { app, ctx, time: 0.0, scroll: egui::Id::NULL };
        list.frames(4);
        list
    }

    fn frame(&mut self, events: Vec<egui::Event>) {
        self.time += 1.0 / 60.0;
        let mut out = self.ctx.run_ui(
            egui::RawInput { time: Some(self.time), screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(320.0, 480.0))), events, ..Default::default() },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    self.scroll = ui.make_persistent_id(egui::IdSalt::new(("layer-rows", self.app.session.active().unwrap().doc.id.0)));
                    layers(&mut self.app, ui);
                });
            },
        );
        out.textures_delta.clear();
    }

    fn frames(&mut self, count: usize) {
        for _ in 0..count {
            self.frame(vec![]);
        }
    }

    fn touch(&mut self, phase: egui::TouchPhase, pos: Pos2) {
        let mut events =
            vec![egui::Event::Touch { device_id: egui::TouchDeviceId(1), id: egui::TouchId(1), phase, pos, force: None }, egui::Event::PointerMoved(pos)];
        if phase != egui::TouchPhase::Move {
            events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: phase == egui::TouchPhase::Start,
                modifiers: Default::default(),
            });
        }
        if phase == egui::TouchPhase::End || phase == egui::TouchPhase::Cancel {
            events.push(egui::Event::PointerGone);
        }
        self.frame(events);
    }

    fn offset(&self) -> f32 {
        egui::scroll_area::State::load(&self.ctx, self.scroll).unwrap().offset.y
    }

    fn row(&self, index: usize) -> Pos2 {
        crate::layer_row_ui::recorded(&self.ctx).get(index).unwrap().row.center()
    }

    fn flick(&mut self) {
        let from = self.row(6);
        self.touch(egui::TouchPhase::Start, from);
        for step in 1..=8 {
            self.touch(egui::TouchPhase::Move, from - vec2(0.0, step as f32 * 14.0));
        }
        self.touch(egui::TouchPhase::End, from - vec2(0.0, 112.0));
        self.frames(2);
    }
}

#[test]
fn touching_a_fling_stops_it_before_rows_move_under_the_finger() {
    let mut h = TouchList::new();
    h.flick();
    let before = h.offset();
    h.frames(2);
    assert!(h.offset() > before + 1.0, "a flick should coast");
    let at = h.row(3);
    h.touch(egui::TouchPhase::Start, at);
    let stopped = h.offset();
    h.frames(8);
    assert!((h.offset() - stopped).abs() < 0.5, "touching a moving list must stop it: {stopped} -> {}", h.offset());
    assert!(!egui::Popup::is_any_open(&h.ctx));
}

#[test]
fn a_small_vertical_drag_locks_scrolling_before_the_hold_timeout() {
    let mut h = TouchList::new();
    let from = h.row(5);
    h.touch(egui::TouchPhase::Start, from);
    // A very slow 6-point movement on a high-density phone must be a scroll.
    for step in 1..=20 {
        h.touch(egui::TouchPhase::Move, from - vec2(0.0, step as f32 * 0.3));
        h.frames(2);
        assert!(!egui::Popup::is_any_open(&h.ctx), "continuous slow movement must not count as a stationary hold");
    }
    h.frames(36);
    assert!(!egui::Popup::is_any_open(&h.ctx), "a slow scroll must not turn into a layer menu");
    assert!(h.offset() >= 5.0, "the list should follow even a very small slow drag");
}

#[test]
fn a_left_swipe_with_finger_drift_keeps_the_list_stationary() {
    let mut h = TouchList::new();
    let selected = h.app.session.active().unwrap().selected_layers();
    let from = h.row(4);
    let before = h.offset();
    h.touch(egui::TouchPhase::Start, from);
    for step in 1..=6 {
        h.touch(egui::TouchPhase::Move, from - vec2(step as f32 * 8.0, step as f32));
    }
    h.touch(egui::TouchPhase::End, from - vec2(48.0, 6.0));
    h.frames(4);
    assert!((h.offset() - before).abs() < 0.5, "left-swipe selection must not scroll: {before} -> {}", h.offset());
    assert_eq!(h.app.session.active().unwrap().selected_layers().len(), selected.len() + 1);
}

#[test]
fn vertical_scroll_from_eyes_and_thumbnails_keeps_selection_visibility_and_order() {
    for x in [18.0, 50.0, 155.0] {
        let mut h = TouchList::new();
        let st = h.app.session.active().unwrap();
        let selected = st.selected_layers();
        let before: Vec<_> = st.doc.layers.iter().map(|l| (l.id, l.visible)).collect();
        let from = pos2(x, h.row(6).y);
        h.touch(egui::TouchPhase::Start, from);
        for step in 1..=12 {
            h.touch(egui::TouchPhase::Move, from - vec2(0.0, step as f32 * 6.0));
        }
        h.touch(egui::TouchPhase::End, from - vec2(0.0, 72.0));
        assert!(h.offset() >= 65.0, "x={x}, offset={}", h.offset());
        assert!(!egui::Popup::is_any_open(&h.ctx));
        let st = h.app.session.active().unwrap();
        assert_eq!(st.selected_layers(), selected);
        assert_eq!(st.doc.layers.iter().map(|l| (l.id, l.visible)).collect::<Vec<_>>(), before);
    }
}

#[test]
fn a_new_swipe_after_scrolling_still_selects_without_vertical_drift() {
    let mut h = TouchList::new();
    h.flick();
    // Catch the fling, then release: this touch must not choose a row.
    let selected = h.app.session.active().unwrap().selected_layers();
    let at = h.row(3);
    h.touch(egui::TouchPhase::Start, at);
    h.touch(egui::TouchPhase::End, at);
    h.frames(2);
    assert_eq!(h.app.session.active().unwrap().selected_layers(), selected);
    let offset = h.offset();
    let from = h.row(4);
    h.touch(egui::TouchPhase::Start, from);
    h.touch(egui::TouchPhase::Move, from - vec2(40.0, 4.0));
    h.touch(egui::TouchPhase::End, from - vec2(40.0, 4.0));
    h.frames(4);
    assert_eq!(h.app.session.active().unwrap().selected_layers().len(), selected.len() + 1);
    assert!((h.offset() - offset).abs() < 0.5);
}

#[test]
fn cancelled_scroll_stops_without_coasting_or_lingering_drag_state() {
    let mut h = TouchList::new();
    let from = h.row(6);
    h.touch(egui::TouchPhase::Start, from);
    for step in 1..=5 {
        h.touch(egui::TouchPhase::Move, from - vec2(0.0, step as f32 * 12.0));
    }
    h.touch(egui::TouchPhase::Cancel, from - vec2(0.0, 60.0));
    let offset = h.offset();
    h.frames(40);
    assert!((h.offset() - offset).abs() < 0.5);
    assert!(!egui::Popup::is_any_open(&h.ctx));
    assert!(h.ctx.data(|d| d.get_temp::<TouchLayerGesture>(touch_layer_key()).is_none()));
    assert!(h.ctx.data(|d| d.get_temp::<u64>(egui::Id::new("layer-drag")).is_none()));
}

#[test]
fn reversing_a_scroll_and_dragging_back_to_the_origin_never_opens_a_menu() {
    let mut h = TouchList::new();
    let from = h.row(5);
    h.touch(egui::TouchPhase::Start, from);
    h.touch(egui::TouchPhase::Move, from - vec2(0.0, 14.0));
    h.touch(egui::TouchPhase::Move, from);
    h.frames(40);
    assert!(!egui::Popup::is_any_open(&h.ctx));
    assert!(h.offset() < 0.5);
    h.touch(egui::TouchPhase::End, from);
    h.frames(10);
    assert!(h.offset() < 0.5);
}

#[test]
fn switching_documents_keeps_each_touch_list_position_and_cancels_momentum() {
    let mut h = TouchList::new();
    h.flick();
    let first_scroll = h.scroll;
    let first_offset = h.offset();
    assert!(first_offset > 50.0);
    h.app.run("file.new", json!({"width":16,"height":16})).unwrap();
    for i in 0..40 {
        h.app.run("layer.new.layer", json!({"name": format!("Second {i}")})).unwrap();
    }
    h.frames(3);
    assert_ne!(h.scroll, first_scroll);
    assert!(h.offset().abs() < 0.5);
    h.flick();
    assert!(h.offset() > 50.0);
    assert!(h.app.session.set_active(0));
    h.frames(3);
    assert_eq!(h.scroll, first_scroll);
    assert!((h.offset() - first_offset).abs() < 0.5);
    let restored = h.offset();
    h.frames(20);
    assert!((h.offset() - restored).abs() < 0.5);
}
