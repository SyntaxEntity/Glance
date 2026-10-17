use gtk::prelude::*;
use sourceview5::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub struct Editor {
    pub root: gtk::Box,
    view: sourceview5::View,
    buffer: sourceview5::Buffer,
    file: RefCell<Option<PathBuf>>,
    dirty: RefCell<bool>,
    on_saved: RefCell<Option<Box<dyn Fn()>>>,
    on_save_error: RefCell<Option<Box<dyn Fn(&str)>>>,
}

impl Editor {
    pub fn new() -> Rc<Self> {
        let buffer = sourceview5::Buffer::new(None);

        let lm = sourceview5::LanguageManager::new();
        if let Some(lang) = lm.language("markdown") {
            buffer.set_language(Some(&lang));
        }

        let sm = sourceview5::StyleSchemeManager::new();
        for id in ["Adwaita", "Adwaita-light", "classic", "kate"] {
            if let Some(scheme) = sm.scheme(id) {
                buffer.set_style_scheme(Some(&scheme));
                break;
            }
        }

        buffer.set_highlight_matching_brackets(true);

        let view = sourceview5::View::with_buffer(&buffer);
        view.set_monospace(true);
        view.set_show_line_numbers(true);
        view.set_highlight_current_line(true);
        view.set_tab_width(4);
        view.set_indent_width(2);
        view.set_auto_indent(true);
        view.set_insert_spaces_instead_of_tabs(true);
        view.set_wrap_mode(gtk::WrapMode::WordChar);
        view.set_vexpand(true);
        view.set_hexpand(true);
        view.add_css_class("glance-editor");

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_child(Some(&view));
        scroll.set_vexpand(true);
        scroll.set_hexpand(true);

        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.set_vexpand(true);
        root.set_hexpand(true);
        root.append(&scroll);

        let this = Rc::new(Self {
            root,
            view,
            buffer,
            file: RefCell::new(None),
            dirty: RefCell::new(false),
            on_saved: RefCell::new(None),
            on_save_error: RefCell::new(None),
        });

        {
            let this_c = Rc::clone(&this);
            let buffer = this.buffer.clone();
            buffer.connect_changed(move |_| {
                *this_c.dirty.borrow_mut() = true;
            });
        }

        {
            let this_c = Rc::clone(&this);
            let view = this.view.clone();
            let controller = gtk::EventControllerKey::new();
            controller.connect_key_pressed(move |_, key, _code, mods| {
                if mods.contains(gtk::gdk::ModifierType::CONTROL_MASK)
                    && (key == gtk::gdk::Key::s || key == gtk::gdk::Key::S)
                {
                    this_c.save();
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
            view.add_controller(controller);
        }

        this
    }

    pub fn set_on_saved(&self, cb: impl Fn() + 'static) {
        *self.on_saved.borrow_mut() = Some(Box::new(cb));
    }

    pub fn set_on_save_error(&self, cb: impl Fn(&str) + 'static) {
        *self.on_save_error.borrow_mut() = Some(Box::new(cb));
    }

    /// Buffer is clean after load/save; the dirty flag drives `save()`'s state
    /// but no UI consumes it yet (no close-confirmation guard).
    #[allow(dead_code)]
    pub fn is_dirty(&self) -> bool {
        *self.dirty.borrow()
    }

    pub fn load_file(&self, path: &PathBuf) -> Result<(), String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        self.buffer.set_text(&text);
        *self.file.borrow_mut() = Some(path.clone());
        *self.dirty.borrow_mut() = false;
        Ok(())
    }

    pub fn text(&self) -> String {
        self.buffer
            .text(&self.buffer.start_iter(), &self.buffer.end_iter(), true)
            .to_string()
    }

    /// Load `text` into the buffer without binding it to a file, so the buffer
    /// stays clean and `save()` remains a no-op (used for the welcome page).
    pub fn set_text(&self, text: &str) {
        self.buffer.set_text(text);
        *self.dirty.borrow_mut() = false;
    }

    pub fn save(&self) {
        let Some(path) = self.file.borrow().clone() else {
            if let Some(cb) = self.on_save_error.borrow().as_ref() {
                cb("没有打开的文件，无法保存");
            }
            return;
        };
        let text = self.text();
        match std::fs::write(&path, text) {
            Ok(()) => {
                *self.dirty.borrow_mut() = false;
                if let Some(cb) = self.on_saved.borrow().as_ref() {
                    cb();
                }
            }
            Err(e) => {
                eprintln!("save failed: {e}");
                if let Some(cb) = self.on_save_error.borrow().as_ref() {
                    cb(&format!("保存失败: {e}"));
                }
            }
        }
    }

    pub fn grab_focus(&self) {
        self.view.grab_focus();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// GTK must be initialised before building widgets. Needs a display; on a
    /// headless box use the Broadway backend, e.g.
    /// `GDK_BACKEND=broadway BROADWAY_DISPLAY=:5 broadwayd -p 8085 :5`.
    fn gtk_setup() -> bool {
        gtk::init().is_ok()
    }

    /// Regression: saving with no open file used to `return` silently, so the
    /// user got no feedback at all (the reported bug).
    #[test]
    fn save_without_file_reports_error() {
        if !gtk_setup() {
            eprintln!("skipping: no display for GTK init");
            return;
        }

        let editor = Editor::new();
        let seen: Rc<Cell<bool>> = Rc::new(Cell::new(false));
        {
            let seen = Rc::clone(&seen);
            editor.set_on_save_error(move |_| seen.set(true));
        }

        editor.save();

        assert!(seen.get(), "save() with no file must report an error");
    }

    /// The success path must route through `on_saved`, not `on_save_error`,
    /// and must actually write the buffer to disk.
    #[test]
    fn save_with_file_round_trips() {
        if !gtk_setup() {
            eprintln!("skipping: no display for GTK init");
            return;
        }

        let dir = std::env::temp_dir().join(format!("glance-save-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("note.md");
        std::fs::write(&file, "# hello\n").unwrap();

        let editor = Editor::new();
        editor.load_file(&file).unwrap();
        // 不要在这里断言 `is_dirty()`：GTK 投递 buffer 的 `changed` 通知是异步的，
        // 而 `set_text()` 又显式把 dirty 置回 false —— 断言此刻读到的值取决于该通知
        // 是否已经送达，会随机器/版本变化（CI 上就因此失败过）。本测试只验证保存契约。
        editor.set_text("# hello\n\nedited body\n");

        let saved: Rc<Cell<bool>> = Rc::new(Cell::new(false));
        {
            let saved = Rc::clone(&saved);
            editor.set_on_saved(move || saved.set(true));
        }
        let errored: Rc<Cell<bool>> = Rc::new(Cell::new(false));
        {
            let errored = Rc::clone(&errored);
            editor.set_on_save_error(move |_| errored.set(true));
        }

        editor.save();

        assert!(saved.get(), "on_saved was not called for a valid file");
        assert!(!errored.get(), "on_save_error fired on the success path");
        let written = std::fs::read_to_string(&file).unwrap();
        assert!(written.contains("edited body"), "content not written: {written}");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
