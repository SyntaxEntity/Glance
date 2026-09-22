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

    #[allow(dead_code)]
    pub fn file(&self) -> Option<PathBuf> {
        self.file.borrow().clone()
    }

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

    #[allow(dead_code)]
    pub fn set_text(&self, text: &str) {
        self.buffer.set_text(text);
        *self.dirty.borrow_mut() = false;
    }

    pub fn save(&self) {
        let Some(path) = self.file.borrow().clone() else {
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
            }
        }
    }

    pub fn grab_focus(&self) {
        self.view.grab_focus();
    }
}
