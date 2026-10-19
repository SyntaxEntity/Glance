use gtk::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use webkit6::prelude::*;

pub struct Viewer {
    pub root: gtk::Box,
    webview: webkit6::WebView,
    preview_dir: PathBuf,
    last_md: RefCell<String>,
    last_outline: RefCell<String>,
    loaded: RefCell<bool>,
    export_op: RefCell<Option<webkit6::PrintOperation>>,
}

impl Viewer {
    pub fn new() -> Rc<Self> {
        let webview = webkit6::WebView::new();
        webview.set_vexpand(true);
        webview.set_hexpand(true);

        if let Some(s) = webkit6::prelude::WebViewExt::settings(&webview) {
            s.set_enable_smooth_scrolling(true);
            s.set_auto_load_images(true);
            s.set_enable_javascript(true);
            s.set_enable_developer_extras(false);
            s.set_default_font_size(17);
            s.set_allow_file_access_from_file_urls(true);
            s.set_allow_universal_access_from_file_urls(true);
        }

        let preview_dir = resolve_preview_dir();

        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.set_vexpand(true);
        root.set_hexpand(true);
        root.append(&webview);

        let this = Rc::new(Self {
            root,
            webview,
            preview_dir,
            last_md: RefCell::new(String::new()),
            last_outline: RefCell::new("[]".into()),
            loaded: RefCell::new(false),
            export_op: RefCell::new(None),
        });

        {
            let this_c = Rc::clone(&this);
            let webview = this.webview.clone();
            webview.connect_load_changed(move |_, event| {
                use webkit6::LoadEvent;
                if event == LoadEvent::Finished {
                    *this_c.loaded.borrow_mut() = true;
                    this_c.flush();
                } else if event == LoadEvent::Started {
                    *this_c.loaded.borrow_mut() = false;
                }
            });
        }

        let uri = format!("file://{}/index.html", this.preview_dir.display());
        this.webview.load_uri(&uri);

        this
    }

    pub fn render_markdown(&self, markdown: &str, outline_json: &str) {
        *self.last_md.borrow_mut() = markdown.to_string();
        *self.last_outline.borrow_mut() = outline_json.to_string();
        self.flush();
    }

    pub fn scroll_to(&self, id: &str) {
        let id_json = serde_json::to_string(id).unwrap_or_else(|_| "\"\"".into());
        let js =
            format!("if (typeof scrollToHeading === 'function') {{ scrollToHeading({id_json}); }}");
        self.eval_js(&js);
    }

    /// Build a print operation that writes PDF to `dest`. Caller connects
    /// signals then calls `start_export` (so handlers are not missed).
    pub fn prepare_export(&self, dest: &std::path::Path) -> webkit6::PrintOperation {
        let uri = gtk::gio::File::for_path(dest).uri();
        let settings = gtk::PrintSettings::new();
        settings.set(gtk::PRINT_SETTINGS_OUTPUT_URI, Some(uri.as_str()));
        settings.set(gtk::PRINT_SETTINGS_OUTPUT_FILE_FORMAT, Some("pdf"));
        settings.set_printer("Print to File");
        let op = webkit6::PrintOperation::new(&self.webview);
        op.set_print_settings(&settings);
        op
    }

    /// Start a prepared export and keep the operation alive until clear_export.
    pub fn start_export(&self, op: webkit6::PrintOperation) {
        *self.export_op.borrow_mut() = Some(op.clone());
        op.print();
    }

    pub fn clear_export(&self) {
        *self.export_op.borrow_mut() = None;
    }

    fn flush(&self) {
        if !*self.loaded.borrow() {
            // Still stash for the page's flushPending()
            let md = self.last_md.borrow().clone();
            let outline = self.last_outline.borrow().clone();
            if md.is_empty() {
                return;
            }
            let md_json = serde_json::to_string(&md).unwrap_or_else(|_| "\"\"".into());
            let outline_arg = outline.clone();
            let outline_js =
                serde_json::to_string(&outline_arg).unwrap_or_else(|_| "\"[]\"".into());
            let js = format!(
                "window.__pending_md = {md_json}; window.__pending_outline = {outline_js};"
            );
            self.eval_js(&js);
            return;
        }

        let md = self.last_md.borrow().clone();
        let outline = self.last_outline.borrow().clone();
        let md_json = serde_json::to_string(&md).unwrap_or_else(|_| "\"\"".into());
        // outline is already a JSON array string; pass as JS string literal
        let outline_js = serde_json::to_string(&outline).unwrap_or_else(|_| "\"[]\"".into());
        let js = format!(
            "if (typeof renderMarkdown === 'function') {{ renderMarkdown({md_json}, {outline_js}); }} else {{ window.__pending_md = {md_json}; window.__pending_outline = {outline_js}; flushPending && flushPending(); }}"
        );
        self.eval_js(&js);
    }

    fn eval_js(&self, js: &str) {
        let webview = self.webview.clone();
        let script = js.to_string();
        webview.evaluate_javascript(
            &script,
            None,
            None,
            None::<&gtk::gio::Cancellable>,
            move |result| {
                if let Err(e) = result {
                    eprintln!("[glance] js error: {e}");
                }
            },
        );
    }
}

fn resolve_preview_dir() -> PathBuf {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // System install (RPM / distro package)
    candidates.push(PathBuf::from("/usr/share/glance/preview"));
    candidates.push(PathBuf::from("/usr/local/share/glance/preview"));

    // Relative to the binary (dev tree / cargo run)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("resources/preview"));
            candidates.push(dir.join("../resources/preview"));
            candidates.push(dir.join("../../resources/preview"));
            candidates.push(dir.join("../../../resources/preview"));
        }
    }

    // Compile-time project root (dev)
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        candidates.push(PathBuf::from(manifest).join("resources/preview"));
    }
    candidates.push(PathBuf::from("resources/preview"));

    for c in candidates {
        if c.join("index.html").exists() {
            return c.canonicalize().unwrap_or(c);
        }
    }

    PathBuf::from("resources/preview")
}
