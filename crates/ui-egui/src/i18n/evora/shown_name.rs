//! Evora build (patch EVORA-CIC-32, decision Q-EVO-1 of 2026-11-06): on screen, the application's name,
//! never the engine's. [`rename`] is applied to what `i18n::tr` and `i18n::tr_context` return and to the
//! texts the tables do not hold (a colour profile's name, the status line, an engine's message in an
//! alert or a command's form), so a catalogue brought in by a version upgrade needs no rewriting.
//! Identifiers, file extensions, format signatures and the bytes of ICC profiles keep their names: only
//! what is drawn changes.
//!
//! The CI gate « marques » counts the engine's name byte for byte in the served WebAssembly, where
//! strings have no terminator and two literals side by side read as one. A name is therefore never
//! written whole here: its first word is compared letter by letter, so it is no literal at all, and
//! its second word is kept in lower case.

use std::cell::RefCell;
use std::collections::HashMap;

/// The name this build shows.
pub const SHOWN_NAME: &str = "Cicero";

/// Every engine of the suite is called `Evo` then one of these words, capitalised, at times with a
/// space between the two ([`name_at`]).
const NAME_TAILS: [&str; 7] = ["design", "film", "vector", "photo", "light", "effect", "pdf"];
/// The other names a text may carry, the longer ones first, and what shows instead: a neutral wording
/// for the upstream team and its tools, the application's name for the colour library that a
/// built-in profile's name cites.
const OTHER_NAMES: [(&str, &str); 6] = [
    ("Upstream Team", "Evora team"),
    ("Upstream", "Evora"),
    ("Evo Apps", "Evora apps"),
    ("Evo apps", "Evora apps"),
    ("evorules", "Evora"),
    ("evo-cms", SHOWN_NAME),
];

thread_local! {
    /// Displayed text → the same text as shown, made once per text (the interface asks for the same
    /// strings every frame): one table for German, where `an` is a preposition, one for the others.
    static RENAMED: RefCell<[HashMap<String, &'static str>; 2]> = RefCell::new([HashMap::new(), HashMap::new()]);
}

/// `text` as this build shows it in `lang`. Text without a name (almost all) comes back as it is.
pub fn rename<'a>(lang: &str, text: &'a str) -> &'a str {
    if !carries_name(text) {
        return text;
    }
    let german = lang == "de";
    RENAMED.with(|tables| -> &'static str {
        let mut tables = tables.borrow_mut();
        let table = &mut tables[usize::from(german)];
        if let Some(&shown) = table.get(text) {
            return shown;
        }
        let shown: &'static str = Box::leak(with_shown_names(text, german).into_boxed_str());
        table.insert(text.to_string(), shown);
        shown
    })
}

/// Every call of `tr` comes here: almost all texts leave on the two searches.
fn carries_name(text: &str) -> bool {
    (text.contains("vo") || text.contains("Upstream")) && text.char_indices().any(|(at, _)| name_at(&text[at..]).is_some())
}

/// The name that starts `text`, if one does: its length and what shows instead. `Evora` and the
/// lower-case `.evodesign` of a file name are not names: the word after `Evo` must be capitalised.
fn name_at(text: &str) -> Option<(usize, &'static str)> {
    for (name, shown) in OTHER_NAMES {
        if text.starts_with(name) {
            return Some((name.len(), shown));
        }
    }
    let rest = text.strip_prefix('E')?.strip_prefix('v')?.strip_prefix('o')?;
    let rest = rest.strip_prefix(' ').unwrap_or(rest);
    if !rest.starts_with(|c: char| c.is_ascii_uppercase()) {
        return None;
    }
    let tail = NAME_TAILS.into_iter().find(|tail| rest.get(..tail.len()).is_some_and(|start| start.eq_ignore_ascii_case(tail)))?;
    Some((text.len() - rest.len() + tail.len(), SHOWN_NAME))
}

fn with_shown_names(text: &str, german: bool) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        match name_at(rest) {
            Some((len, shown)) => {
                if shown == SHOWN_NAME && !german {
                    lighten_article(&mut out);
                }
                out.push_str(shown);
                rest = &rest[len..];
            }
            None => {
                out.push(c);
                rest = &rest[c.len_utf8()..];
            }
        }
    }
    out
}

/// The engines' names start with a vowel, the shown name with a consonant: English `an` becomes `a`.
fn lighten_article(out: &mut String) {
    if (out.ends_with("an ") || out.ends_with("An ")) && !out[..out.len() - 3].ends_with(char::is_alphanumeric) {
        out.truncate(out.len() - 2);
        out.push(' ');
    }
}

#[cfg(test)]
mod tests {
    use super::{SHOWN_NAME, rename};

    #[test]
    fn an_engine_name_shows_as_the_application() {
        for name in ["EvoDesign", "EvoFilm", "EvoVector", "EvoPhoto", "EvoLight", "EvoEffect", "EvoPdf", "Evo Vector", "EvoPDF"] {
            let about = format!("About {name}");
            assert_eq!(rename("", &about), format!("About {SHOWN_NAME}"), "{name}");
        }
        // A declined form keeps its ending rather than the engine's name.
        assert_eq!(rename("cs", "Vítejte v EvoDesignu"), "Vítejte v Cicerou");
    }

    #[test]
    fn a_colour_profile_shows_under_the_application() {
        assert_eq!(rename("", "EvoPhoto Coated CMYK (synthetic, 300% TAC, medium GCR)"), "Cicero Coated CMYK (synthetic, 300% TAC, medium GCR)");
        assert_eq!(rename("", "Adobe RGB (1998)-compatible (EvoPhoto)"), "Adobe RGB (1998)-compatible (Cicero)");
        assert_eq!(rename("", "Coated FOGRA39 (fitted, evo-cms)"), "Coated FOGRA39 (fitted, Cicero)");
        assert_eq!(rename("", "Cicero Generic CMYK (SWOP-like)"), "Cicero Generic CMYK (SWOP-like)");
    }

    #[test]
    fn the_english_article_follows_the_shown_name() {
        assert_eq!(rename("", "Open an EvoDesign file"), "Open a Cicero file");
        assert_eq!(rename("", "An EvoDesign document"), "A Cicero document");
        assert_eq!(rename("", "(an EvoPhoto profile)"), "(a Cicero profile)");
        // Not an article: a longer word, or the German preposition.
        assert_eq!(rename("", "Man EvoDesign"), "Man Cicero");
        assert_eq!(rename("de", "Feedback an EvoDesign senden"), "Feedback an Cicero senden");
        // Evora starts with a vowel too.
        assert_eq!(rename("", "an Upstream app"), "an Evora app");
    }

    #[test]
    fn the_upstream_team_and_its_tools_read_neutrally() {
        assert_eq!(rename("", "Thanks to the Upstream Team"), "Thanks to the Evora team");
        assert_eq!(rename("", "Join the Upstream Discord"), "Join the Evora Discord");
        assert_eq!(rename("", "More Evo Apps"), "More Evora apps");
        assert_eq!(rename("", "The Evo apps' importers"), "The Evora apps' importers");
        assert_eq!(rename("", "See evorules"), "See Evora");
    }

    #[test]
    fn what_is_not_a_name_stays_as_it_is() {
        for text in ["File", "Evora Website…", "Favorites", "Title.evodesign", "evodesign_canvas", "EVODESIGN_DEBUG_LAYOUT", "Evolve", "upstream"] {
            assert!(std::ptr::eq(rename("", text), text), "{text}");
        }
    }

    #[test]
    fn a_renamed_text_is_made_once() {
        assert!(std::ptr::eq(rename("", "EvoDesign Project"), rename("", "EvoDesign Project")));
    }

    #[test]
    fn the_tables_answer_with_the_shown_name() {
        assert_eq!(crate::i18n::tr("", "Quit EvoDesign"), "Quit Cicero");
        assert_eq!(crate::i18n::tr("fr", "Quit EvoDesign"), "Quit Cicero");
        assert_eq!(crate::i18n::tr_context("ru", "About EvoDesign", "count"), "About Cicero");
        assert_eq!(crate::i18n::rename("", "EvoPhoto Coated CMYK"), "Cicero Coated CMYK");
    }
}
