# PhotoCraft Android port

An experimental native Android port of PhotoCraft 0.7.0, based on upstream commit
`c436aa7294671c608f274895f6c8ec334dce86bd`. It uses the existing Rust engine,
egui/eframe UI and wgpu renderer. There is no web server or WebView.

## Install and use

The phone build is `dist/android/PhotoCraft-0.7.0-arm64-v8a.apk`. It requires an
ARM64 device, Android 9/API 28 or newer, and a compatible Vulkan/OpenGL ES 3 GPU.
It is a locally signed build, not an official upstream release.

Open the APK on the phone and allow installation from that file manager when
Android asks. The editor opens in landscape. The top action bar provides New,
Open, Save, Save As, Undo, Redo and Fit. Layers toggles the side dock; Options
toggles tool settings. Original menus and tools remain available. Use the
system document picker to open images and select where to save. PSD and
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
document provider. Uninstalling the app deletes its private staging and recovery.

## Limits

- This is a desktop-style editor adapted for landscape touch screens. A tablet
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
welcome CTA opens QQ group 378525932, with a copy-number fallback when QQ cannot
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
