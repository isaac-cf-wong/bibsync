//! Field-level access to the body of a BibTeX entry.
//!
//! Entries are kept as text so that everything bibsync does not touch is
//! written back exactly as the provider or the user wrote it. These helpers
//! locate one field's value inside that text and splice a replacement in,
//! leaving the rest of the body untouched.

use std::ops::Range;

/// One `name = value` field of an entry body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Field {
    /// Field name, lower-cased.
    pub(crate) name: String,
    /// Byte range of the whole field, from its name to the end of its value.
    pub(crate) span: Range<usize>,
    /// Byte range of the value, including its delimiters.
    pub(crate) value_span: Range<usize>,
}

impl Field {
    /// The value without its outer `"..."` or `{...}` delimiters.
    ///
    /// Returns `None` for a bare or concatenated (`#`) value.
    pub(crate) fn delimited_text<'a>(&self, body: &'a str) -> Option<&'a str> {
        let raw = &body[self.value_span.clone()];
        let inner = raw
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix('"'))
            .or_else(|| {
                raw.strip_prefix('{')
                    .and_then(|rest| rest.strip_suffix('}'))
            })?;
        // A value such as `{a} # {b}` also starts and ends with a brace.
        (value_end(raw.as_bytes(), 0) == Some(raw.len())).then_some(inner)
    }
}

/// Parse the fields of an entry body (the text after `key,`).
///
/// Returns `None` when the body does not follow the `name = value` grammar;
/// callers then leave the entry unchanged.
pub(crate) fn parse_fields(body: &str) -> Option<Vec<Field>> {
    let bytes = body.as_bytes();
    let mut fields = Vec::new();
    let mut index = skip_separators(bytes, 0);
    while index < bytes.len() {
        let name_start = index;
        while index < bytes.len() && is_name_byte(bytes[index]) {
            index += 1;
        }
        if index == name_start {
            return None;
        }
        let name = body[name_start..index].to_ascii_lowercase();
        index = skip_whitespace(bytes, index);
        if bytes.get(index) != Some(&b'=') {
            return None;
        }
        index = skip_whitespace(bytes, index + 1);
        let value_start = index;
        let mut value_stop = value_end(bytes, index)?;
        // Concatenations: `"a" # macro # {b}`.
        loop {
            let next = skip_whitespace(bytes, value_stop);
            if bytes.get(next) != Some(&b'#') {
                break;
            }
            value_stop = value_end(bytes, skip_whitespace(bytes, next + 1))?;
        }
        fields.push(Field {
            name,
            span: name_start..value_stop,
            value_span: value_start..value_stop,
        });
        index = skip_whitespace(bytes, value_stop);
        match bytes.get(index) {
            None => break,
            Some(b',') => index = skip_separators(bytes, index),
            Some(_) => return None,
        }
    }
    Some(fields)
}

/// Return the first field called `name` (case-insensitive).
pub(crate) fn find_field(body: &str, name: &str) -> Option<Field> {
    parse_fields(body)?
        .into_iter()
        .find(|field| field.name.eq_ignore_ascii_case(name))
}

/// Replace the text inside `field`'s delimiters, keeping the delimiter style
/// unless the new text needs braces to stay a single quoted value.
pub(crate) fn replace_delimited_text(body: &str, field: &Field, text: &str) -> String {
    let raw = &body[field.value_span.clone()];
    let value = if raw.starts_with('"') && !text.contains('"') {
        format!("\"{text}\"")
    } else {
        format!("{{{text}}}")
    };
    let mut updated = String::with_capacity(body.len() + value.len());
    updated.push_str(&body[..field.value_span.start]);
    updated.push_str(&value);
    updated.push_str(&body[field.value_span.end..]);
    updated
}

/// Remove `field` from the body together with the separator that follows it
/// (or, for the last field, the one before it).
pub(crate) fn remove_field(body: &str, field: &Field) -> String {
    let bytes = body.as_bytes();
    let after = skip_whitespace(bytes, field.span.end);
    let (start, end) = if bytes.get(after) == Some(&b',') {
        (field.span.start, skip_whitespace(bytes, after + 1))
    } else {
        // Last field: drop the preceding comma instead, if any.
        let before = body[..field.span.start].trim_end();
        let start = before.strip_suffix(',').map_or(field.span.start, str::len);
        (start, field.span.end)
    };
    let mut updated = String::with_capacity(body.len());
    updated.push_str(&body[..start]);
    updated.push_str(&body[end..]);
    updated
}

/// Split `text` on commas that are not inside braces.
pub(crate) fn split_top_level(text: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0_i32;
    let mut start = 0;
    for (offset, ch) in text.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ if ch == separator && depth == 0 => {
                parts.push(&text[start..offset]);
                start = offset + ch.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&text[start..]);
    parts
}

fn is_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':' | b'.' | b'+')
}

fn skip_whitespace(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    index
}

fn skip_separators(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && (bytes[index].is_ascii_whitespace() || bytes[index] == b',') {
        index += 1;
    }
    index
}

/// End (exclusive) of the single value token starting at `start`.
fn value_end(bytes: &[u8], start: usize) -> Option<usize> {
    match bytes.get(start)? {
        b'{' => {
            let mut depth = 0_i32;
            for (offset, byte) in bytes[start..].iter().enumerate() {
                match byte {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            return Some(start + offset + 1);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        b'"' => {
            let mut depth = 0_i32;
            let mut index = start + 1;
            while index < bytes.len() {
                match bytes[index] {
                    // A backslash escapes the next byte, so `\"` (an accent
                    // such as `J\"{u}rg`) is content and `\\"` still closes.
                    b'\\' => index += 1,
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    b'"' if depth == 0 => return Some(index + 1),
                    _ => {}
                }
                index += 1;
            }
            None
        }
        _ => {
            let mut index = start;
            while index < bytes.len()
                && !bytes[index].is_ascii_whitespace()
                && !matches!(bytes[index], b',' | b'#' | b'}' | b'"' | b'{')
            {
                index += 1;
            }
            (index > start).then_some(index)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{find_field, parse_fields, remove_field, replace_delimited_text, split_top_level};

    const BODY: &str = "author = \"Abac, A. G. and others\",\n    collaboration = \"LIGO Scientific, VIRGO, Virgo,, KAGRA\",\n    title = \"{GWTC-4.0: Population Properties}\",\n    month = jan # \"~1\",\n    year = 2026";

    #[test]
    fn parses_quoted_braced_bare_and_concatenated_values() {
        let fields = parse_fields(BODY).expect("parses");
        let names = fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>();
        assert_eq!(names, ["author", "collaboration", "title", "month", "year"]);
        assert_eq!(&BODY[fields[3].value_span.clone()], "jan # \"~1\"");
        assert_eq!(fields[3].delimited_text(BODY), None);
        assert_eq!(&BODY[fields[4].value_span.clone()], "2026");
        assert_eq!(
            fields[2].delimited_text(BODY),
            Some("{GWTC-4.0: Population Properties}")
        );
    }

    #[test]
    fn backslash_escapes_inside_quoted_values() {
        let body = "title = \"J\\\"{u}rg\", note = \"path\\\\\", collaboration = \"LIGO,, Virgo\"";
        let fields = parse_fields(body).expect("parses");
        let names = fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>();
        assert_eq!(names, ["title", "note", "collaboration"]);
        assert_eq!(fields[0].delimited_text(body), Some("J\\\"{u}rg"));
        assert_eq!(fields[1].delimited_text(body), Some("path\\\\"));
        assert_eq!(fields[2].delimited_text(body), Some("LIGO,, Virgo"));
    }

    #[test]
    fn rejects_malformed_bodies() {
        assert_eq!(parse_fields("author \"x\""), None);
        assert_eq!(parse_fields("author = {unclosed"), None);
        assert_eq!(parse_fields("author = \"a\" title = \"b\""), None);
    }

    #[test]
    fn concatenated_braced_value_is_not_delimited_text() {
        let body = "title = {a} # {b}";
        let field = find_field(body, "title").expect("field");
        assert_eq!(field.delimited_text(body), None);
    }

    #[test]
    fn replaces_value_and_keeps_rest_of_body() {
        let field = find_field(BODY, "COLLABORATION").expect("field");
        let updated = replace_delimited_text(BODY, &field, "LIGO Scientific, KAGRA");
        assert_eq!(
            updated,
            BODY.replace(
                "\"LIGO Scientific, VIRGO, Virgo,, KAGRA\"",
                "\"LIGO Scientific, KAGRA\""
            )
        );
        let braced = replace_delimited_text(BODY, &field, "say \"hi\"");
        assert!(braced.contains("collaboration = {say \"hi\"},"));
    }

    #[test]
    fn removes_middle_and_last_fields() {
        let field = find_field(BODY, "collaboration").expect("field");
        let updated = remove_field(BODY, &field);
        assert!(!updated.contains("collaboration"));
        assert!(updated.starts_with("author = \"Abac, A. G. and others\",\n    title ="));

        let last = find_field(BODY, "year").expect("field");
        let updated = remove_field(BODY, &last);
        assert!(updated.ends_with("month = jan # \"~1\""));
    }

    #[test]
    fn splits_only_top_level_commas() {
        assert_eq!(
            split_top_level("{A, B}, C,, D", ','),
            ["{A, B}", " C", "", " D"]
        );
    }
}
