# PhotoCraft Android port

An experimental native Android port of PhotoCraft 0.8.2, based on upstream commit
`d43db3e153c76aefce9b32cef9099e458eaf17a8`. It uses the existing Rust engine,
egui/eframe UI and wgpu renderer. There is no web server or WebView.

## Install and use

The phone build is `dist/android/PhotoCraft-0.8.2-android.11-8211-arm64-v8a.apk`. It requires an
ARM64 device, Android 9/API 28 or newer, and a compatible Vulkan/OpenGL ES 3 GPU.
It is a locally signed build, not an official upstream release.

Open the APK on the phone and allow installation from that file manager when
Android asks. The editor supports portrait and landscape. The scrollable top action bar provides New,
Open, Save, Save As, Undo, Redo and Fit. Layers toggles the side dock or, on a
narrow phone, a floating panel; Options toggles tool settings. The full menu bar
scrolls horizontally on narrow screens. Use the
embedded native file manager to open images and select where to save. Authorize
a phone folder with Add Phone Folder; a system picker remains available. PSD and
PhotoCraft documents retain layers according to the upstream format support.
Choose an extension such as `.png`, `.psd` or `.pcraft` in Save As to select the
output format. PNG/JPEG export is also available through the original File menu.
Upstream Save writes layered formats in place; for flat PNG/JPEG it opens Save As
again to avoid losing layers. The toolbar can be scrolled to reach other tools.

Preferences, brush presets and recovery snapshots live in app-private storage.
Saves are first written atomically to app-private staging, then copied to the
user-selected document provider. Provider write failures are reported to the
editor and do not count as a successful save. The staging file remains available
for retry. Atomic replacement of the external destination depends on its
document provider. Uninstalling the app deletes its private Works, staging and recovery.

## Limits

- This is a desktop-style editor adapted for touch screens. A tablet
  or a mouse/keyboard is more comfortable for dense dialogs and advanced tools.
- The Android picker limits each imported file to 256 MiB and each selection to
  16 files. Large decoded images can still exceed a phone's available memory.
- Native clipboard images, screen color sampling and stylus pressure/tilt have
  not been implemented in this port. Touch drawing and the app's internal image
  clipboard use the upstream editor.
- HEIC/HEIF support is not enabled in this build. System fonts provide CJK UI
  fallback where `/system/fonts/NotoSansCJK-Regular.ttc` is available.
- The desktop accessibility backend is disabled only for Android NativeActivity.
- Devices and GPU drivers differ. The included validation report states the
  actual test environment; it does not imply testing on every physical phone.

## Rebuild on this Windows machine

All newly installed toolchains, SDKs, download archives, emulator data and build
caches are under **`E:\photocraft-build`**. The existing JDK at
`D:\java\jdk-21.0.11` is reused. No permanent PATH change is needed.

```powershell
cd 'D:\Codex 项目1\photocraft-main'
.\packaging\android\build.ps1
# Emulator build:
.\packaging\android\build.ps1 -Abi x86_64
# Repackage a compiled native library:
.\packaging\android\build.ps1 -SkipRust
```

Pinned local tools: Rust 1.95.0 (`x86_64-pc-windows-gnullvm` host), LLVM-MinGW
20261006, Android NDK 28.2.13676358, SDK platform 35, Build Tools 35.0.0, JDK 21.
The build script uses the checked-in Cargo lockfile and keeps unwinding enabled.
It packages Java platform glue with javac/d8/aapt2, aligns for 16 KiB pages, signs
with apksigner, and verifies the result. Android code and APK resources are in
this directory; the Rust entry point is `apps/photocraft-android`.

Signing material is generated once in `E:\photocraft-build\signing` and is not
included in the source archive. Retain that directory securely to sign upgrades
that can be installed over this build. Different signing keys require uninstalling
the previous installation first.

## Validation

The app has an opt-in on-device engine regression check:

```powershell
adb shell am force-stop ai.storyteller.photocraft.android
adb shell am start -n ai.storyteller.photocraft.android/ai.storyteller.photocraft.PhotoCraftActivity --ez self_test true
adb logcat -d -s PhotoCraft:I
```

Look for `PHOTOCRAFT_SELF_TEST {"passed":true,...}`. It draws a red stroke,
verifies PNG pixels and dimensions, checks Undo/Redo, round-trips layered PSD and
PhotoCraft files, and checks rejection of a malformed PNG. It does not substitute
for testing the system picker and UI, which are checked separately.

## Platform boundary

Android requires an Activity and a native entry point. The small Java class here
only handles lifecycle and the Android Storage Access Framework; all editing and
UI drawing remain in Rust. The Android crate has `unsafe_code = "deny"`, with
two narrowly scoped exceptions for its exported entry symbol and for acquiring
owned JNI handles from AndroidApp's live VM/Activity pointers. Each has a safety
comment; no core crate permits additional unsafe code. JNI calls use local
reference frames and attach the current thread before touching Java objects.


## Android touch update (0.7.0-android.2)

Two fingers pan, pinch and rotate the canvas around their midpoint. Navigation
owns the gesture until all fingers lift, cancelling the pending brush preview.
Touch layer rows scroll instead of reordering; taps select, and mouse dragging
still reorders. Android panel toggles are labeled 图层 and 选项. The Android
welcome CTA opens QQ group 1129505510, with a copy-number fallback when QQ cannot
be opened. Package version code 7002 retains the previous local signing key.

## Text and effects update (0.7.0-android.3 / 7003)

The native multiline text dialog feeds type.edit; a floating Rust toolbar formats
text through type.setStyle. TTF/OTF/TTC imports persist in private Fonts storage.
Android system font directories are scanned by the text engine. Move auto-select
and transform controls default on. Touch submenus expand inline; Filter Gallery
uses a compact layout. PSD long effect IDs normalize before parsing, with fallback
from malformed lmfx to lfx2/lfxs/lrFX and an import warning.
Device self-test also checks seven filters plus undo, three PSD styles and Unicode
text editing. Full PSD corpus oracle and mutation suites passed in the host test
profile; release corpus compilation hit toolchain metadata errors.

## Layer and narrow-screen update (0.7.0-android.4 / 7004)

Swipe left on individual rows in the Layers panel to add them to the selection;
vertical swipes scroll and taps select one layer. Hold a selected row, then drag
it to another open document tab to copy the selected layers to that document.
Quick Export as PNG uses Android's save picker and document-provider writer.
Portrait is enabled; narrow screens use a floating Layers panel and a
horizontally scrollable menu bar.
The welcome screen stacks vertically below 480 points; New Document uses a
scrollable vertical form below 600 points, keeping Create and Close visible.
Panel toggles retain visible Chinese labels while other actions scroll.

## Layer context-menu follow-up (0.7.0-android.6 / 7006)

Hold a layer thumbnail or name for about half a second to open the existing desktop
context menu. Release, then tap an action or tap outside to dismiss. Moving the held
finger dismisses the popup and continues the existing cross-document copy gesture.
Left-swipe additive selection and vertical scrolling remain available. Both APK ABIs
are built with version code 7006 and version name 0.7.0-android.6.

## Swipe selection follow-up (0.7.0-android.5 / 7005)

Android layer rows enable finger gestures even before egui has seen a Touch event.
They retain the release position when PointerGone accompanies a fast swipe, and
recognize a left displacement over 24 points. A Chinese selected-layer count is
shown above the panel controls; swipe from the thumbnail or name, not the eye.
Selected rows disable drag scrolling while held; ordinary vertical scrolling and
long-hold document transfer remain supported.

The first toolbar gear opens compact Chinese Interface Settings. Minus/plus
buttons select 1..10 UI size levels (60..150%, 10% per step); level 5 is the
unchanged 100% default. Changes apply live via the existing prefs.set command and
persist through the platform preferences service. The window has a default-reset
button and stays within the small screen as its scale changes.

## Save original PSD fix (0.7.0-android.7 / 7007)

Picked documents retain a named private mirror and its original document-provider
URI, including the granted persistent read/write permissions. The picker now
answers with Paths instead of anonymous ContentsMany, so the existing Rust
background-open job records the source identity. Save overwrites the original
PSD/PSB/PhotoCraft destination through commitFile without opening CREATE_DOCUMENT.
Save As still opens the system picker, and successful Save As becomes the target
of subsequent saves. Cancelled or failed writes keep the original document identity
and unsaved edits. A read-only provider or expired grant reports an error; it does
not silently create a copy. Newly created untitled documents choose a destination
on their first save. Font imports remain transient and do not register destinations.

Documents already opened in older versions have lost their original destination;
reopen the original PSD with 7007 to establish it. Android's provider streaming
overwrite cannot guarantee an atomic remote replacement, so retain backups of
important PSDs. The complete encoded private mirror is kept if a provider fails.

## Pixel pencil and embedded file manager (introduced in 7008; current entry in 8209)

Tap the independent pencil icon directly below Brush in the left toolbar to activate the existing
paint.pencil engine with a square, aliased 1px tip, full opacity/flow, no pressure
dynamics and no smoothing. Its floating panel offers integer sizes 1..64px,
quick presets and auto erase. Other sampled brush settings are reset on entry.

The toolbar folder button now opens a native egui file manager using the editor's
theme. It browses private Works by default, or phone directories granted through
Android's folder authorization dialog. Android 11+ restricts authorization of
the Download root; choose or create a subfolder instead. Granted directories and
the last directory persist. Revoked grants fall back to Works. No broad storage
permission or WebView is used; SAF provider operations run on background workers.

Tap files to toggle selection (up to 16), then Open Selected Files. Tap directories
to enter; Back returns. Search, refresh, new folder, per-entry rename/delete and
Save As are available. Existing-name saves and deletes ask for confirmation;
only empty directories can be deleted. Text fields invoke Android's keyboard.
The body scrolls independently of the footer on small screens. System Picker
remains a fallback. Save still overwrites the original PSD destination, including
files opened through this manager; Save As retargets subsequent saves.

Works is app-private and is deleted on uninstall. Save important work to an
authorized phone folder. Imports keep the existing 256 MiB per-file limit, and
directory listings are bounded to 4096 entries.

## Upstream v0.8.2 integration (8209)

Merged the fixed v0.8.2 source (`d43db3e153c76aefce9b32cef9099e458eaf17a8`)
against the original 0.7.0 base while retaining Android changes. No WebView,
replacement UI framework or storage permission expansion was introduced.
Android and desktop retain the shared command registry and editor. Android has
a separate Pencil slot; desktop keeps the upstream Brush flyout. The top bar
no longer has a Pixel button. Version of this upstream merge: 0.8.2-android.9 (8209), same local signer.

The upstream folder chooser uses this embedded browser. Features requiring
filesystem folder paths can select private Works directories; SAF phone folders
remain available for individual open/save and file management. Choosing a SAF
folder for path-based batch processing reports the limitation without closing
the browser. See validation/验证报告.md for checks and device scope.

## Android input update (8210)

Version: 0.8.2-android.10 / 8210. Touch layer ordering retains long-press menus,
vertical list scrolling and left-swipe selection. External mice emit independent
buttons and scroll axes; Android Ctrl/Alt/Shift state reaches the existing
Photoshop-style command bindings. All community entries use QQ 1129505510.

安卓键鼠及图层长按排序说明：[android-keyboard-mouse.md](../../docs/android-keyboard-mouse.md)。本地补丁 winit 的 Android 鼠标/修饰键输入，使用同版本 path patch，许可见 `vendor/winit/LICENSE`。

## Touch tool groups (8211)

Android grouped toolbar tools open on a tap, right-click or stationary long press.
Tap a menu row to select it; tap outside or press Escape to dismiss. Menu rows
use 36-point touch targets and fit narrow screens. Scrolling the toolbar does
not open a group. The standalone pixel Pencil remains below Brush.
