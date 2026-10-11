# PhotoCraft · Android 移植版

基于 [storytold/photocraft](https://github.com/storytold/photocraft) 的开源原生图像编辑器，使用 Rust、egui/eframe 和 wgpu，支持图层、蒙版、调整图层、笔刷、文字、矢量、滤镜及 PSD 文件。

本仓库发布实验性 Android 移植版 **0.7.0-android.3（7003）**，基于上游 0.7.0、提交 `c436aa7294671c608f274895f6c8ec334dce86bd`。这是独立维护的修改版本，并非上游官方 Android 发行版。README 的功能介绍与结构参考上游项目。

![PhotoCraft 桌面界面示例](docs/images/photocraft-demo.jpg)

*上图为上游桌面界面示例；Android 版本沿用编辑器，并适配横屏触摸操作。*

## 下载与安装

前往 [Releases](https://github.com/SnowOwlLeader/photocraft/releases) 下载：

| 文件 | 适用设备 |
| --- | --- |
| [PhotoCraft-0.7.0-arm64-v8a.apk](https://github.com/SnowOwlLeader/photocraft/releases/download/v0.7.0-android.3/PhotoCraft-0.7.0-arm64-v8a.apk) | Android 9 及以上的 ARM64 手机、平板 |
| [PhotoCraft-0.7.0-x86_64.apk](https://github.com/SnowOwlLeader/photocraft/releases/download/v0.7.0-android.3/PhotoCraft-0.7.0-x86_64.apk) | x86_64 Android 模拟器 |

下载对应 APK 后，在文件管理器中打开，按系统提示允许安装，再横屏启动 PhotoCraft。设备需要兼容的 Vulkan / OpenGL ES 3 GPU。两个包是同一应用的不同架构版本，普通 ARM64 手机只需下载第一个。

## 主要功能

- 图层、分组、蒙版、混合模式、调整图层和图层样式。
- 笔刷、选区、裁剪、变换、文字、矢量与常用图像滤镜。
- PSD / PSB 和原生 `.pcraft` 文档，以及 PNG、JPEG 等图像格式；具体兼容程度见[格式文档](docs/file-format-parity.md)。
- 原生 Rust 编辑引擎与 wgpu 渲染；桌面、命令行、WebAssembly 入口保留在源码中。
- 上游提供命令注册表、CLI、JSON 控制通道和 MCP 自动化能力，见[控制协议](docs/control-protocol.md)。

项目仍处于早期阶段，功能支持不代表 Photoshop 完全兼容。更多实现与差距见 [ROADMAP](ROADMAP.md) 和[已知差距](docs/gaps.md)。

## Android 适配

- 系统文件选择器负责打开和保存；`Save As` 可通过 `.png`、`.psd`、`.pcraft` 等后缀选择格式。
- 顶部提供 New、Open、Save、Save As、Undo、Redo、Fit，以及“图层”“选项”入口。
- 双指平移、缩放、旋转画布视图，图层列表支持触摸滚动和点选。
- 文字工具提供系统输入框、浮动格式工具栏和 TTF / OTF / TTC 字体导入。
- 移动工具默认自动选层并显示变换框，滤镜菜单与滤镜库适配触屏。
- 欢迎页提供 QQ 社群入口：378525932。

编辑分层作品建议保留 `.pcraft` 或 `.psd`，PNG/JPEG 为合成图像。完整构建与平台说明见 [Android README](packaging/android/README.md)。

## 已知限制与验证范围

界面仍采用桌面编辑器布局，平板或外接键鼠更适合复杂操作。暂不支持原生图片剪贴板、屏幕吸色、手写笔压力/倾斜；当前 APK 未启用 HEIC/HEIF。一次最多打开 16 个文件，单文件上限 256 MiB，大图仍受设备内存限制。

已有本地验证记录覆盖 Android 15 x86_64 模拟器的核心流程；尚未在真实 ARM64 手机上验证。此发布直接提供已有 APK，不表示已经完成所有设备和高级功能测试。

## 从源码构建

桌面开发使用 Rust / Cargo，详见[开发文档](docs/development.md)：

```sh
cargo run -p photocraft
```

Android 构建脚本使用 Windows PowerShell、Rust、Android SDK/NDK 和 JDK 21。先按 [Android 构建说明](packaging/android/README.md) 配置工具路径，再运行：

```powershell
.\packaging\android\build.ps1
.\packaging\android\build.ps1 -Abi x86_64
```

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
