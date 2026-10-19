use crate::outline::Heading;
use gtk::pango;
use gtk::prelude::*;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum TreeItem {
    Dir { path: PathBuf, name: String },
    File { path: PathBuf, name: String },
}

/// 打开文件的回调；抽别名以简化字段签名（clippy::type_complexity）。
type OpenFileCallback = Box<dyn Fn(PathBuf)>;
/// 大纲跳转的回调；理由同上。
type OutlineJumpCallback = Box<dyn Fn(Heading)>;

pub struct Sidebar {
    pub root: gtk::Box,
    stack: gtk::Stack,
    files_list: gtk::ListBox,
    outline_list: gtk::ListBox,
    path_label: gtk::Label,
    workdir: RefCell<Option<PathBuf>>,
    on_open_file: RefCell<Option<OpenFileCallback>>,
    on_outline_jump: RefCell<Option<OutlineJumpCallback>>,
    expanded: RefCell<Vec<PathBuf>>,
    headings: RefCell<Vec<Heading>>,
}

impl Sidebar {
    pub fn new() -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.add_css_class("glance-sidebar");
        root.add_css_class("sidebar");

        let switcher = gtk::StackSwitcher::new();
        switcher.set_halign(gtk::Align::Center);
        root.append(&switcher);

        let path_label = gtk::Label::new(None);
        path_label.add_css_class("glance-sub-label");
        path_label.set_ellipsize(pango::EllipsizeMode::Start);
        path_label.set_margin_start(12);
        path_label.set_margin_end(12);
        path_label.set_margin_top(4);
        path_label.set_margin_bottom(4);
        path_label.set_xalign(0.0);
        root.append(&path_label);

        let stack = gtk::Stack::new();
        stack.set_hexpand(true);
        stack.set_vexpand(true);
        switcher.set_stack(Some(&stack));

        let files_page = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let files_list = gtk::ListBox::new();
        files_list.set_selection_mode(gtk::SelectionMode::Single);
        files_list.add_css_class("navigation-sidebar");
        files_list.set_vexpand(true);

        let files_scroll = gtk::ScrolledWindow::new();
        files_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        files_scroll.set_vexpand(true);
        files_scroll.set_child(Some(&files_list));
        files_page.append(&files_scroll);

        stack.add_titled(&files_page, Some("files"), "文件");

        let outline_page = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let outline_list = gtk::ListBox::new();
        outline_list.set_selection_mode(gtk::SelectionMode::Single);
        outline_list.add_css_class("navigation-sidebar");
        outline_list.set_vexpand(true);

        let outline_scroll = gtk::ScrolledWindow::new();
        outline_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        outline_scroll.set_child(Some(&outline_list));
        outline_page.append(&outline_scroll);

        stack.add_titled(&outline_page, Some("outline"), "大纲");
        root.append(&stack);

        let this = Rc::new(Self {
            root,
            stack,
            files_list,
            outline_list,
            path_label,
            workdir: RefCell::new(None),
            on_open_file: RefCell::new(None),
            on_outline_jump: RefCell::new(None),
            expanded: RefCell::new(Vec::new()),
            headings: RefCell::new(Vec::new()),
        });

        {
            let this_weak = Rc::clone(&this);
            let files_list = this.files_list.clone();
            files_list.connect_row_activated(move |_, row| {
                let this = &this_weak;
                let index = row.index();
                let items = this.visible_items();
                if let Some(item) = items.get(index as usize) {
                    match item {
                        TreeItem::Dir { path, .. } => {
                            this.toggle_expand(path.clone());
                        }
                        TreeItem::File { path, .. } => {
                            if let Some(cb) = this.on_open_file.borrow().as_ref() {
                                cb(path.clone());
                            }
                        }
                    }
                }
            });
        }

        {
            let this_weak = Rc::clone(&this);
            let outline_list = this.outline_list.clone();
            outline_list.connect_row_activated(move |_, row| {
                let this = &this_weak;
                let index = row.index() as usize;
                let headings = this.headings.borrow().clone();
                if let (Some(h), Some(cb)) =
                    (headings.get(index), this.on_outline_jump.borrow().as_ref())
                {
                    cb(h.clone());
                }
            });
        }

        this
    }

    pub fn set_on_open_file(&self, cb: impl Fn(PathBuf) + 'static) {
        *self.on_open_file.borrow_mut() = Some(Box::new(cb));
    }

    pub fn set_on_outline_jump(&self, cb: impl Fn(Heading) + 'static) {
        *self.on_outline_jump.borrow_mut() = Some(Box::new(cb));
    }

    pub fn show_files(&self) {
        self.stack.set_visible_child_name("files");
    }

    pub fn show_outline(&self) {
        self.stack.set_visible_child_name("outline");
    }

    pub fn set_workdir(&self, dir: Option<PathBuf>) {
        *self.workdir.borrow_mut() = dir.clone();
        match dir {
            Some(d) => {
                self.path_label.set_text(&d.display().to_string());
                self.path_label
                    .set_tooltip_text(Some(&d.display().to_string()));
            }
            None => {
                self.path_label.set_text("未设置工作目录");
                self.path_label.set_tooltip_text(None);
            }
        }
        self.refresh_files();
    }

    pub fn refresh_files(&self) {
        while let Some(row) = self.files_list.row_at_index(0) {
            self.files_list.remove(&row);
        }

        let Some(root) = self.workdir.borrow().clone() else {
            return;
        };

        {
            let mut exp = self.expanded.borrow_mut();
            if !exp.contains(&root) {
                exp.push(root.clone());
            }
        }

        for item in self.visible_items() {
            let row = self.make_tree_row(&item);
            self.files_list.append(&row);
        }
    }

    fn toggle_expand(&self, path: PathBuf) {
        {
            let mut exp = self.expanded.borrow_mut();
            if let Some(pos) = exp.iter().position(|p| *p == path) {
                exp.remove(pos);
            } else {
                exp.push(path);
            }
        }
        self.refresh_files();
    }

    fn is_expanded(&self, path: &Path) -> bool {
        self.expanded.borrow().iter().any(|p| p == path)
    }

    fn visible_items(&self) -> Vec<TreeItem> {
        let mut out = Vec::new();
        let Some(root) = self.workdir.borrow().clone() else {
            return out;
        };
        collect_level(&root, self, &mut out);
        out
    }

    fn depth_for(&self, path: &Path, root: &Path) -> usize {
        let rel = path.strip_prefix(root).unwrap_or(path);
        rel.components().count().saturating_sub(1)
    }

    fn make_tree_row(&self, item: &TreeItem) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 6);

        let (path, name, is_dir) = match item {
            TreeItem::Dir { path, name } => (path, name, true),
            TreeItem::File { path, name } => (path, name, false),
        };

        let root = self.workdir.borrow().clone().unwrap_or_default();
        let depth = self.depth_for(path, &root);
        hbox.set_margin_start(8 + (depth as i32) * 14);

        if is_dir {
            let icon_name = if self.is_expanded(path) {
                "folder-open-symbolic"
            } else {
                "folder-symbolic"
            };
            let img = gtk::Image::from_icon_name(icon_name);
            img.set_pixel_size(16);
            hbox.append(&img);

            let label = gtk::Label::new(Some(name));
            label.add_css_class("glance-dir-label");
            label.set_xalign(0.0);
            label.set_hexpand(true);
            label.set_ellipsize(pango::EllipsizeMode::End);
            hbox.append(&label);

            let arrow = gtk::Image::from_icon_name(if self.is_expanded(path) {
                "pan-down-symbolic"
            } else {
                "pan-end-symbolic"
            });
            arrow.set_pixel_size(12);
            arrow.set_opacity(0.55);
            hbox.append(&arrow);
        } else {
            let img = gtk::Image::from_icon_name("text-x-markdown-symbolic");
            img.set_pixel_size(14);
            img.set_opacity(0.75);
            hbox.append(&img);

            let label = gtk::Label::new(Some(name));
            label.add_css_class("glance-file-label");
            label.set_xalign(0.0);
            label.set_hexpand(true);
            label.set_ellipsize(pango::EllipsizeMode::End);
            hbox.append(&label);
        }

        row.set_child(Some(&hbox));
        row
    }

    pub fn set_outline(&self, headings: &[Heading]) {
        *self.headings.borrow_mut() = headings.to_vec();
        while let Some(row) = self.outline_list.row_at_index(0) {
            self.outline_list.remove(&row);
        }

        for h in headings {
            let row = gtk::ListBoxRow::new();
            let label = gtk::Label::new(Some(&h.text));
            label.set_xalign(0.0);
            label.set_ellipsize(pango::EllipsizeMode::End);
            label.add_css_class(match h.level {
                1 => "glance-outline-l1",
                2 => "glance-outline-l2",
                3 => "glance-outline-l3",
                _ => "glance-outline-l4",
            });
            row.set_child(Some(&label));
            self.outline_list.append(&row);
        }
    }
}

fn collect_level(dir: &Path, sidebar: &Sidebar, out: &mut Vec<TreeItem>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    let mut dirs: Vec<(PathBuf, String)> = Vec::new();
    let mut files: Vec<(PathBuf, String)> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.is_dir() {
            dirs.push((path, name));
        } else if meta.is_file() && is_markdown(&name) {
            files.push((path, name));
        }
    }

    dirs.sort_by_key(|a| a.1.to_lowercase());
    files.sort_by_key(|a| a.1.to_lowercase());

    for (path, name) in dirs {
        let expanded = sidebar.is_expanded(&path);
        out.push(TreeItem::Dir {
            path: path.clone(),
            name,
        });
        if expanded {
            collect_level(&path, sidebar, out);
        }
    }
    for (path, name) in files {
        out.push(TreeItem::File { path, name });
    }
}

fn is_markdown(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".md") || lower.ends_with(".markdown") || lower.ends_with(".mdown")
}
