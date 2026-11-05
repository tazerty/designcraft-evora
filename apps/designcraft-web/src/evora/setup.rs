//! Evora build (patch CIC-26, class E4): the new document Evora asks for.
//!
//! `evora:open {docRef, name, new: true}` makes an A4 portrait document in millimetres, the engine's
//! other defaults kept (the `A4` preset of `file.new`, [`default_new`]). The host may then apply the
//! specification the member chose on Evora's creation page with `evora:command {command:
//! "host.newDocumentSetup", params}`: a host command outside the catalogue (the catalogue holds menu
//! items only; like the diagnostics of `commands`, it is never one), since `evora:open` takes no field
//! more (protocol rule).
//!
//! - `params` is closed, every key required, lengths in millimetres: `{width, height, pages,
//!   facingPages, columns, gutter, margins: {top, bottom, inside, outside}, bleed}`. The bounds are
//!   the engine's own (`file.new`): page sides above 0 and at most 216 in, 1 to 9999 pages, 1 to 216
//!   columns, a bleed from 0 to 1296 pt, margins and gutter at least 0, every number finite (in
//!   millimetres and in points). Anything else is refused with `invalid_content` ([`parse`]) and
//!   changes nothing.
//! - It applies only to the host's blank, the document `evora:open {new: true}` made in this session,
//!   in front, never saved (no `evora:saved {ok: true}` for it) and never changed (`docs`): the blank
//!   is replaced by `file.new` with the specification ([`Spec::file_new`]), same title, same `docRef`,
//!   the blank closed. It never takes the member's work away: in any other state it is refused.

use serde_json::{Value, json};
use wasm_bindgen::JsValue;

use crate::evora::ehp::{get, only_keys, plain_object};

/// The host command (never a menu item, never in the catalogue).
pub const COMMAND: &str = "host.newDocumentSetup";
/// The preset of a blank of the host without a specification: A4 portrait, in millimetres.
const DEFAULT_PRESET: &str = "A4";
/// The document's units under a specification (`file.new` `units`).
const MILLIMETERS: &str = "millimeters";

const PT_PER_MM: f64 = 72.0 / 25.4;
/// The engine's bounds (`file.new`): a page side of 216 in, a bleed of 1296 pt, in points then in
/// millimetres, and the counts of pages and columns.
const MAX_SIDE_PT: f64 = 15_552.0;
const MAX_SIDE_MM: f64 = 5_486.4;
const MAX_BLEED_PT: f64 = 1_296.0;
const MAX_BLEED_MM: f64 = 457.2;
const MAX_PAGES: f64 = 9_999.0;
const MAX_COLUMNS: f64 = 216.0;

/// The keys of `params`, all required, none other.
const KEYS: [&str; 8] = ["width", "height", "pages", "facingPages", "columns", "gutter", "margins", "bleed"];
/// The keys of `params.margins`, all required, none other.
const MARGIN_KEYS: [&str; 4] = ["top", "bottom", "inside", "outside"];

/// A specification checked against the engine's bounds, lengths in millimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spec {
    width: f64,
    height: f64,
    pages: u32,
    facing_pages: bool,
    columns: u32,
    gutter: f64,
    /// Top, bottom, inside, outside.
    margins: [f64; 4],
    bleed: f64,
}

/// `file.new` of a blank of the host (`evora:open {new: true}`): A4 portrait in millimetres.
pub fn default_new(title: &str) -> Value {
    json!({"title": title, "preset": DEFAULT_PRESET})
}

/// The `params` of `host.newDocumentSetup`, checked; `None`: refused (`invalid_content`).
pub fn parse(params: &JsValue) -> Option<Spec> {
    if !plain_object(params) || !only_keys(params, &KEYS) {
        return None;
    }
    let margins = get(params, "margins");
    if !plain_object(&margins) || !only_keys(&margins, &MARGIN_KEYS) {
        return None;
    }
    let [top, bottom, inside, outside] = MARGIN_KEYS.map(|k| length(&get(&margins, k)));
    Some(Spec {
        width: side(&get(params, "width"))?,
        height: side(&get(params, "height"))?,
        pages: count(&get(params, "pages"), MAX_PAGES)?,
        facing_pages: get(params, "facingPages").as_bool()?,
        columns: count(&get(params, "columns"), MAX_COLUMNS)?,
        gutter: length(&get(params, "gutter"))?,
        margins: [top?, bottom?, inside?, outside?],
        bleed: length(&get(params, "bleed")).filter(|mm| *mm <= MAX_BLEED_MM)?,
    })
}

/// A length: a finite number of millimetres, at least 0, whose points stay finite.
fn length(v: &JsValue) -> Option<f64> {
    v.as_f64().filter(|mm| mm.is_finite() && *mm >= 0.0 && (*mm * PT_PER_MM).is_finite())
}

/// A page side: a length above 0 and at most 216 in.
fn side(v: &JsValue) -> Option<f64> {
    length(v).filter(|mm| *mm > 0.0 && *mm <= MAX_SIDE_MM)
}

/// A whole number from 1 to `max`.
fn count(v: &JsValue, max: f64) -> Option<u32> {
    if !js_sys::Number::is_safe_integer(v) {
        return None;
    }
    v.as_f64().filter(|n| (1.0..=max).contains(n)).map(|n| n as u32)
}

impl Spec {
    /// `file.new` with this specification and `title`: lengths in points (the engine's), the
    /// document's units in millimetres. The bounds in points absorb the rounding of the conversion.
    pub fn file_new(&self, title: &str) -> Value {
        let pt = |mm: f64| mm * PT_PER_MM;
        let [top, bottom, inside, outside] = self.margins.map(pt);
        json!({
            "title": title,
            "units": MILLIMETERS,
            "width": pt(self.width).min(MAX_SIDE_PT),
            "height": pt(self.height).min(MAX_SIDE_PT),
            "pages": self.pages,
            "facingPages": self.facing_pages,
            "columns": self.columns,
            "gutter": pt(self.gutter),
            "margins": {"top": top, "bottom": bottom, "inside": inside, "outside": outside},
            "bleed": pt(self.bleed).min(MAX_BLEED_PT),
        })
    }
}
