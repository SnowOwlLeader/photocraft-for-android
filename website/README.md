# PhotoCraft 静态官网

使用原生 HTML + CSS + JavaScript，包含首页、独立下载页、CSS 3D 分层画布、鼠标视差、滚动渐入、编辑器预览切换和明暗主题。没有 npm 依赖、构建步骤、数据库、API 密钥或后端服务。

## 本地预览

可以直接双击 `index.html`，推荐使用本地 HTTP 服务检查所有交互：

```powershell
cd website
python -m http.server 8080
```

浏览器访问 `http://localhost:8080/`。Python 只用于本地预览，正式虚拟主机不需要 Python。

## 上传到虚拟主机

1. 将**这个目录里面的内容**上传到网站根目录，例如 `public_html/`、`wwwroot/` 或 `htdocs/`。注意保留 `assets/` 目录层级。
2. 确保默认首页包含 `index.html`。如果根目录已有主机自带的 `index.php`，在主机控制台设置默认首页优先级。
3. 打开域名即可访问；下载页地址是 `https://你的域名/download.html`。

支持 Apache、Nginx、IIS、宝塔及静态网站托管，无需伪静态、路由重写或特殊响应头。所有站内路径均为相对路径，也可以上传到 `public_html/photocraft/` 这样的子目录。无需修改域名配置文件。浏览器应使用当前版本 Chrome、Edge、Firefox 或 Safari；3D 效果使用 CSS，无 WebGL/CDN 依赖。

```text
index.html             首页
download.html          下载与安装指南
assets/
  styles.css           响应式布局、主题、CSS 3D 与动效
  theme.js             首屏主题初始化，避免主题闪烁
  app.js               导航、视差、主题记忆、预览与架构选择
  mark.svg             原创网站标识
  landscape-*.svg      原创三层风景画布
  editor-*.webp        本地编辑器截图
  LICENSE.txt          素材来源与许可
LICENSE-MIT
LICENSE-APACHE
NOTICE
README.md
```

所有视觉素材均随网站一起打包，无在线字体、统计脚本或第三方运行时。APK 下载统一通过 `https://gh.b52m.cn/` 代理，地址格式为代理前缀加完整 GitHub 下载地址；项目与发布说明链接仍指向 GitHub。网站本身不会连接 GitHub API，也不会自动追踪或切换发布版本。

## 更新发布版本

当前固定到仓库 README 中的 **0.14.0-android.1 / 14001**，提供通过 `gh.b52m.cn` 代理的 GitHub Releases APK 地址。

发布新版本时，统一更新：

- `download.html`：默认下载地址、两个直接下载地址、版本说明地址、版本号、设备要求和更新说明。
- `assets/app.js`：`releaseBase`、下载文件名和展示版本号（搜索 `0.14.0-android.1`、`14001`）。
- `index.html`：顶部版本徽标、FAQ 中的版本说明。

如果需要将 APK 一并放在自己的虚拟主机，可新建 `downloads/` 目录上传文件，再将上述三个静态下载地址与 `app.js` 的动态下载地址改为对应的相对路径。检查主机支持 `.apk` 下载，并留足空间和带宽。

## 可访问性与降级

- 主题默认跟随系统，用户切换后通过 `localStorage` 记住偏好；存储被禁用时仍可切换。
- 系统开启「减少动态效果」后，自动关闭漂浮、旋转、视差和滚动渐入。
- 导航、图层展开、预览标签、架构选择及 FAQ 均支持键盘；预览标签支持左右方向键、Home 和 End。
- 关闭 JavaScript 时，所有正文、FAQ、站内链接和下载页的直接 APK 链接仍然可用。
- 截图为上游桌面编辑器界面，网站已明确标注，未冒充 Android 截图。原版绘画均属公有领域，详见 `assets/LICENSE.txt`。

## 浏览器回归检查（仅开发时使用）

在仓库根目录安装可选的测试工具并运行：

```powershell
python -m pip install playwright
python scripts/test_website.py --browser "C:\Program Files\Google\Chrome\Application\chrome.exe"
```

也可先执行 `python -m playwright install chromium`，再不带 `--browser` 运行。测试自动启动本地临时 HTTP 服务，覆盖主机根目录与子目录、主题存储、移动菜单、3D 交互、预览键盘导航、下载架构切换、文件直开、减少动效以及关闭 JavaScript 的降级。不会实际下载 APK。
