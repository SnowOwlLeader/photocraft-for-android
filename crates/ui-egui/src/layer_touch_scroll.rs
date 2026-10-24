//! Finger scrolling for the Layers list. A single direction decision keeps the scroll
//! area's drag recognizer from competing with swipe selection and held-layer drags.

use egui::{Pos2, Rect};

use super::{TouchLayerMode, touch_layer_mode};

#[derive(Clone)]
struct Finger {
    origin: Pos2,
    last: Pos2,
    mode: TouchLayerMode,
    sampled_at: f64,
    moved_at: f64,
}

#[derive(Clone)]
pub(super) struct LayerTouchScroll {
    key: egui::Id,
    document: photocraft_doc::DocId,
    viewport: Rect,
    max_offset: f32,
    pub offset: f32,
    velocity: f32,
    at: f64,
    finger: Option<Finger>,
    /// A touch that catches a moving list stops it without choosing the row underneath.
    pub caught_fling: bool,
}

impl LayerTouchScroll {
    pub fn begin(ui: &egui::Ui, document: photocraft_doc::DocId, row_mode: Option<TouchLayerMode>) -> Self {
        let ctx = ui.ctx();
        let scroll_id = ui.make_persistent_id(egui::IdSalt::new(("layer-rows", document.0)));
        let key = scroll_id.with("finger-scroll");
        // Offsets belong to documents; an inactive document must not resume its old fling.
        let active_key = ui.id().with("layer-touch-active-document");
        let switched = ctx.data_mut(|d| {
            let previous = d.get_temp::<photocraft_doc::DocId>(active_key);
            d.insert_temp(active_key, document);
            previous != Some(document)
        });
        let touch = super::layer_pointer_is_touch(ctx);
        let (now, pressed, released, down, pos, touch, cancel, wheel) = ctx.input(|i| {
            (
                i.time,
                i.pointer.primary_pressed(),
                i.pointer.any_released(),
                i.pointer.primary_down(),
                i.pointer.interact_pos(),
                touch,
                !i.focused || i.multi_touch().is_some() || i.events.iter().any(|e| matches!(e, egui::Event::Touch { phase: egui::TouchPhase::Cancel, .. })),
                i.events.iter().any(|e| matches!(e, egui::Event::MouseWheel { .. })),
            )
        });
        let mut state = ctx.data(|d| d.get_temp::<Self>(key)).unwrap_or(Self {
            key,
            document,
            viewport: Rect::NOTHING,
            max_offset: 0.0,
            offset: 0.0,
            velocity: 0.0,
            at: now,
            finger: None,
            caught_fling: false,
        });
        state.offset = egui::scroll_area::State::load(ctx, scroll_id).map_or(0.0, |s| s.offset.y);
        let dt = (now - state.at).clamp(0.0, 0.05) as f32;
        if switched || document != state.document || now - state.at > 0.5 || cancel || wheel || pressed && !touch {
            state.velocity = 0.0;
            state.finger = None;
            state.caught_fling = false;
        }
        state.document = document;
        state.at = now;
        if !down && !released {
            state.caught_fling = false;
        }
        if pressed && !cancel {
            let was_moving = state.velocity.abs() >= 20.0;
            state.velocity = 0.0;
            state.caught_fling = false;
            // Leave the scrollbar to egui. The eye column, thumbnails, names, effects
            // and unused list space all accept a vertical finger scroll.
            if let Some(p) = pos.filter(|p| touch && ui.rect_contains_pointer(state.viewport) && p.x < state.viewport.right() - 10.0) {
                state.caught_fling = was_moving;
                state.finger = Some(Finger {
                    origin: p,
                    last: p,
                    mode: if was_moving { TouchLayerMode::Scrolling } else { TouchLayerMode::Pending },
                    sampled_at: now,
                    moved_at: now,
                });
            }
        }
        if !cancel && let Some(finger) = &mut state.finger {
            if let Some(p) = pos {
                // A row's held/dragging state takes precedence. Once scrolling is
                // chosen, returning towards the press point never turns into a hold.
                if finger.mode == TouchLayerMode::Pending {
                    finger.mode = row_mode.filter(|_| !pressed).unwrap_or_else(|| touch_layer_mode(p - finger.origin, 0.0));
                } else if !pressed && matches!(row_mode, Some(TouchLayerMode::Held | TouchLayerMode::Dragging)) {
                    finger.mode = row_mode.unwrap_or(TouchLayerMode::Held);
                }
                if finger.mode == TouchLayerMode::Scrolling {
                    let delta = p.y - finger.last.y;
                    state.offset -= delta;
                    if delta.abs() > 0.01 {
                        let sample_dt = (now - finger.sampled_at).max(1.0 / 240.0) as f32;
                        let speed = (-delta / sample_dt).clamp(-2400.0, 2400.0);
                        state.velocity = state.velocity * 0.35 + speed * 0.65;
                        finger.moved_at = now;
                    } else if now - finger.moved_at > 0.08 {
                        state.velocity = 0.0;
                    }
                    finger.last = p;
                    finger.sampled_at = now;
                } else {
                    state.velocity = 0.0;
                }
            }
            if !down {
                if !released || finger.mode != TouchLayerMode::Scrolling || now - finger.moved_at > 0.10 {
                    state.velocity = 0.0;
                }
                state.finger = None;
            }
        } else if !down && !cancel {
            // Exponential decay integrated over the frame: equal travel at 60/120Hz.
            let decay = (-7.0 * dt).exp();
            state.offset += state.velocity * (1.0 - decay) / 7.0;
            state.velocity *= decay;
            if state.velocity.abs() < 20.0 {
                state.velocity = 0.0;
            }
        }
        state.offset = state.offset.clamp(0.0, state.max_offset);
        if state.offset <= 0.0 && state.velocity < 0.0 || state.offset >= state.max_offset && state.velocity > 0.0 {
            state.velocity = 0.0;
        }
        state
    }

    pub fn finish<R>(mut self, ctx: &egui::Context, output: &egui::scroll_area::ScrollAreaOutput<R>) {
        self.viewport = output.inner_rect;
        self.max_offset = (output.content_size.y - output.inner_rect.height()).max(0.0);
        self.offset = output.state.offset.y;
        if self.finger.is_none() && self.velocity != 0.0 {
            ctx.request_repaint();
        }
        ctx.data_mut(|d| d.insert_temp(self.key, self));
    }
}
