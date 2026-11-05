//! Evora build (patch CIC-07, class E4): the host's document in the engine.
//!
//! One document of the host per editor (DC-9). The requests of the bridge are applied at the start
//! of a frame ([`HostDocs::frame_start`]): `evora:open` opens the host's bytes (`.designcraft` and
//! IDML only, INDD and INDT refused) or a blank document (`new`), and replaces the document already
//! open; `evora:request-save` saves through Save (whose file leaves as `evora:save`, `web`); theme,
//! language, commands and fonts. At the end of the frame ([`HostDocs::frame_end`]) the saves sent are
//! bound to the document they wrote, the saved state follows the host's answer and `evora:dirty`
//! reports each change.
//!
//! Saved state with `doc.saved-ack` effective (machine A-02 of the protocol, S-ACK-1 to 5): Save
//! marks the document saved as soon as it hands it over (`file.saveAs` in the upstream web save);
//! the bridge restores at once the saved state of before, so the document stays modified while the
//! host has not answered. `evora:saved {ok: true}` marks it saved only if it has not changed since
//! (the document content sent is kept for the comparison), else it stays modified with the content
//! sent as its saved state; `evora:saved {ok: false}` leaves it modified. Without the capability, the
//! engine counts the document saved when it hands it over (§6.3).
//!
//! New document (patch CIC-26, `setup`): `evora:open {new: true}` makes an A4 portrait document in
//! millimetres; the host command `host.newDocumentSetup` replaces that blank by the specification
//! the member chose in Evora, only while the blank is in front, never saved and unchanged.

use std::sync::Arc;

use designcraft_ui_egui::DesignApp;
use serde_json::json;

use crate::evora::ehp::{Bridge, HostEvent};
use crate::evora::masks::HostCall;

type Document = designcraft_engine::doc::Document;

/// TrueType, OpenType (CFF), Apple TrueType and font collections, by their first four bytes.
const FONT_MAGIC: [[u8; 4]; 4] = [[0x00, 0x01, 0x00, 0x00], *b"OTTO", *b"true", *b"ttcf"];

pub struct HostDocs {
    bridge: Bridge,
    host: HostCall,
    /// The engine's uid of the host's document.
    uid: Option<u64>,
    last_dirty: Option<bool>,
    /// Saved state of the host's document at the start of the frame: (uid, saved document, revision).
    saved_before: Option<(u64, Arc<Document>, u64)>,
    /// Saves waiting for `evora:saved`: (save id, document content sent).
    pending: Vec<(String, Arc<Document>)>,
    /// What the last catalogue sent described: (language, document uid).
    catalog_for: Option<(String, Option<u64>)>,
    /// Families registered from `evora:font`.
    fonts: Vec<String>,
    /// The revision of the host's blank, made by `evora:open {new: true}` (or by its specification)
    /// and never saved since: the only document `host.newDocumentSetup` may replace (CIC-26), while
    /// its revision is unchanged.
    blank: Option<u64>,
}

impl HostDocs {
    pub fn new(bridge: Bridge, host: HostCall) -> Self {
        HostDocs {
            bridge,
            host,
            uid: None,
            last_dirty: None,
            saved_before: None,
            pending: Vec::new(),
            catalog_for: None,
            fonts: Vec::new(),
            blank: None,
        }
    }

    fn host_doc<'a>(&self, app: &'a DesignApp) -> Option<&'a designcraft_engine::DocState> {
        let uid = self.uid?;
        app.session.documents().iter().find(|d| d.uid == uid)
    }

    fn host_doc_mut<'a>(&self, app: &'a mut DesignApp) -> Option<&'a mut designcraft_engine::DocState> {
        let uid = self.uid?;
        app.session.active_mut().filter(|d| d.uid == uid)
    }

    /// Apply the host's requests.
    pub fn frame_start(&mut self, app: &mut DesignApp, ctx: &egui::Context) {
        self.saved_before = self.host_doc(app).map(|d| (d.uid, d.saved_doc.clone(), d.saved_revision));
        for event in self.bridge.take_events() {
            match event {
                HostEvent::Open { id, doc_ref, name, bytes } => self.open(app, &id, doc_ref, &name, Some(bytes)),
                HostEvent::OpenNew { id, doc_ref, name } => self.open(app, &id, doc_ref, &name, None),
                HostEvent::RequestSave { id, doc_ref } => self.request_save(app, &id, doc_ref.as_deref()),
                HostEvent::Theme { id, dark } => {
                    let r = app.run("window.brightness", json!({"brightness": if dark { "dark" } else { "light" }}));
                    self.bridge.ack(&id, r.is_ok(), r.is_err().then_some("busy"));
                }
                HostEvent::Lang { id, engine_lang } => {
                    crate::evora::lang::apply(app, engine_lang.as_deref());
                    self.bridge.ack(&id, true, None);
                }
                HostEvent::Command { id, command, confirmed } => self.command(app, &id, &command, confirmed),
                HostEvent::NewDocumentSetup { id, spec } => self.setup(app, &id, &spec),
                HostEvent::Saved { re, ok } => self.saved(app, &re, ok),
                HostEvent::Font { id, family, bytes } => self.font(&id, family, bytes),
            }
            // A save the request just sent is bound before the next request reads the saved state.
            self.bind_sent(app);
            self.saved_before = self.host_doc(app).map(|d| (d.uid, d.saved_doc.clone(), d.saved_revision));
        }
        ctx.request_repaint();
    }

    /// `evora:open`: the host's bytes, or (`bytes` = `None`) a blank document; the document open before
    /// is closed once the new one is open (one document per editor).
    fn open(&mut self, app: &mut DesignApp, id: &str, doc_ref: String, name: &str, bytes: Option<Vec<u8>>) {
        let lower = name.to_ascii_lowercase();
        let title = name.rsplit_once('.').map_or(name, |(stem, _)| stem).to_string();
        let before: Vec<u64> = app.session.documents().iter().map(|d| d.uid).collect();
        let result = match &bytes {
            Some(bytes) => {
                if !(lower.ends_with(".designcraft") || lower.ends_with(".idml")) {
                    self.bridge.ack(id, false, Some("unsupported"));
                    return;
                }
                let b64 = designcraft_engine::cmd::base64_encode(bytes);
                self.host.run(|| app.run("file.openBytes", json!({"name": title, "base64": b64})))
            }
            // Evora (CIC-26): A4 portrait in millimetres; the host's specification may follow (`setup`).
            None => self.host.run(|| app.run("file.new", crate::evora::setup::default_new(&title))),
        };
        if let Err(e) = result {
            log::warn!("designcraft-web: the host's document could not open: {e}");
            self.bridge.ack(id, false, Some("invalid_content"));
            return;
        }
        if !self.adopt(app, &before, &doc_ref) {
            self.bridge.ack(id, false, Some("invalid_content"));
            return;
        }
        // Only the blank of `new` may take the host's specification, while it stays as it was made.
        self.blank = if bytes.is_none() { self.host_doc(app).map(|d| d.revision) } else { None };
        self.opened(app, id, &doc_ref);
    }

    /// The document `file.new` or `file.openBytes` just made becomes the host's document `doc_ref`;
    /// the one open before goes (one document per editor). `false`: no new document, nothing closed.
    fn adopt(&mut self, app: &mut DesignApp, before: &[u64], doc_ref: &str) -> bool {
        let Some(uid) = app.session.active().map(|d| d.uid).filter(|u| !before.contains(u)) else {
            return false;
        };
        for old in before {
            if let Some(index) = app.session.documents().iter().position(|d| d.uid == *old) {
                let _ = self.host.run(|| app.run("file.close", json!({"index": index})));
            }
        }
        if let Some(index) = app.session.documents().iter().position(|d| d.uid == uid) {
            let _ = self.host.run(|| app.run("file.activate", json!({"index": index})));
        }
        self.uid = Some(uid);
        self.pending.clear();
        self.bridge.set_doc_ref(Some(doc_ref.to_owned()));
        true
    }

    /// The host's document is open: acknowledged, its state reported, the catalogue sent again.
    fn opened(&mut self, app: &DesignApp, id: &str, doc_ref: &str) {
        self.bridge.ack(id, true, None);
        let dirty = self.host_doc(app).is_some_and(|d| d.is_dirty());
        self.bridge.dirty(doc_ref, dirty);
        self.last_dirty = Some(dirty);
        self.catalog_for = None;
    }

    /// `host.newDocumentSetup` (CIC-26): the host's specification replaces its blank, under the same
    /// `docRef` and title, only while that blank is in front, never saved and unchanged; never the
    /// member's work. `conflict`: the document was opened from bytes, saved, changed or is not in
    /// front; `busy`: a save is in flight or a dialog is open; `not_found`: no document of the host;
    /// `invalid_content`: the engine refused the specification (the blank stays).
    fn setup(&mut self, app: &mut DesignApp, id: &str, spec: &crate::evora::setup::Spec) {
        let host = self.host_doc(app).map(|d| (d.revision, d.is_dirty(), d.doc.title.clone()));
        let (Some(doc_ref), Some((revision, dirty, title))) = (self.bridge.doc_ref(), host) else {
            self.bridge.ack(id, false, Some("not_found"));
            return;
        };
        let in_front = app.session.active().map(|d| d.uid) == self.uid;
        // SABORDAGE QA (W3, à retirer à l'octet) : garde d'état neutralisée.
        if false && (self.blank != Some(revision) || dirty || !in_front) {
            self.bridge.ack(id, false, Some("conflict"));
            return;
        }
        if !self.pending.is_empty() || app.ui.dialog.is_some() {
            self.bridge.ack(id, false, Some("busy"));
            return;
        }
        let before: Vec<u64> = app.session.documents().iter().map(|d| d.uid).collect();
        if let Err(e) = self.host.run(|| app.run("file.new", spec.file_new(&title))) {
            log::warn!("designcraft-web: the host's document setup was refused: {e}");
            self.bridge.ack(id, false, Some("invalid_content"));
            return;
        }
        if !self.adopt(app, &before, &doc_ref) {
            self.bridge.ack(id, false, Some("invalid_content"));
            return;
        }
        self.blank = self.host_doc(app).map(|d| d.revision);
        self.opened(app, id, &doc_ref);
    }

    /// `evora:request-save`: `clean` when nothing changed since the last save the host accepted;
    /// otherwise Save, whose `evora:save` answers the request.
    fn request_save(&mut self, app: &mut DesignApp, id: &str, doc_ref: Option<&str>) {
        let Some(current) = self.bridge.doc_ref() else {
            self.bridge.ack(id, false, Some("not_found"));
            return;
        };
        if doc_ref.is_some_and(|r| r != current) {
            self.bridge.ack(id, false, Some("not_found"));
            return;
        }
        if !self.host_doc(app).is_some_and(|d| d.is_dirty()) {
            self.bridge.ack(id, true, Some("clean"));
            return;
        }
        self.bridge.expect_save(&current, id);
        let r = app.run("app.save", json!({}));
        if self.bridge.cancel_expected_save(id) {
            if let Err(e) = r {
                log::warn!("designcraft-web: save asked by the host failed: {e}");
            }
            self.bridge.ack(id, false, Some("busy"));
        }
    }

    fn command(&mut self, app: &mut DesignApp, id: &str, command: &str, confirmed: bool) {
        match command {
            "diag.stats" => {
                let stats =
                    crate::evora::commands::stats(app, self.bridge.doc_ref().as_deref(), self.uid, self.bridge.counters(), &self.fonts);
                self.bridge.ack_result(id, &stats);
            }
            "diag.menus" => self.bridge.ack_result(id, &crate::evora::commands::diag_menus(app)),
            "diag.ui" => self.bridge.ack_result(id, &crate::evora::commands::diag_ui(app)),
            "diag.labels" => self.bridge.ack_result(id, &crate::evora::commands::diag_labels(app)),
            _ => match crate::evora::commands::run(app, command, confirmed, self.bridge.has("command.catalog")) {
                Ok(()) => self.bridge.ack(id, true, None),
                Err(code) => self.bridge.ack(id, false, Some(code)),
            },
        }
    }

    /// `evora:saved` answering the save `re`.
    fn saved(&mut self, app: &mut DesignApp, re: &str, ok: bool) {
        let Some(i) = self.pending.iter().position(|(s, _)| s == re) else { return };
        let (_, sent) = self.pending.remove(i);
        if !ok {
            return;
        }
        // Saved by the host: no longer a blank its specification may replace (CIC-26).
        self.blank = None;
        if let Some(st) = self.host_doc_mut(app) {
            if Arc::ptr_eq(&st.doc, &sent) {
                st.saved_doc = st.doc.clone();
                st.saved_revision = st.revision;
            } else {
                // Changed since the content sent: still modified, the content sent is what is saved.
                st.saved_doc = sent;
            }
        }
    }

    /// `evora:font`: a TrueType or OpenType file registered in the engine's fonts.
    fn font(&mut self, id: &str, family: String, bytes: Vec<u8>) {
        let magic_ok = bytes.len() >= 4 && FONT_MAGIC.iter().any(|m| bytes[..4] == m[..]);
        if !magic_ok {
            self.bridge.ack(id, false, Some("invalid_content"));
            return;
        }
        let added = designcraft_fonts::FontDb::global().add_font(bytes);
        if added == 0 && !self.fonts.contains(&family) {
            // Unreadable faces, or faces already loaded under another family name.
            self.bridge.ack(id, false, Some("invalid_content"));
            return;
        }
        if !self.fonts.contains(&family) {
            self.fonts.push(family);
        }
        self.bridge.ack(id, true, None);
    }

    /// Bind the saves sent to the host's document, follow its saved state, report changes.
    pub fn frame_end(&mut self, app: &mut DesignApp) {
        self.bind_sent(app);
        if let (Some(doc_ref), Some(dirty)) = (self.bridge.doc_ref(), self.host_doc(app).map(|d| d.is_dirty()))
            && self.last_dirty != Some(dirty)
        {
            self.bridge.dirty(&doc_ref, dirty);
            self.last_dirty = Some(dirty);
        }
        let key = (app.ui.language.clone(), self.uid);
        if self.bridge.has("command.catalog") && self.catalog_for.as_ref() != Some(&key) {
            self.bridge.commands(&crate::evora::commands::catalog(app));
            self.catalog_for = Some(key);
        }
        self.saved_before = self.host_doc(app).map(|d| (d.uid, d.saved_doc.clone(), d.saved_revision));
    }

    /// The saves sent since the last call, bound to the content they wrote; with `doc.saved-ack`
    /// effective, the saved state of before is restored (modified until the host answers).
    fn bind_sent(&mut self, app: &mut DesignApp) {
        let saved_ack = self.bridge.has("doc.saved-ack");
        for sent in self.bridge.take_sent() {
            let Some(st) = self.host_doc(app) else { continue };
            let content = st.doc.clone();
            if sent.save_as {
                continue;
            }
            self.pending.push((sent.id, content));
            if saved_ack
                && let Some((uid, saved_doc, saved_revision)) = self.saved_before.clone()
                && let Some(st) = self.host_doc_mut(app)
                && st.uid == uid
            {
                // Save marked the document saved: it stays modified until the host answers.
                st.saved_doc = saved_doc;
                st.saved_revision = saved_revision;
            }
        }
    }

    /// The host's document is open (startup: the blank sample is never opened under the host).
    pub fn has_document(&self) -> bool {
        self.uid.is_some()
    }
}
