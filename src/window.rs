use crate::config::Config;
use crate::editor::Editor;
use crate::outline::{self, Heading};
use crate::sidebar::Sidebar;
use crate::viewer::Viewer;
use adw::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

struct WindowState {
    cfg: RefCell<Config>,
    current_file: RefCell<Option<PathBuf>>,
    edit_mode: RefCell<bool>,
    current_headings: RefCell<Vec<Heading>>,
}

struct App {
    window: adw::ApplicationWindow,
    sidebar: Rc<Sidebar>,
    viewer: Rc<Viewer>,
    editor: Rc<Editor>,
    content_stack: gtk::Stack,
    title_label: gtk::Label,
    subtitle_label: gtk::Label,
    save_btn: gtk::Button,
    sidebar_toggle: gtk::ToggleButton,
    state: Rc<WindowState>,
}

pub fn on_activate(app: &adw::Application) {
    open_window(app, None);
}

pub fn on_open(app: &adw::Application, files: &[gtk::gio::File], _hint: &str) {
    let first = files.first().and_then(|f| f.path()).map(PathBuf::from);
    open_window(app, first);
}

fn open_window(app: &adw::Application, initial: Option<PathBuf>) {
    let cfg = crate::config::load();

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Glance")
        .default_width(1180)
        .default_height(780)
        .width_request(880)
        .height_request(520)
        .build();
    window.set_icon_name(Some("glance"));

    let provider = gtk::CssProvider::new();
    provider.load_from_path(css_path());
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let header = adw::HeaderBar::new();
    header.set_show_title(false);

    let sidebar_btn = gtk::ToggleButton::new();
    sidebar_btn.set_icon_name("view-list-symbolic");
    sidebar_btn.set_tooltip_text(Some("显示/隐藏侧边栏"));
    sidebar_btn.set_active(cfg.sidebar_visible);
    header.pack_start(&sidebar_btn);

    let open_dir_btn = gtk::Button::from_icon_name("folder-open-symbolic");
    open_dir_btn.set_tooltip_text(Some("设置工作目录"));
    header.pack_start(&open_dir_btn);

    let open_file_btn = gtk::Button::from_icon_name("document-open-symbolic");
    open_file_btn.set_tooltip_text(Some("打开 Markdown 文件"));
    header.pack_start(&open_file_btn);

    let title_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    title_box.set_hexpand(true);
    title_box.set_halign(gtk::Align::Center);

    let title_label = gtk::Label::new(Some("Glance"));
    title_label.add_css_class("glance-title-label");
    title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let subtitle_label = gtk::Label::new(Some("Markdown · LaTeX 阅读器"));
    subtitle_label.add_css_class("glance-sub-label");
    subtitle_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title_box.append(&title_label);
    title_box.append(&subtitle_label);
    header.set_title_widget(Some(&title_box));

    let edit_btn = gtk::ToggleButton::new();
    edit_btn.set_icon_name("document-edit-symbolic");
    edit_btn.set_tooltip_text(Some("切换编辑/预览（默认预览）"));
    edit_btn.set_active(false);
    header.pack_end(&edit_btn);

    let save_btn = gtk::Button::from_icon_name("document-save-symbolic");
    save_btn.set_tooltip_text(Some("保存 (Ctrl+S)"));
    save_btn.add_css_class("suggested-action");
    save_btn.set_visible(false);
    header.pack_end(&save_btn);

    let export_btn = gtk::Button::from_icon_name("document-print-symbolic");
    export_btn.set_tooltip_text(Some("导出 PDF"));
    header.pack_end(&export_btn);

    let split = adw::OverlaySplitView::new();
    split.set_sidebar_position(gtk::PackType::Start);
    split.set_min_sidebar_width(200.0);
    split.set_max_sidebar_width(340.0);
    split.set_sidebar_width_fraction(0.22);
    split.set_collapsed(false);

    let sidebar = Sidebar::new();
    split.set_sidebar(Some(&sidebar.root));

    let viewer = Viewer::new();
    let editor = Editor::new();

    let content_stack = gtk::Stack::new();
    content_stack.set_hexpand(true);
    content_stack.set_vexpand(true);
    content_stack.add_named(&viewer.root, Some("preview"));
    content_stack.add_named(&editor.root, Some("edit"));
    content_stack.set_visible_child_name("preview");
    split.set_content(Some(&content_stack));

    let toast_overlay = adw::ToastOverlay::new();
    toast_overlay.set_child(Some(&split));

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.set_content(Some(&toast_overlay));
    window.set_content(Some(&toolbar_view));

    let state = Rc::new(WindowState {
        cfg: RefCell::new(cfg),
        current_file: RefCell::new(None),
        edit_mode: RefCell::new(false),
        current_headings: RefCell::new(Vec::new()),
    });

    let app_ui = Rc::new(App {
        window: window.clone(),
        sidebar: sidebar.clone(),
        viewer: viewer.clone(),
        editor: editor.clone(),
        content_stack: content_stack.clone(),
        title_label: title_label.clone(),
        subtitle_label: subtitle_label.clone(),
        save_btn: save_btn.clone(),
        sidebar_toggle: sidebar_btn.clone(),
        state: state.clone(),
    });

    // Clone out of RefCell first — temporaries in `if let` live for the whole block.
    {
        let wd = app_ui.state.cfg.borrow().workdir.clone();
        if let Some(wd) = wd {
            if wd.is_dir() {
                app_ui.sidebar.set_workdir(Some(wd));
            }
        }
    }

    *app_ui.state.edit_mode.borrow_mut() = false;
    app_ui.content_stack.set_visible_child_name("preview");
    {
        let visible = app_ui.state.cfg.borrow().sidebar_visible;
        split.set_show_sidebar(visible);
    }

    {
        let ui = Rc::clone(&app_ui);
        sidebar.set_on_open_file(move |path| {
            ui.open_file(path);
        });
    }
    {
        let ui = Rc::clone(&app_ui);
        sidebar.set_on_outline_jump(move |heading| {
            ui.viewer.scroll_to(&heading.id);
        });
    }
    {
        let ui = Rc::clone(&app_ui);
        editor.set_on_saved(move || {
            ui.refresh_preview();
            ui.toast("已保存");
            ui.persist_config();
        });
    }

    {
        let ui = Rc::clone(&app_ui);
        let split = split.clone();
        sidebar_btn.connect_toggled(move |btn| {
            split.set_show_sidebar(btn.is_active());
            ui.persist_config();
        });
    }

    {
        let ui = Rc::clone(&app_ui);
        open_dir_btn.connect_clicked(move |_| {
            let dialog = gtk::FileDialog::new();
            dialog.set_title("选择工作目录");
            let parent = ui.window.clone();
            let ui2 = Rc::clone(&ui);
            dialog.select_folder(Some(&parent), gtk::gio::Cancellable::NONE, move |res| {
                if let Ok(folder) = res {
                    if let Some(path) = folder.path() {
                        let path = PathBuf::from(path);
                        ui2.sidebar.set_workdir(Some(path.clone()));
                        ui2.state.cfg.borrow_mut().workdir = Some(path);
                        ui2.persist_config();
                        ui2.sidebar.show_files();
                    }
                }
            });
        });
    }

    {
        let ui = Rc::clone(&app_ui);
        open_file_btn.connect_clicked(move |_| {
            let dialog = gtk::FileDialog::new();
            dialog.set_title("打开 Markdown 文件");
            let filter = gtk::FileFilter::new();
            filter.set_name(Some("Markdown"));
            filter.add_pattern("*.md");
            filter.add_pattern("*.markdown");
            filter.add_pattern("*.mdown");
            let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
            filters.append(&filter);
            dialog.set_filters(Some(&filters));

            let parent = ui.window.clone();
            let ui2 = Rc::clone(&ui);
            dialog.open(Some(&parent), gtk::gio::Cancellable::NONE, move |res| {
                if let Ok(file) = res {
                    if let Some(path) = file.path() {
                        ui2.open_file(PathBuf::from(path));
                    }
                }
            });
        });
    }

    {
        let ui = Rc::clone(&app_ui);
        edit_btn.connect_toggled(move |btn| {
            ui.set_edit_mode(btn.is_active());
        });
    }

    {
        let ui = Rc::clone(&app_ui);
        save_btn.connect_clicked(move |_| {
            ui.editor.save();
        });
    }

    {
        let ui = Rc::clone(&app_ui);
        export_btn.connect_clicked(move |_| {
            ui.on_export_pdf();
        });
    }

    {
        let ui = Rc::clone(&app_ui);
        let key = gtk::EventControllerKey::new();
        key.connect_key_pressed(move |_, k, _c, mods| {
            if mods.contains(gtk::gdk::ModifierType::CONTROL_MASK)
                && (k == gtk::gdk::Key::s || k == gtk::gdk::Key::S)
            {
                ui.editor.save();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        window.add_controller(key);
    }

    let last_file = app_ui.state.cfg.borrow().last_file.clone();
    if let Some(path) = initial {
        app_ui.open_file(path);
    } else if let Some(last) = last_file {
        if last.is_file() {
            app_ui.open_file(last);
        } else {
            app_ui.show_welcome();
        }
    } else {
        app_ui.show_welcome();
    }

    window.present();
}

impl App {
    fn toast(&self, msg: &str) {
        // ToastOverlay is directly under ToolbarView
        if let Some(child) = self.window.content() {
            if let Ok(overlay) = child.downcast::<adw::ToolbarView>() {
                if let Some(c) = overlay.content() {
                    if let Ok(ov) = c.downcast::<adw::ToastOverlay>() {
                        ov.add_toast(adw::Toast::new(msg));
                    }
                }
            }
        }
    }

    fn persist_config(&self) {
        let cfg = {
            let mut cfg = self.state.cfg.borrow_mut();
            cfg.edit_mode = *self.state.edit_mode.borrow();
            cfg.last_file = self.state.current_file.borrow().clone();
            cfg.sidebar_visible = self.sidebar_toggle.is_active();
            cfg.clone()
        };
        crate::config::save(&cfg);
    }

    fn set_edit_mode(&self, edit: bool) {
        *self.state.edit_mode.borrow_mut() = edit;
        self.content_stack
            .set_visible_child_name(if edit { "edit" } else { "preview" });
        self.save_btn.set_visible(edit);
        if !edit {
            self.refresh_preview();
        } else {
            self.editor.grab_focus();
        }
        self.persist_config();
    }

    fn show_welcome(&self) {
        self.title_label.set_text("Glance");
        self.subtitle_label
            .set_text("设置工作目录或打开 Markdown 文件");
        let welcome = r#"# 欢迎使用 Glance

低饱和护眼的 **Markdown + LaTeX** 阅读器。

## 快速开始

1. 点击左上角 **文件夹** 设置工作目录
2. 在侧边栏选择 `.md` 文件
3. 需要修改时点右上角 **编辑**（默认关闭）

## LaTeX 示例

行内公式 $E = mc^2$ 以及积分：

$$
\int_{-\infty}^{\infty} e^{-x^2}\,\mathrm{d}x = \sqrt{\pi}
$$

矩阵：

$$
\begin{pmatrix}
a & b \\
c & d
\end{pmatrix}
\begin{pmatrix}
x \\ y
\end{pmatrix}
=
\begin{pmatrix}
ax + by \\
cx + dy
\end{pmatrix}
$$

## 大纲

侧边栏「大纲」会根据 `#` 标题生成，点击可跳转。
"#;
        let headings = outline::extract_headings(welcome);
        *self.state.current_headings.borrow_mut() = headings.clone();
        self.sidebar.set_outline(&headings);
        let outline_json = serde_json::to_string(&headings).unwrap_or_else(|_| "[]".into());
        self.viewer.render_markdown(welcome, &outline_json);
        self.sidebar.show_outline();
        self.content_stack.set_visible_child_name("preview");
    }

    fn open_file(&self, path: PathBuf) {
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                *self.state.current_file.borrow_mut() = Some(path.clone());
                let _ = self.editor.load_file(&path);

                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "未命名".into());
                self.title_label.set_text(&name);
                self.subtitle_label.set_text(&path.display().to_string());

                let headings = outline::extract_headings(&text);
                *self.state.current_headings.borrow_mut() = headings.clone();
                self.sidebar.set_outline(&headings);

                let outline_json =
                    serde_json::to_string(&headings).unwrap_or_else(|_| "[]".into());
                self.viewer.render_markdown(&text, &outline_json);

                {
                    self.state.cfg.borrow_mut().last_file = Some(path);
                }
                self.persist_config();
                self.sidebar.show_outline();
            }
            Err(e) => {
                self.toast(&format!("打开失败: {e}"));
            }
        }
    }

    fn refresh_preview(&self) {
        let text = self.editor.text();
        let headings = outline::extract_headings(&text);
        *self.state.current_headings.borrow_mut() = headings.clone();
        self.sidebar.set_outline(&headings);
        let outline_json = serde_json::to_string(&headings).unwrap_or_else(|_| "[]".into());
        self.viewer.render_markdown(&text, &outline_json);
    }

    fn on_export_pdf(self: &Rc<Self>) {
        let Some(file) = self.state.current_file.borrow().clone() else {
            self.toast("请先打开 Markdown 文件");
            return;
        };
        let stem = file
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "document".into());
        let dialog = gtk::FileDialog::builder()
            .title("导出 PDF")
            .initial_name(format!("{stem}.pdf"))
            .build();
        let ui = Rc::clone(self);
        dialog.save(
            Some(&self.window),
            gtk::gio::Cancellable::NONE,
            move |res| {
                if let Ok(dest) = res {
                    let Some(path) = dest.path() else { return };
                    if path.as_os_str().is_empty() {
                        return;
                    }
                    let op = ui.viewer.prepare_export(&path);
                    {
                        let ui2 = Rc::clone(&ui);
                        op.connect_failed(move |_, err| {
                            ui2.viewer.clear_export();
                            ui2.toast(&format!("导出失败: {err}"));
                        });
                    }
                    {
                        let ui2 = Rc::clone(&ui);
                        let path2 = path.clone();
                        op.connect_finished(move |_| {
                            ui2.viewer.clear_export();
                            ui2.toast(&format!("PDF 已导出: {}", path2.display()));
                        });
                    }
                    ui.viewer.start_export(op);
                    ui.toast("正在导出 PDF…");
                }
            },
        );
    }
}

fn css_path() -> String {
    let candidates = [
        std::path::PathBuf::from("/usr/share/glance/glance.css"),
        std::path::PathBuf::from("/usr/local/share/glance/glance.css"),
        std::path::PathBuf::from("resources/glance.css"),
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/glance.css"),
    ];
    for c in candidates {
        if c.exists() {
            return c.display().to_string();
        }
    }
    "resources/glance.css".into()
}
