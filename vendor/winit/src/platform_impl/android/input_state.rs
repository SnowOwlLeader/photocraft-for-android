//! Android NDK input constants, isolated so desktop tests cover the Android adapter.
//! Values are from android/input.h and android/keycodes.h (public Android SDK).
use crate::event::{ElementState, MouseButton};
use crate::keyboard::ModifiersState;

pub fn modifiers(meta: u32, key: Option<(u32, bool)>) -> ModifiersState {
    // Generic, left, right, left keycode, right keycode.
    let groups = [
        (1, 0x40, 0x80, 59, 60, ModifiersState::SHIFT),
        (2, 0x10, 0x20, 57, 58, ModifiersState::ALT),
        (0x1000, 0x2000, 0x4000, 113, 114, ModifiersState::CONTROL),
        (0x10000, 0x20000, 0x40000, 117, 118, ModifiersState::SUPER),
    ];
    let mut state = ModifiersState::empty();
    for (generic, left, right, lk, rk, flag) in groups {
        let mut bits = meta & (generic | left | right);
        if let Some((code, pressed)) = key {
            if code == lk || code == rk {
                if pressed {
                    bits |= generic;
                } else {
                    bits &= !(generic | if code == lk { left } else { right });
                }
            }
        }
        if bits != 0 {
            state.insert(flag);
        }
    }
    state
}

/// HoverExit precedes Down on Android and can already carry the new button state.
/// It only changes cursor presence; consuming that snapshot loses the following press.
pub fn mouse_button_snapshot(previous: u32, reported: u32, action: u32) -> u32 {
    match action {
        10 => previous,          // AMOTION_EVENT_ACTION_HOVER_EXIT
        3 => 0,                  // AMOTION_EVENT_ACTION_CANCEL
        0 if reported == 0 => 1, // Synthetic primary Down without button metadata.
        _ => reported,
    }
}

pub fn mouse_buttons(previous: u32, current: u32) -> Vec<(MouseButton, ElementState)> {
    [(1, MouseButton::Left), (2, MouseButton::Right), (4, MouseButton::Middle), (8, MouseButton::Back), (16, MouseButton::Forward)]
        .into_iter()
        .filter(|(mask, _)| (previous ^ current) & mask != 0)
        .map(|(mask, button)| (button, if current & mask == 0 { ElementState::Released } else { ElementState::Pressed }))
        .collect()
}

pub fn wheel(horizontal: f32, vertical: f32) -> Option<(f32, f32)> {
    (horizontal.is_finite() && vertical.is_finite() && (horizontal != 0.0 || vertical != 0.0))
        .then_some((horizontal.clamp(-1000.0, 1000.0), vertical.clamp(-1000.0, 1000.0)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ctrl_shift_alt_and_sided_release() {
        assert_eq!(modifiers(0x1000 | 1 | 2, None), ModifiersState::CONTROL | ModifiersState::SHIFT | ModifiersState::ALT);
        assert_eq!(modifiers(0, Some((113, true))), ModifiersState::CONTROL);
        assert_eq!(modifiers(0x1000 | 0x2000, Some((113, false))), ModifiersState::empty());
        assert_eq!(modifiers(0x1000 | 0x2000 | 0x4000, Some((113, false))), ModifiersState::CONTROL);
        assert_eq!(modifiers(0x20000, None), ModifiersState::SUPER);
    }
    #[test]
    fn mouse_buttons_deduplicate_android_down_and_button_press() {
        assert_eq!(mouse_buttons(0, 1), [(MouseButton::Left, ElementState::Pressed)]);
        assert!(mouse_buttons(1, 1).is_empty());
        assert_eq!(mouse_buttons(1, 3), [(MouseButton::Right, ElementState::Pressed)]);
        assert_eq!(mouse_buttons(7, 0).len(), 3);
        assert!(mouse_buttons(0, 0x8000).is_empty());
    }
    #[test]
    fn hover_exit_does_not_consume_the_next_mouse_press() {
        for button in [1, 2, 4] {
            let hover = mouse_button_snapshot(0, button, 10);
            assert!(mouse_buttons(0, hover).is_empty());
            let down = mouse_button_snapshot(hover, button, 0);
            assert_eq!(mouse_buttons(hover, down).len(), 1);
            assert!(mouse_buttons(down, mouse_button_snapshot(down, button, 11)).is_empty());
            assert_eq!(mouse_buttons(down, mouse_button_snapshot(down, 0, 1)).len(), 1);
        }
        assert_eq!(mouse_button_snapshot(1, 0, 10), 1);
        assert_eq!(mouse_button_snapshot(1, 1, 3), 0);
        assert_eq!(mouse_button_snapshot(0, 0, 0), 1);
    }
    #[test]
    fn wheel_preserves_axes_and_rejects_nonfinite_data() {
        assert_eq!(wheel(-1.0, 2.5), Some((-1.0, 2.5)));
        assert_eq!(wheel(0.0, 0.0), None);
        assert_eq!(wheel(f32::NAN, 1.0), None);
        assert_eq!(wheel(0.0, f32::INFINITY), None);
    }
}
