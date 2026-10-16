# Glance

低饱和护眼的 **Markdown + LaTeX** 阅读器，面向 GNOME / Linux 桌面。

![GTK4](https://img.shields.io/badge/GTK-4.22-3d3846?logo=gnome&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-2021-3d3846?logo=rust&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-6d8b74)
![DeepSeek Assisted Development](https://img.shields.io/badge/DeepSeek%20Assisted-Development-brightgreen?style=for-the-badge&logo=deepseek)

📖 **文档**：[PROJECT.md](PROJECT.md) 项目现状与架构 · [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) 构建·打包·发布

## 功能

- **Markdown 渲染**：标题、列表、表格、代码块、引用、图片
- **源码编辑**：GtkSourceView 语法高亮，默认关闭；`Ctrl+S` 保存
- **LaTeX 公式**：行内 `$...$` 与块级 `$$...$$`（KaTeX，离线字体）
- **工作目录文件树**：侧栏只显示目录结构与 `.md` 文件
- **文档大纲**：按标题生成 TOC，点击跳转到锚点
- **护眼主题**：暖纸色低饱和配色，GTK 与 WebView 一致

## 截图

预览模式下渲染 Markdown 与 LaTeX 公式，左侧为文件树 / 大纲。

## 依赖

### 构建（Fedora / RHEL 系）

```bash
sudo dnf install gtk4-devel libadwaita-devel webkitgtk6.0-devel \
  gtksourceview5-devel gobject-introspection-devel glib2-devel \
  cairo-devel pango-devel gdk-pixbuf2-devel graphene-devel
```

### 运行

- GTK 4 / libadwaita / WebKitGTK 6.0 / GtkSourceView 5（GNOME 桌面通常已自带）

## 构建与运行

```bash
cargo build --release
./target/release/glance [file.md]
# 或
./run.sh docs/welcome.md
```

首次使用建议：

1. 点左上角文件夹图标，选择工作目录（例如本仓库的 `docs/`）
2. 在侧栏「文件」中打开 `.md`
3. 需要修改时点右上角编辑按钮（默认预览）

## 快捷键

| 快捷键 | 作用 |
|--------|------|
| `Ctrl+S` | 保存当前文件 |
| 点击大纲项 | 跳转到对应标题 |
| 右上角铅笔 | 切换编辑 / 预览（默认预览） |

## LaTeX 示例

```markdown
行内：$E = mc^2$

块级：

$$
\int_{-\infty}^{\infty} e^{-x^2}\,\mathrm{d}x = \sqrt{\pi}
$$
```

## 项目结构

```
src/
  main.rs        入口
  window.rs      主窗口 / 交互
  sidebar.rs     文件树 + 大纲
  editor.rs      GtkSourceView 编辑器
  viewer.rs      WebKit 预览
  outline.rs     标题大纲解析
  config.rs      工作目录等配置
resources/
  glance.css     GTK 护眼主题
  preview/       WebView 模板（marked.js + KaTeX）
docs/            示例笔记
data/glance.desktop
data/icons/      应用图标（hicolor：SVG + 多尺寸 PNG）
```

## 技术栈

| 组件 | 用途 |
|------|------|
| Rust + gtk4-rs | 原生 GNOME UI |
| libadwaita | HIG 风格控件 |
| WebKitGTK 6 | Markdown / LaTeX 渲染 |
| GtkSourceView 5 | 源码编辑与高亮 |
| marked.js | Markdown → HTML |
| KaTeX | LaTeX 公式 |

## 路线图

- [ ] 多标签页
- [ ] 全文搜索
- [ ] 自定义主题切换
- [x] 导出 PDF（右上角打印按钮）
- [ ] 导出 HTML

## DeepSeek Assisted Development

![DeepSeek Assisted Development](https://img.shields.io/badge/DeepSeek%20Assisted-Development-brightgreen?style=for-the-badge&logo=deepseek)

本仓库代码由 **DeepSeek** 辅助编写（vibecoding）。AI 生成的代码可能包含缺陷、过时用法或不完整实现，
请自行评估后使用与集成。

### 开发方式

本项目采用 **Vibe Coding**（AI 结对编程）：人类给出需求与验收标准，AI agent 完成代码编写、调试、打包与文档整理。

- **开发工具**：DeepSeek Harness（当前维护）· [MiMoCode](https://mimo.xiaomi.com/mimocode)（v0.1.x 阶段）
- **协作方式**：自然语言描述需求与缺陷 → AI 实现并自检（`cargo check` / `cargo test` / 打包验证）→ 人工审阅与提交

关键逻辑（Markdown / LaTeX 渲染、文件树、大纲）仍建议人工复核后再用于生产环境。

## 许可证

MIT — 详见 [LICENSE](LICENSE)。

第三方组件（均为 MIT）：

- [marked.js](https://github.com/markedjs/marked)
- [KaTeX](https://github.com/KaTeX/KaTeX)（含字体）
