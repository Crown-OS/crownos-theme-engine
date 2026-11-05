//! Sets `key=value` pairs in one section of an INI file, keeping every other
//! line, comment and section untouched.

use std::borrow::Cow;
use std::mem;

pub fn set_keys<'a>(text: &'a str, section: &str, entries: &[(&str, &str)]) -> Cow<'a, str> {
    let mut written = vec![false; entries.len()];
    let mut out = String::with_capacity(text.len() + 64);
    let mut in_section = false;
    let mut found_section = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            if in_section {
                write_missing(&mut out, entries, &mut written);
            }
            in_section = name == section;
            found_section |= in_section;
        } else if in_section
            && let Some(index) = key_of(trimmed)
                .and_then(|key| entries.iter().position(|(entry_key, _)| *entry_key == key))
        {
            if !mem::replace(&mut written[index], true) {
                push_entry(&mut out, entries[index]);
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    if !found_section {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push('[');
        out.push_str(section);
        out.push_str("]\n");
    }
    write_missing(&mut out, entries, &mut written);
    if out == text {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(out)
    }
}

fn key_of(line: &str) -> Option<&str> {
    if line.starts_with(['#', ';']) {
        return None;
    }
    line.split_once('=').map(|(key, _)| key.trim())
}

fn push_entry(out: &mut String, (key, value): (&str, &str)) {
    out.push_str(key);
    out.push('=');
    out.push_str(value);
    out.push('\n');
}

/// Inserts the entries not seen yet after the last non-blank line written.
fn write_missing(out: &mut String, entries: &[(&str, &str)], written: &mut [bool]) {
    if written.iter().all(|done| *done) {
        return;
    }
    let content_end = (out.trim_end().len() + 1).min(out.len());
    let blank_tail = out.split_off(content_end);
    for (entry, done) in entries.iter().zip(written.iter_mut()) {
        if !mem::replace(done, true) {
            push_entry(out, *entry);
        }
    }
    out.push_str(&blank_tail);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_existing_and_appends_missing_keys_inside_the_section() {
        let text = "[Settings]\ngtk-theme-name = Old\n; note\n\n[Other]\ngtk-theme-name=Keep\n";
        let updated = set_keys(
            text,
            "Settings",
            &[("gtk-theme-name", "New"), ("dark", "1")],
        );
        assert_eq!(
            updated,
            "[Settings]\ngtk-theme-name=New\n; note\ndark=1\n\n[Other]\ngtk-theme-name=Keep\n"
        );
        assert!(matches!(
            set_keys(&updated, "Settings", &[("dark", "1")]),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn creates_the_section_and_drops_duplicate_keys() {
        assert_eq!(
            set_keys("", "Appearance", &[("a", "1")]),
            "[Appearance]\na=1\n"
        );
        assert_eq!(
            set_keys("[S]\na=1\na=2\n", "S", &[("a", "3")]),
            "[S]\na=3\n"
        );
        assert_eq!(set_keys("[X]\n", "S", &[("a", "3")]), "[X]\n\n[S]\na=3\n");
    }
}
