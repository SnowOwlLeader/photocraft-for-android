//! #589: layers dragged onto another document's tab, then dropped on its canvas, are copied there.

use egui::accesskit::Role;
use egui::{Modifiers, PointerButton, Pos2, Rect, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use photocraft_doc::LayerId;
use serde_json::json;

use crate::PhotocraftApp;
use crate::canvas::ViewXform;
use crate::state::Tool;

/// "source-doc" (40×30) with a red 10×10 square at (4, 6) on layer "paint", and "dest-doc"
/// (60×50). The source is active.
fn harness() -> (Harness<'static, PhotocraftApp>, LayerId) {
    let mut s = photocraft_engine::Session::new();
    s.execute("file.new", json!({"width": 40, "height": 30, "name": "source-doc"})).unwrap();
    let paint = LayerId(s.execute("layer.new.layer", json!({"name": "paint"})).unwrap()["layer"].as_u64().unwrap());
    s.execute("select.rect", json!({"x": 4, "y": 6, "width": 10, "height": 10})).unwrap();
    s.execute("edit.fill", json!({"color": "#ff0000"})).unwrap();
    s.execute("select.deselect", json!({})).unwrap();
    s.execute("file.new", json!({"width": 60, "height": 50, "name": "dest-doc"})).unwrap();
    s.set_active(0);
    let mut h = Harness::builder().with_size(vec2(1440.0, 900.0)).with_max_steps(64).build_eframe(move |cc| {
        PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
        PhotocraftApp::new(s, crate::Services::default())
    });
    h.run_steps(8);
    (h, paint)
}

/// A document tab, found by its accessible title ("name @ 100% (…)", see `pro_tabs`).
fn tab(h: &Harness<'_, PhotocraftApp>, name: &str) -> Pos2 {
    h.get_by_label_contains(&format!("{name} @")).rect().center()
}

/// Press at `path[0]`, move through the rest (a few frames at each) and release at the last
/// point with `release` held.
fn drag_along(h: &mut Harness<'_, PhotocraftApp>, path: &[Pos2], release: Modifiers) {
    let first = path[0];
    h.event(egui::Event::PointerMoved(first));
    h.run_steps(1);
    h.event(egui::Event::PointerButton { pos: first, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
    h.run_steps(1);
    for w in path.windows(2) {
        for k in 1..=4 {
            h.event(egui::Event::PointerMoved(w[0] + (w[1] - w[0]) * (k as f32 / 4.0)));
            h.run_steps(1);
        }
        h.run_steps(2);
    }
    let last = path[path.len() - 1];
    h.event(egui::Event::ModifiersChanged(release));
    h.event(egui::Event::PointerButton { pos: last, button: PointerButton::Primary, pressed: false, modifiers: release });
    h.run_steps(3);
    h.event(egui::Event::ModifiersChanged(Modifiers::NONE));
    h.run_steps(1);
}

fn mark_touch_screen(h: &mut Harness<'_, PhotocraftApp>) {
    for phase in [egui::TouchPhase::Start, egui::TouchPhase::End] {
        h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(1), id: egui::TouchId(1), phase, pos: Pos2::ZERO, force: None });
        h.run_steps(1);
    }
}

fn swipe_layer_left(h: &mut Harness<'_, PhotocraftApp>, label: &str) {
    let row = h.get_by_role_and_label(Role::Button, label).rect().center();
    h.event(egui::Event::PointerMoved(row));
    h.run_steps(1);
    h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(3), id: egui::TouchId(3), phase: egui::TouchPhase::Start, pos: row, force: None });
    h.event(egui::Event::PointerButton { pos: row, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
    h.run_steps(1);
    for distance in [20.0, 50.0] {
        h.event(egui::Event::PointerMoved(row - vec2(distance, 0.0)));
        h.run_steps(1);
    }
    h.event(egui::Event::Touch {
        device_id: egui::TouchDeviceId(3),
        id: egui::TouchId(3),
        phase: egui::TouchPhase::End,
        pos: row - vec2(50.0, 0.0),
        force: None,
    });
    h.event(egui::Event::PointerButton { pos: row - vec2(50.0, 0.0), button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
    h.run_steps(2);
}

#[test]
fn a_short_swipe_released_between_frames_keeps_the_previous_selection() {
    let (mut h, paint) = harness();
    let second = LayerId(h.state_mut().run("layer.new.layer", json!({"name":"second"})).unwrap()["layer"].as_u64().unwrap());
    h.run_steps(2);
    mark_touch_screen(&mut h);
    let from = h.get_by_role_and_label(Role::Button, "paint").rect().center();
    h.event(egui::Event::PointerMoved(from));
    h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(3), id: egui::TouchId(3), phase: egui::TouchPhase::Start, pos: from, force: None });
    h.event(egui::Event::PointerButton { pos: from, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
    h.run_steps(1);
    let to = from - vec2(28.0, 2.0);
    h.event(egui::Event::PointerMoved(to));
    h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(3), id: egui::TouchId(3), phase: egui::TouchPhase::End, pos: to, force: None });
    h.event(egui::Event::PointerButton { pos: to, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
    h.event(egui::Event::PointerGone);
    h.run_steps(3);
    let selected = h.state().session.active().unwrap().selected_layers();
    assert!(selected.contains(&paint) && selected.contains(&second), "a fast touch release must add the swiped layer: {selected:?}");
}

#[test]
fn a_touch_hold_release_keeps_the_menu_open_and_its_action_targets_the_held_row() {
    {
        let (mut h, paint) = harness();
        let other = LayerId(h.state_mut().run("layer.new.layer", json!({"name": "other"})).unwrap()["layer"].as_u64().unwrap());
        h.run_steps(2);
        mark_touch_screen(&mut h);
        let row = h.get_by_role_and_label(Role::Button, "paint").rect().center();
        h.event(egui::Event::PointerMoved(row));
        h.run_steps(1);
        h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(3), id: egui::TouchId(3), phase: egui::TouchPhase::Start, pos: row, force: None });
        h.event(egui::Event::PointerButton { pos: row, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
        h.run_steps(8);
        assert!(egui::Popup::is_any_open(&h.ctx));
        assert!(super::get(&h.ctx).is_some(), "holding still arms copying without changing any document");
        h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(3), id: egui::TouchId(3), phase: egui::TouchPhase::End, pos: row, force: None });
        h.event(egui::Event::PointerButton { pos: row, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
        h.event(egui::Event::PointerGone);
        h.run_steps(3);
        assert!(egui::Popup::is_any_open(&h.ctx), "lifting after the hold must keep the menu open");
        assert_eq!(h.state().session.documents()[0].doc.layers.len(), 3, "the initial finger release must not run an action");
        h.get_by_role_and_label(Role::Button, "Delete Layer").click();
        h.run_steps(3);
        let doc = &h.state().session.documents()[0].doc;
        assert!(doc.layer(paint).is_none(), "the held layer is deleted");
        assert!(doc.layer(other).is_some(), "the originally selected layer remains");
        assert!(!egui::Popup::is_any_open(&h.ctx), "choosing an action closes the menu");
    }
}

#[test]
fn a_touch_hold_menu_preserves_a_multiselection_and_closes_on_outside_tap() {
    let (mut h, paint) = harness();
    let other = LayerId(h.state_mut().run("layer.new.layer", json!({"name": "other"})).unwrap()["layer"].as_u64().unwrap());
    h.run_steps(2);
    mark_touch_screen(&mut h);
    swipe_layer_left(&mut h, "paint");
    let row = h.get_by_role_and_label(Role::Button, "paint").rect().center();
    h.event(egui::Event::PointerMoved(row));
    h.run_steps(1);
    h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(3), id: egui::TouchId(3), phase: egui::TouchPhase::Start, pos: row, force: None });
    h.event(egui::Event::PointerButton { pos: row, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
    h.run_steps(8);
    h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(3), id: egui::TouchId(3), phase: egui::TouchPhase::End, pos: row, force: None });
    h.event(egui::Event::PointerButton { pos: row, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
    h.event(egui::Event::PointerGone);
    h.run_steps(3);
    assert!(egui::Popup::is_any_open(&h.ctx));
    h.get_by_role_and_label(Role::Button, "Delete Layers");
    let selected = h.state().session.active().unwrap().selected_layers();
    assert!(selected.contains(&paint) && selected.contains(&other));
    let outside = h.state().last_canvas_rect.center();
    h.event(egui::Event::PointerMoved(outside));
    h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(3), id: egui::TouchId(3), phase: egui::TouchPhase::Start, pos: outside, force: None });
    h.event(egui::Event::PointerButton { pos: outside, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
    h.run_steps(1);
    h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(3), id: egui::TouchId(3), phase: egui::TouchPhase::End, pos: outside, force: None });
    h.event(egui::Event::PointerButton { pos: outside, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
    h.run_steps(3);
    assert!(!egui::Popup::is_any_open(&h.ctx));
}

#[test]
fn a_short_stationary_hold_reported_as_a_click_does_not_replace_multiselection() {
    let (mut h, paint) = harness();
    let other = LayerId(h.state_mut().run("layer.new.layer", json!({"name": "other"})).unwrap()["layer"].as_u64().unwrap());
    h.run_steps(2);
    mark_touch_screen(&mut h);
    swipe_layer_left(&mut h, "paint");
    let row = h.get_by_role_and_label(Role::Button, "paint").rect().center();
    // Android can classify the stationary release as a primary click as well as a hold.
    h.ctx.options_mut(|o| o.input_options.max_click_duration = 2.0);
    h.event(egui::Event::PointerMoved(row));
    h.run_steps(1);
    h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(3), id: egui::TouchId(3), phase: egui::TouchPhase::Start, pos: row, force: None });
    h.event(egui::Event::PointerButton { pos: row, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
    h.run_steps(6);
    h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(3), id: egui::TouchId(3), phase: egui::TouchPhase::End, pos: row, force: None });
    h.event(egui::Event::PointerButton { pos: row, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
    h.event(egui::Event::PointerGone);
    h.run_steps(3);
    assert!(egui::Popup::is_any_open(&h.ctx));
    h.get_by_role_and_label(Role::Button, "Delete Layers");
    let selected = h.state().session.active().unwrap().selected_layers();
    assert!(selected.contains(&paint) && selected.contains(&other), "the release click must not collapse the selection");
}

#[test]
fn touch_swipes_select_layers_and_a_long_press_copies_them_to_another_document() {
    let (mut h, paint) = harness();
    let second = LayerId(h.state_mut().run("layer.new.layer", json!({"name": "second"})).unwrap()["layer"].as_u64().unwrap());
    let third = LayerId(h.state_mut().run("layer.new.layer", json!({"name": "third"})).unwrap()["layer"].as_u64().unwrap());
    h.run_steps(2);
    mark_touch_screen(&mut h);
    swipe_layer_left(&mut h, "paint");
    swipe_layer_left(&mut h, "second");
    let selected = h.state().session.active().unwrap().selected_layers();
    assert!(selected.contains(&paint) && selected.contains(&second) && selected.contains(&third), "left swipes add rows to the selection: {selected:?}");

    let row = h.get_by_role_and_label(Role::Button, "second").rect().center();
    let destination = tab(&h, "dest-doc");
    h.event(egui::Event::PointerMoved(row));
    h.run_steps(1);
    h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(3), id: egui::TouchId(3), phase: egui::TouchPhase::Start, pos: row, force: None });
    h.event(egui::Event::PointerButton { pos: row, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
    h.run_steps(8);
    assert!(super::get(&h.ctx).is_some(), "holding a row arms the transfer");
    assert!(egui::Popup::is_any_open(&h.ctx), "a stationary hold opens the layer context menu");
    h.event(egui::Event::PointerMoved(destination));
    h.run_steps(2);
    assert!(!egui::Popup::is_any_open(&h.ctx), "moving the held finger dismisses the menu for transfer");
    h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(3), id: egui::TouchId(3), phase: egui::TouchPhase::End, pos: destination, force: None });
    h.event(egui::Event::PointerButton { pos: destination, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
    h.event(egui::Event::PointerGone);
    h.run_steps(3);

    assert_eq!(h.state().session.documents()[0].doc.layers.len(), 4, "the source remains unchanged");
    let target = h.state().session.documents()[1].doc.clone();
    assert_eq!(target.layers.len(), 4, "three selected layers were copied");
    for name in ["paint", "second", "third"] {
        assert!(target.layers.iter().any(|layer| layer.name == name), "missing {name}");
    }
}

/// Native Android input includes both the real Touch and its emulated primary pointer.
fn touch_at(h: &mut Harness<'_, PhotocraftApp>, phase: egui::TouchPhase, pos: Pos2) {
    h.event(egui::Event::Touch { device_id: egui::TouchDeviceId(8), id: egui::TouchId(8), phase, pos, force: None });
    h.event(egui::Event::PointerMoved(pos));
    match phase {
        egui::TouchPhase::Start | egui::TouchPhase::End => {
            h.event(egui::Event::PointerButton { pos, button: PointerButton::Primary, pressed: phase == egui::TouchPhase::Start, modifiers: Modifiers::NONE });
            if phase == egui::TouchPhase::End {
                h.event(egui::Event::PointerGone);
            }
        }
        _ => {}
    }
}

fn hold_and_reorder(h: &mut Harness<'_, PhotocraftApp>, source: &str, to: Pos2, same_frame_release: bool) {
    let from = h.get_by_role_and_label(Role::Button, source).rect().center();
    touch_at(h, egui::TouchPhase::Start, from);
    h.run_steps(8);
    assert!(egui::Popup::is_any_open(&h.ctx));
    touch_at(h, egui::TouchPhase::Move, to);
    if !same_frame_release {
        h.run_steps(2);
        assert!(!egui::Popup::is_any_open(&h.ctx));
    }
    touch_at(h, egui::TouchPhase::End, to);
    h.run_steps(3);
}

#[test]
fn touch_hold_reorders_above_and_below_and_undo_restores_the_stack() {
    for (above, same_frame) in [(true, false), (false, true)] {
        let (mut h, paint) = harness();
        let second = LayerId(h.state_mut().run("layer.new.layer", json!({"name":"second"})).unwrap()["layer"].as_u64().unwrap());
        h.run_steps(3);
        let (source, target) = if above { ("paint", "second") } else { ("second", "paint") };
        let row = h.get_by_role_and_label(Role::Button, target).rect();
        let to = egui::pos2(row.center().x, if above { row.top() + 4.0 } else { row.bottom() - 4.0 });
        let before: Vec<_> = h.state().session.active().unwrap().doc.layers.iter().map(|l| l.id).collect();
        let steps = h.state().session.active().unwrap().history.entries().len();
        hold_and_reorder(&mut h, source, to, same_frame);
        let st = h.state().session.active().unwrap();
        assert_eq!(st.doc.layers.iter().rev().take(2).map(|l| l.id).collect::<Vec<_>>(), [paint, second]);
        assert_eq!(st.history.entries().len(), steps + 1, "a reorder is one undo step");
        h.state_mut().run("edit.undo", json!({})).unwrap();
        assert_eq!(h.state().session.active().unwrap().doc.layers.iter().map(|l| l.id).collect::<Vec<_>>(), before);
        assert_eq!(h.state().session.documents()[1].doc.layers.len(), 1, "reorder must not copy into another project");
    }
}

#[test]
fn touch_hold_moves_a_multiselection_into_a_group_in_stable_order() {
    let (mut h, paint) = harness();
    let second = LayerId(h.state_mut().run("layer.new.layer", json!({"name":"second"})).unwrap()["layer"].as_u64().unwrap());
    let group = LayerId(h.state_mut().run("layer.new.group", json!({"name":"folder"})).unwrap()["layer"].as_u64().unwrap());
    h.state_mut().run("layer.select", json!({"layer":second.0})).unwrap();
    h.run_steps(3);
    swipe_layer_left(&mut h, "paint");
    h.run_steps(2);
    let to = h.get_by_role_and_label(Role::Button, "folder").rect().center();
    hold_and_reorder(&mut h, "paint", to, false);
    let st = h.state().session.active().unwrap();
    let photocraft_doc::LayerContent::Group(g) = &st.doc.layer(group).unwrap().content else { panic!("not a group") };
    assert_eq!(g.children.iter().map(|l| l.id).collect::<Vec<_>>(), [paint, second]);
    assert_eq!(st.selected_layers().len(), 2);
}

#[test]
fn mouse_drag_keeps_working_after_touch_and_alt_drag_copies() {
    for copy in [false, true] {
        let (mut h, paint) = harness();
        let second = LayerId(h.state_mut().run("layer.new.layer", json!({"name":"second"})).unwrap()["layer"].as_u64().unwrap());
        h.run_steps(3);
        mark_touch_screen(&mut h); // Historical touch-screen discovery must not disable the mouse.
        let from = h.get_by_role_and_label(Role::Button, "paint").rect().center();
        let target = h.get_by_role_and_label(Role::Button, "second").rect();
        let to = egui::pos2(target.center().x, target.top() + 4.0);
        drag_along(&mut h, &[from, to], if copy { Modifiers::ALT } else { Modifiers::NONE });
        let st = h.state().session.active().unwrap();
        assert_eq!(st.doc.layers.len(), if copy { 4 } else { 3 });
        assert_eq!(st.doc.layers[1].id, if copy { paint } else { second });
        assert!(!egui::Popup::is_any_open(&h.ctx));
    }
}

/// The copy in "dest-doc" (its active layer) and its bounds.
fn copy(h: &Harness<'_, PhotocraftApp>) -> (String, Rect) {
    let st = h.state().session.active().unwrap();
    assert_eq!(st.doc.name, "dest-doc", "the destination is shown");
    let l = st.active_layer.and_then(|id| st.doc.layer(id)).unwrap();
    let b = photocraft_engine::layer_multi_cmds::layer_bounds(l).unwrap();
    (l.name.clone(), Rect::from_min_max(egui::pos2(b.x0 as f32, b.y0 as f32), egui::pos2(b.x1 as f32, b.y1 as f32)))
}

fn source_square(h: &Harness<'_, PhotocraftApp>, paint: LayerId) -> photocraft_geom::Rect {
    let doc = &h.state().session.documents()[0].doc;
    photocraft_engine::layer_multi_cmds::layer_bounds(doc.layer(paint).unwrap()).unwrap()
}

#[test]
fn a_layers_panel_drag_onto_a_tab_drops_the_layer_where_it_is_released() {
    let (mut h, _) = harness();
    let row = h.get_by_role_and_label(Role::Button, "paint").rect().center();
    let over_tab = tab(&h, "dest-doc");
    let drop = h.state().last_canvas_rect.center();
    drag_along(&mut h, &[row, over_tab, drop], Modifiers::NONE);
    let at = ViewXform::active(h.state()).unwrap().to_doc(drop);
    let (name, r) = copy(&h);
    assert_eq!(name, "paint");
    assert!((r.center().x - at[0] as f32).abs() <= 1.0 && (r.center().y - at[1] as f32).abs() <= 1.0, "centred on the drop point {at:?}: {r:?}");
    assert_eq!(h.state().session.documents()[0].doc.layers.len(), 2, "the source keeps its layers");
    assert_eq!(h.state().session.active().unwrap().doc.layers.len(), 2);
}

#[test]
fn shift_centres_the_copy_when_the_documents_differ_in_size() {
    let (mut h, _) = harness();
    let row = h.get_by_role_and_label(Role::Button, "paint").rect().center();
    let over_tab = tab(&h, "dest-doc");
    let drop = h.state().last_canvas_rect.center() + vec2(80.0, 60.0);
    drag_along(&mut h, &[row, over_tab, drop], Modifiers::SHIFT);
    assert_eq!(copy(&h).1, Rect::from_min_max(egui::pos2(25.0, 20.0), egui::pos2(35.0, 30.0)));
}

#[test]
fn a_move_tool_drag_onto_a_tab_copies_the_layer_under_the_pointer_and_leaves_the_source() {
    let (mut h, paint) = harness();
    h.state_mut().ui.tool = Tool::Move;
    h.run_steps(2);
    let before = source_square(&h, paint);
    let steps = h.state().session.documents()[0].history.entries().len();
    let grab = ViewXform::active(h.state()).unwrap().to_screen(9.0, 11.0);
    let grab_doc = ViewXform::active(h.state()).unwrap().to_doc(grab);
    let over_tab = tab(&h, "dest-doc");
    let drop = h.state().last_canvas_rect.center();
    drag_along(&mut h, &[grab, over_tab, drop], Modifiers::NONE);
    let at = ViewXform::active(h.state()).unwrap().to_doc(drop);
    let (name, r) = copy(&h);
    assert_eq!(name, "paint");
    // The grabbed pixel lands under the pointer.
    let want = egui::pos2((9.0 + at[0] - grab_doc[0]) as f32, (11.0 + at[1] - grab_doc[1]) as f32);
    assert!((r.center() - want).length() <= 1.5, "{r:?} around {want:?}");
    assert_eq!(source_square(&h, paint), before, "the source layer did not move");
    assert_eq!(h.state().session.documents()[0].history.entries().len(), steps, "no Move step in the source");
}

#[test]
fn dropping_back_on_the_source_copies_nothing() {
    let (mut h, _) = harness();
    let row = h.get_by_role_and_label(Role::Button, "paint").rect().center();
    let (dest_tab, source_tab) = (tab(&h, "dest-doc"), tab(&h, "source-doc"));
    let drop = h.state().last_canvas_rect.center();
    drag_along(&mut h, &[row, dest_tab, source_tab, drop], Modifiers::NONE);
    let docs = h.state().session.documents();
    assert_eq!((docs[0].doc.layers.len(), docs[1].doc.layers.len()), (2, 1));
    assert_eq!(h.state().session.active_index(), Some(0));
}
