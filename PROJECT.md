# Glance 项目现状

> 本文件是项目的**唯一事实来源**，取代已删除的 `HANDOFF.md`。
> 文档分工：[README.md](README.md) 对外介绍 / 本文件 = 现状与架构 / [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) 构建·打包·发布。
>
> 最后核对：2026-10-17 · 基准 commit [`654804a`](https://github.com/SyntaxEntity/Glance/commit/654804a)（`master`）

---

## 1. 一句话定位

**Glance** —— 低饱和护眼的 **Markdown + LaTeX** 桌面阅读器，面向 GNOME / Linux。MIT 许可，Rust 原生 + WebKitGTK 渲染，公式字体本地打包，**完全离线**。

| 项 | 值 |
|------|------|
| 仓库 | https://github.com/SyntaxEntity/Glance（public） |
| 远程 URL | SSH：`git@<host>:<owner>/<repo>.git`（原 `ssh://…:443` 形式已于 2026-10-17 弃用） |
| 当前版本 | `0.1.2` |
| 分支 | `master` |
| 语言 / 工具链 | Rust 2021 |
| 许可证 | MIT |

---

## 2. 功能现状

| 能力 | 状态 | 实现位置 |
|------|------|----------|
| Markdown 渲染（表格 / 代码块 / 引用 / 图片） | ✅ | WebView 内 marked.js |
| LaTeX 公式（行内 `$…$` / 块级 `$$…$$`） | ✅ | KaTeX + 本地字体 |
| 源码编辑（语法高亮，`Ctrl+S`） | ✅ | GtkSourceView 5，默认关闭 |
| 工作目录文件树（只列目录 + `.md`） | ✅ | `sidebar.rs` |
| 文档大纲（标题 → TOC，点击跳转） | ✅ | `outline.rs` + 锚点注入 |
| 护眼主题（GTK 与 WebView 一致） | ✅ | `resources/glance.css`、`preview/theme.css` |
| **导出 PDF** | ✅ | WebKit `PrintOperation`（打印到文件） |
| 导出 HTML | ❌ 未做 | — |
| 多标签页 / 全文搜索 / 主题切换 | ❌ 未做 | 见 §7 |

---

## 3. 代码结构

单 crate 二进制，约 1590 行 Rust（含测试）：

```
src/
  main.rs      30   入口，adw::Application（HANDLES_OPEN）
  window.rs   506   主窗口 / HeaderBar / 交互 / PDF 导出 —— 最大模块
  sidebar.rs  340   文件树 + 大纲，Stack 切换
  editor.rs   234   GtkSourceView 编辑器 + 保存回执 + 单元测试
  outline.rs  204   标题 → 大纲解析 + 单元测试
  viewer.rs   181   WebKitGTK 预览与 JS 桥
  config.rs    90   配置持久化 + 单元测试

resources/
  glance.css        GTK 护眼主题
  preview/          WebView 模板：index.html / render.js / marked.min.js
                    / katex.min.js / auto-render.min.js / theme.css / fonts/
data/
  glance.desktop    桌面入口
  icons/hicolor/    SVG + 16/24/32/48/64/128/256/512 PNG
docs/               示例笔记 + 开发文档
packaging/          RPM / DEB 打包（见 docs/DEVELOPMENT.md）
```

**技术栈**：gtk4-rs `0.11`(`v4_22`) · libadwaita `0.9`(`v1_9`) · webkit6 `0.6`(`v2_52`) · sourceview5 `0.11`(`v5_18`) · serde / serde_json。

---

## 4. 架构要点

### 4.1 渲染链

```
Markdown 文本
  ├─→ Rust: outline::extract_headings()   解析 ATX 标题 → Vec<Heading>
  │                                       生成 slug 锚点（重名自动 -1/-2）
  └─→ viewer::render_markdown(text, outline_json)
        └─→ 序列化成 JS 字面量，evaluate_javascript 注入
              └─→ render.js: marked.parse() → KaTeX renderMathInElement()
                    └─→ 按 Rust 侧的大纲给 <h1..h6> 打 id，供跳转
```

**关键设计**：锚点由 **Rust 侧**决定而非 JS 侧，因为侧栏大纲与预览必须用同一套 id 才能跳转。`render.js` 里保留了一份 `slugify` 只是兜底（当 `outline[i]` 缺失时）。

### 4.2 其它要点

- **资源解析顺序**（装包后仍可运行）：系统路径 `/usr/share/glance/…` → 二进制相对路径 → 编译期 `CARGO_MANIFEST_DIR` → 当前工作目录。见 `viewer.rs::resolve_preview_dir`、`window.rs::css_path`。
- **WebView 注入时序**：页面未 `LoadEvent::Finished` 前，内容先存进 `window.__pending_md`，加载完成后由 `flushPending()` 补渲染 —— 避免竞态丢内容。
- **编辑器**：`Editor` 通过 `on_saved` / `on_save_error` 两个回调向上汇报，不直接依赖窗口，便于单测。
- **配置**：`~/.config/glance/config.json`，存 `workdir` / `last_file` / `edit_mode` / `sidebar_visible`；损坏或缺失时回退默认值，不 panic。

---

## 5. 构建与运行

```bash
cargo build --release
./target/release/glance [file.md]

# 或经封装脚本（自动挑 release/debug 二进制）
./run.sh docs/welcome.md
```

**构建依赖（Fedora / RHEL 系）**：

```bash
sudo dnf install gtk4-devel libadwaita-devel webkitgtk6.0-devel \
  gtksourceview5-devel gobject-introspection-devel glib2-devel \
  cairo-devel pango-devel gdk-pixbuf2-devel graphene-devel
```

**运行依赖**：GTK 4 / libadwaita / WebKitGTK 6.0 / GtkSourceView 5，GNOME 桌面通常自带。

> ⚠️ 依赖版本偏新（GTK 4.22 / libadwaita 1.9 / WebKitGTK 6.0 / GtkSourceView 5.18）。Debian / Ubuntu 稳定版可能过旧，装包前先确认。

---

## 6. 测试

```bash
cargo test
```

16 项测试，全部无需网络（`config` / `outline` 纯逻辑，`editor` 需 GTK 显示）：

| 测试组 | 数量 | 覆盖 |
|------|------|------|
| `outline::tests` | 10 | 标题层级、代码围栏（``` 与 ~~~）、`#NoSpace` 不误判、超 6 级、闭合 `#` 剥离、中文 slug、标点折叠与 80 字符上限、行号、同名去重、空标题跳过 |
| `editor::tests` | 2 | 无文件保存必须回报错误（回归）；保存成功路径、落盘内容、dirty 复位 |
| `config::tests` | 4 | JSON 往返、损坏 JSON 回退默认、**缺字段容忍**、路径落在用户配置目录 |

**编辑器测试需要 GTK 显示**。无头环境用 Broadway 后端：

```bash
broadwayd -p 8085 :5 &
GDK_BACKEND=broadway BROADWAY_DISPLAY=:5 cargo test
```

（若 GTK 初始化失败，这两个测试会打印 `skipping` 并返回，不会误报失败。）

---

## 7. 路线图

- [ ] 多标签页
- [ ] 全文搜索
- [ ] 自定义主题切换
- [x] 导出 PDF（右上角打印按钮）
- [ ] 导出 HTML

**已知可改进点**（未排期）：

- 编辑模式的欢迎页可编辑但无文件绑定，保存会提示"没有打开的文件"；开文件后正常。
- 关闭窗口不检查未保存改动（`Editor::is_dirty()` 已就绪但无 UI 消费）。
- 无第三方依赖的依赖更新（Dependabot）配置。

---

## 8. 开发方式

本项目采用 **Vibe Coding**（AI 结对编程）：人类给出需求与验收标准，AI agent 完成编码、调试、打包与文档整理。

- 开发工具：DeepSeek Harness（当前维护）· [MiMoCode](https://mimo.xiaomi.com/mimocode)（v0.1.x 阶段）
- 协作方式：自然语言描述 → AI 实现并自检（`cargo check` / `cargo test` / 打包验证）→ 人工审阅与提交
- 对外披露：`README.md` 末尾含标准 DeepSeek 披露卡片

请以审阅辅助代码的心态使用：关键逻辑（Markdown / LaTeX 渲染、文件树、大纲）建议人工复核后再用于生产。

---

## 9. 交接须知（给下一个 agent）

- **远程操作可能失败**：本机 `git` 走 SSH 当前不可用（`/etc/ssh/ssh_config.d/` 属主为 `nobody:nobody`，需 root 修复）。**不要自行发明绕行方案**，按工作区规范上报。（2026-10-17 实测）
- **版本号三处同步**：`Cargo.toml`、`packaging/glance.spec`（`Version` + `%changelog`）、DEB 版本由脚本从 `Cargo.toml` 读取。`%changelog` 的**星期几必须与日期自洽**，否则 rpmbuild 告警。
- **提交身份**：author **与** committer 都必须是 `SyntaxEntity <265272538+SyntaxEntity@users.noreply.github.com>`，否则推送被 GH007 拒绝（`git config user.email` 已配好）。
- **无 CI**：`.github/` 不存在，四包目前靠本机脚本产出。
- **构建中间物**均在 `.gitignore` 内：`packaging/rpmbuild/`、`packaging/deb/`、`packaging/arm-rpm/`。
