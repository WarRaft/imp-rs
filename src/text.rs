//! The text form of an import list: BNI, a `[imp]` section with the
//! version and a `file=` line per file, in order: its flags (version 1)
//! and its path, quoted.
//!
//! ```ini
//! [imp]
//! version=1
//! file=13,"war3mapImported\Icon.blp"
//! file=10,"Units\Human\Footman\Footman.mdx"
//! ```
//!
//! Version 0 has no flags: `file="Icon.blp"`. The flags' names and where
//! a file is in the map are for an editor to show.

use bni_rs::Document;
use bni_rs::ast::Item;
use bni_rs::string::{quote, unquote};
use parser_rs::{ParseError, Span};

use crate::{Import, Imports};

/// The section's name.
pub const HEADER: &str = "imp";

/// The text form of `list`.
pub fn to_text(list: &Imports) -> String {
    let mut out = format!("[{HEADER}]\nversion={}\n", list.version);
    for f in &list.files {
        if list.version == 1 {
            out += &format!("file={},{}\n", f.flags, quote(&f.path));
        } else {
            out += &format!("file={}\n", quote(&f.path));
        }
    }
    out
}

/// Is `doc` an import list's text form: is its first section `[imp]`?
pub fn is_import_text(doc: &Document) -> bool {
    doc.sections.first().is_some_and(|s| s.name.text == HEADER)
}

/// What [`from_text`] read: the list when the text has no errors, and
/// every error, in order.
#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    pub list: Option<Imports>,
    pub errors: Vec<ParseError>,
}

pub fn from_text(text: &str) -> Parsed {
    from_bni(&bni_rs::parse(text))
}

/// [`from_text`] of a document already parsed. Its BNI errors come first.
pub fn from_bni(doc: &Document) -> Parsed {
    let mut errors = doc.errors.clone();
    let mut list = Imports { version: 1, files: Vec::new() };
    let Some(section) = doc.sections.first().filter(|s| s.name.text == HEADER) else {
        let span = doc.sections.first().map_or(Span::new(0, 0, 0, 1), |s| s.name.span);
        errors.push(error(span, format!("the file starts with [{HEADER}]")));
        return Parsed { list: None, errors };
    };
    // Errors with a place of their own (a string's), added after.
    let mut placed = Vec::new();
    let mut err = |item: &Item, m: String| errors.push(error(item_span(item), m));
    for item in doc.preamble.iter().chain(doc.sections[1..].iter().flat_map(|s| &s.items)) {
        err(item, format!("only [{HEADER}]: version and file lines"));
    }
    let mut files = Vec::new();
    for item in &section.items {
        match item.key.as_ref().map(|k| k.text.trim()) {
            Some("version") => match item.value_text().trim() {
                "0" => list.version = 0,
                "1" => list.version = 1,
                v => err(item, format!("unknown version {v}: 0 (Reign of Chaos) or 1 (The Frozen Throne)")),
            },
            Some("file") => files.push(item),
            _ => err(item, "a line is `version=` or `file=`".into()),
        }
    }
    for item in files {
        let fields = item.fields.as_slice();
        let (flags, path) = match (list.version, fields) {
            (1, [flags, path]) => (Some(flags), path),
            (0, [path]) => (None, path),
            (1, _) => {
                err(item, "`file=flags,\"path\"` in version 1".into());
                continue;
            }
            _ => {
                err(item, "`file=\"path\"` in version 0: it has no flags".into());
                continue;
            }
        };
        let flags = match flags.map(|f| f.text().trim().parse::<u8>()) {
            None => 0,
            Some(Ok(n)) => n,
            Some(Err(_)) => {
                err(item, "the flags are a number from 0 to 255".into());
                continue;
            }
        };
        match unquote(path) {
            Ok(path) => list.files.push(Import { flags, path }),
            Err(e) => placed.push(e),
        }
    }
    errors.extend(placed);
    errors.sort_by_key(|e| (e.line, e.col));
    let list = errors.is_empty().then_some(list);
    Parsed { list, errors }
}

fn error(span: Span, message: impl Into<String>) -> ParseError {
    let end_col = if span.end_line == span.start_line { span.end_col.max(span.start_col + 1) } else { span.start_col + 1 };
    ParseError { message: message.into(), line: span.start_line, col: span.start_col, end_col }
}

fn item_span(item: &Item) -> Span {
    let start = item.key.as_ref().map_or(item.eq, |k| k.span);
    let end = item.fields.last().map_or(item.eq, |f| f.span);
    Span::new(start.start_line, start.start_col, end.end_line, end.end_col)
}

#[cfg(test)]
#[path = "text.test.rs"]
mod tests;
