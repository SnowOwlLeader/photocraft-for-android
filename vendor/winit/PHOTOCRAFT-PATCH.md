# PhotoCraft Android input patch

Source: winit 0.30.13, https://crates.io/crates/winit/0.30.13 (Apache-2.0; LICENSE retained).
Changes are restricted to Android input events and host tests of the Android mappings.
Mouse source/tool detection emits CursorMoved, MouseInput (left/right/middle/back/forward),
MouseWheel and CursorEntered/Left instead of Touch; real fingers/styli retain Touch.
Keyboard and pointer meta state emit ModifiersChanged before key/button/wheel events.
Modifier-key releases handle pre-transition Android meta snapshots and sided keys.
Focus loss clears adapter state. Mouse DOWN/BUTTON_PRESS duplicates are deduplicated.
HOVER_EXIT keeps the previous button state: Android can attach the upcoming Down's
buttons to it, before egui has a cursor position for that press.
No JS, WebView, or replacement UI/event loop has been introduced.
