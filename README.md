# PhotoCraft · Android 移植版

基于 [storytold/photocraft](https://github.com/storytold/photocraft) 的开源原生图像编辑器，使用 Rust、egui/eframe 和 wgpu，支持图层、蒙版、调整图层、笔刷、文字、矢量、滤镜及 PSD 文件。

本仓库当前发布版本为 **0.14.0-android.1（14001）**，基于上游 v0.14.0、提交 `ff4738810134d7f7e68b49f2e4f231b686282ff3`。这是独立维护的修改版本，并非上游官方 Android 发行版。README 的功能介绍与结构参考上游项目。

![PhotoCraft 桌面界面示例](docs/images/photocraft-demo.jpg)

*上图为上游桌面界面示例；Android 版本沿用编辑器，并适配横屏触摸操作。*

## 静态官网

官网位于 [`website/`](website/README.md)，包含产品首页、3D 分层动效、明暗主题及独立下载页。使用原生 HTML/CSS/JS，将 `website/` 内的文件上传到虚拟主机即可运行，无需构建或后端。可直接打开 `website/index.html` 预览，部署说明见 [官网 README](website/README.md)。

## 下载与安装

前往 [Releases](https://github.com/SnowOwlLeader/photocraft-for-android/releases/tag/v0.14.0-android.1) 下载最新安装文件：

| 文件 | 适用设备 |
| --- | --- |
| [PhotoCraft-0.14.0-android.1-14001-arm64-v8a.apk](https://github.com/SnowOwlLeader/photocraft-for-android/releases/download/v0.14.0-android.1/PhotoCraft-0.14.0-android.1-14001-arm64-v8a.apk) | Android 9 及以上的 ARM64 手机、平板 |
| [PhotoCraft-0.14.0-android.1-14001-x86_64.apk](https://github.com/SnowOwlLeader/photocraft-for-android/releases/download/v0.14.0-android.1/PhotoCraft-0.14.0-android.1-14001-x86_64.apk) | x86_64 Android 模拟器 |

下载对应 APK 后，在文件管理器中打开，按系统提示允许安装，再启动 PhotoCraft。设备需要兼容的 Vulkan / OpenGL ES 3 GPU。两个包是同一应用的不同架构版本，普通 ARM64 手机只需下载第一个。

本地 APK 与源码输出在 `dist/android/`；本次构建已发布至 GitHub Releases。本版通过上游 0.12.0 基线、现有安卓定制与 v0.14.0 三方合并同步更新：保留原 Rust 引擎 / egui / wgpu 架构和安卓 SAF 文件服务，接入上游新版文字、图层、PSD、滤镜及依赖修复。像素铅笔改为左侧画笔下方独立铅笔图标，点击启用 1px 无抗锯齿绘制，悬浮窗可调 1～64px 与自动擦除。保留原生文件管理器、PSD 覆盖保存、左滑多选、长按菜单、跨文档复制、双指画布操作、字体上传、QQ 社群和 UI 大小 1～10 档设置。

## 主要功能

- 图层、分组、蒙版、混合模式、调整图层和图层样式。
- 笔刷、选区、裁剪、变换、文字、矢量与常用图像滤镜。
- PSD / PSB 和原生 `.pcraft` 文档，以及 PNG、JPEG 等图像格式；具体兼容程度见[格式文档](docs/file-format-parity.md)。
- 原生 Rust 编辑引擎与 wgpu 渲染；桌面、命令行、WebAssembly 入口保留在源码中。
- 上游提供命令注册表、CLI、JSON 控制通道和 MCP 自动化能力，见[控制协议](docs/control-protocol.md)。

项目仍处于早期阶段，功能支持不代表 Photoshop 完全兼容。更多实现与差距见 [ROADMAP](ROADMAP.md) 和[已知差距](docs/gaps.md)。

## Android 适配

- 内嵌文件管理器负责打开和另存为；首次访问手机文件夹点击“添加手机目录”授权，保留系统选择器备用。`Save As` 可通过 `.png`、`.psd`、`.pcraft` 等后缀选择格式。
- 左侧画笔下方的铅笔图标启用 1px 像素铅笔，可在悬浮工具窗调整整数像素大小与自动擦除。
- 顶部提供 New、Open、Save、Save As、Undo、Redo、Fit，以及“图层”“选项”入口。
- 双指平移、缩放、旋转画布视图，图层列表支持触摸滚动和点选。
- 图层行左滑加入多选，长按后拖到其他已打开的文档标签可复制选中图层。
- 窄屏使用可横滑菜单和右下图层悬浮窗；欢迎页与新建文档窗口适配小屏。
- 文字工具提供系统输入框、浮动格式工具栏和 TTF / OTF / TTC 字体导入。
- 移动工具默认自动选层并显示变换框，滤镜菜单与滤镜库适配触屏。
- 工具组轻点即可展开，菜单支持大触摸行、窄屏布局与点按选择；滚动工具栏不会误展开。
- 外接鼠标按钮、滚轮与键盘修饰键接入原有快捷键，详见[安卓键鼠说明](docs/android-keyboard-mouse.md)。
- 欢迎页提供 QQ 社群入口：1129505510。

编辑分层作品建议保留 `.pcraft` 或 `.psd`，PNG/JPEG 为合成图像。完整构建与平台说明见 [Android README](packaging/android/README.md)。

## 已知限制与验证范围

界面仍采用桌面编辑器布局，平板或外接键鼠更适合复杂操作。暂不支持原生图片剪贴板、屏幕吸色、手写笔压力/倾斜；当前 APK 未启用 HEIC/HEIF。一次最多打开 16 个文件，单文件上限 256 MiB，大图仍受设备内存限制。

已有本地验证记录覆盖 Android 15 x86_64 模拟器的核心流程；尚未在真实 ARM64 手机上验证。此发布直接提供已有 APK，不表示已经完成所有设备和高级功能测试。

本版现有验证中，两项 RTX 5060 Ti / Vulkan 主机 GPU 计时比例测试仍失败；其余验证范围与限制见 [v0.14.0 同步记录](docs/android-upstream-0.14.0.md)。

## 从源码构建

桌面开发使用 Rust / Cargo，详见[开发文档](docs/development.md)：

```sh
cargo run -p photocraft
```

Android 构建脚本使用 Windows PowerShell、Rust、Android SDK/NDK 和 JDK 21。先按 [Android 构建说明](packaging/android/README.md) 配置工具路径，再运行：

**Want a specific build or format?** On GitHub, the [latest release](https://github.com/storytold/photocraft/releases/latest) has every build listed below, and [all releases](https://github.com/storytold/photocraft/releases) have earlier versions and their notes. `<ver>` in the file names is the version number, and `SHA256SUMS.txt` lists a checksum for every file.

### Windows

| Build | Installer | Portable |
|---|---|---|
| x64 (64-bit Intel/AMD) | `photocraft-<ver>-windows-x64.msi` | `photocraft-<ver>-windows-x64-portable.zip` |
| arm64 (Snapdragon and other ARM PCs) | `photocraft-<ver>-windows-arm64.msi` | `photocraft-<ver>-windows-arm64-portable.zip` |
| x86 (32-bit) | `photocraft-<ver>-windows-x86.msi` | `photocraft-<ver>-windows-x86-portable.zip` |

Installers and executables are code-signed.

### macOS

| Build | File | Notes |
|---|---|---|
| App, universal (Apple silicon + Intel) | `photocraft-<ver>-macos-universal.dmg` | Signed and notarized |
| Command-line tool, universal | `photocraft-cli-<ver>-macos-universal.zip` | Signed and notarized |

### Linux

| Format | x86_64 | aarch64 (ARM64) | Notes |
|---|---|---|---|
| AppImage | `photocraft-<ver>-linux-x86_64.AppImage` | `photocraft-<ver>-linux-aarch64.AppImage` | Runs anywhere; updates itself with [AppImageUpdate](https://github.com/AppImageCommunity/AppImageUpdate) (`.zsync` files) |
| Flatpak | `photocraft-<ver>-linux-x86_64.flatpak` | `photocraft-<ver>-linux-aarch64.flatpak` | Sandboxed; `flatpak install --user <file>` |
| Debian/Ubuntu | `photocraft-<ver>-linux-x86_64.deb` | `photocraft-<ver>-linux-aarch64.deb` | |
| Fedora/RHEL/openSUSE | `photocraft-<ver>-linux-x86_64.rpm` | `photocraft-<ver>-linux-aarch64.rpm` | |
| Tarball | `photocraft-<ver>-linux-x86_64.tar.gz` | `photocraft-<ver>-linux-aarch64.tar.gz` | Unpack anywhere |

RISC-V (riscv64): `photocraft-<ver>-linux-riscv64.tar.gz` only, built against Ubuntu 24.04 (needs glibc 2.39+).

### FreeBSD

| Build | File |
|---|---|
| x86_64 | `photocraft-<ver>-freebsd-x86_64.tar.gz` |

### Web (WebAssembly)

| Build | File | Notes |
|---|---|---|
| Static site | `photocraft-web-<ver>.zip` | Runs in a modern browser; host it on any static server |

## The Crafting Apps

PhotoCraft is one of the **Crafting Apps**: free, open-source creative tools from the
[ArtCraft](https://getartcraft.com/) team, each written from scratch in Rust and each able to
stand on its own.

| | App | What it's for | Code | Learn more |
|:-:|---|---|---|---|
| <img src="https://raw.githubusercontent.com/storytold/photocraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.photocraft.png" alt="" width="32" height="32"> | **PhotoCraft** | **Image editing: layers, masks, type and real PSD files · you are here** | [GitHub](https://github.com/storytold/photocraft) | [Website](https://getartcraft.com/apps/photocraft) |
| <img src="https://raw.githubusercontent.com/storytold/vectorcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.vectorcraft.png" alt="" width="32" height="32"> | **VectorCraft** | Vector illustration | [GitHub](https://github.com/storytold/vectorcraft) | [Website](https://getartcraft.com/apps/vectorcraft) |
| <img src="https://raw.githubusercontent.com/storytold/filmcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.filmcraft.png" alt="" width="32" height="32"> | **FilmCraft** | Video editing, color and sound | [GitHub](https://github.com/storytold/filmcraft) | [Website](https://getartcraft.com/apps/filmcraft) |
| <img src="https://raw.githubusercontent.com/storytold/lightcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.lightcraft.png" alt="" width="32" height="32"> | **LightCraft** | Photo library and raw development | [GitHub](https://github.com/storytold/lightcraft) | [Website](https://getartcraft.com/apps/lightcraft) |
| <img src="https://raw.githubusercontent.com/storytold/pdfcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.pdfcraft.png" alt="" width="32" height="32"> | **PdfCraft** | Reading, organizing and protecting PDFs | [GitHub](https://github.com/storytold/pdfcraft) | [Website](https://getartcraft.com/apps/pdfcraft) |
| <img src="https://raw.githubusercontent.com/storytold/effectcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.effectcraft.png" alt="" width="32" height="32"> | **EffectCraft** | Motion graphics and visual effects | [GitHub](https://github.com/storytold/effectcraft) | [Website](https://getartcraft.com/apps/effectcraft) |
| <img src="https://raw.githubusercontent.com/storytold/designcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.designcraft.png" alt="" width="32" height="32"> | **DesignCraft** | Page layout and publishing | [GitHub](https://github.com/storytold/designcraft) | [Website](https://getartcraft.com/apps/designcraft) |

And [**ArtCraft**](https://getartcraft.com/) itself, our AI image and video studio for artists who want real control.

The Crafting Apps share the same conventions: clean-room and pure Rust, native on macOS, Windows and Linux, in the browser via WebAssembly, and fully drivable by agents.

<br>

<p align="center">
  <a href="https://discord.gg/artcraft"><img alt="Join the ArtCraft community on Discord" src="https://img.shields.io/badge/Join%20us%20on%20Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white" height="40"></a>
</p>

<h3 align="center">Come make things with us</h3>

<p align="center">
  Our Discord is where artists of every kind hang out: people who paint, shoot, draw, cut film,
  set type, and people still figuring out what they like to make. Share what you're working on,
  ask for help, tell us what's broken, or tell us what you wish these tools could do.
  Whatever your medium and however long you've been at it, you're welcome here.
</p>

<p align="center">
  <a href="https://discord.gg/artcraft"><b>discord.gg/artcraft</b></a> ·
  <a href="https://getartcraft.com/">getartcraft.com</a> ·
  <a href="https://getartcraft.com/apps">The Crafting Apps</a> ·
  <a href="https://getartcraft.com/apps/photocraft">PhotoCraft</a>
</p>

---

APK 输出到 `dist/android/`。签名私钥不随源码发布；自己构建时使用自己的签名密钥。

## 项目结构

| 目录 | 内容 |
| --- | --- |
| `crates/` | 文档、图像处理、PSD/格式、渲染、编辑引擎与 egui UI |
| `apps/photocraft-android/` | Android Rust 入口 |
| `packaging/android/` | Android Activity、Manifest 与构建脚本 |
| `apps/photocraft/` | 桌面应用 |
| `apps/photocraft-cli/` | 命令行工具 |
| `apps/photocraft-web/` | WebAssembly 应用 |
| `docs/` | 架构、开发、功能与兼容性文档 |

## 许可与致谢

PhotoCraft 源码采用 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE) 双重许可。保留原作者 ArtCraft Team 与 PhotoCraft contributors 的版权声明，参见 [NOTICE](NOTICE)。第三方素材及其许可见 [ATTRIBUTION.md](ATTRIBUTION.md)，示例画作来源见 [SOURCES](docs/images/SOURCES.md)。

感谢 [storytold/photocraft](https://github.com/storytold/photocraft) 及其贡献者。本移植版不代表上游团队，也不隶属于 Adobe；Photoshop 等名称仅用于说明兼容的工作流程。

## 上游工具目录

<h4>🧰 67 tools</h4>
Move · Artboard · Rectangular and Elliptical Marquee · Single Row Marquee · Single Column Marquee · Lasso · Polygonal Lasso · Magnetic Lasso · Object Selection · Quick Selection · Magic Wand · Crop · Perspective Crop · Slice · Slice Select · Frame · Eyedropper · Color Sampler · Ruler · Note · Count · Remove · Spot Healing Brush · Healing Brush · Patch · Content-Aware Move · Red Eye · Brush · Pencil · Mixer Brush · Color Replacement · Clone Stamp · Pattern Stamp · History Brush · Eraser · Background Eraser · Magic Eraser · Gradient · Paint Bucket · Blur · Sharpen · Smudge · Dodge · Burn · Sponge · Pen · Freeform Pen · Curvature Pen · Add Anchor Point · Delete Anchor Point · Convert Point · Horizontal and Vertical Type · Horizontal and Vertical Type Mask · Path Selection · Direct Selection · Rectangle · Ellipse · Triangle · Polygon · Line · Custom Shape · Hand · Rotate View · Zoom

安卓定制版 8210：图层长按拖动排序、QQ 社群 1129505510、原生外接键鼠输入。详见 [安卓键鼠操作](docs/android-keyboard-mouse.md) 和 [Android 构建](packaging/android/README.md)。
