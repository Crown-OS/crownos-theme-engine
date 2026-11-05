//! A marked region the engine owns inside a file the user owns.
//!
//! The block is appended once and replaced in place afterwards, so every line
//! outside the markers is left exactly as the user wrote it.

use std::borrow::Cow;

const BEGIN: &str = ">>> crownos-theme-engine >>>";
const END: &str = "<<< crownos-theme-engine <<<";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comment {
    Hash,
    Css,
}

impl Comment {
    fn marker(self, label: &str) -> String {
        match self {
            Self::Hash => format!("# {label}"),
            Self::Css => format!("/* {label} */"),
        }
    }
}

pub fn upsert<'a>(text: &'a str, comment: Comment, body: &str) -> Cow<'a, str> {
    let (begin, end) = (comment.marker(BEGIN), comment.marker(END));
    let block = format!("{begin}\n{}\n{end}\n", body.trim_end());
    let existing = text.find(&begin).and_then(|start| {
        let marker_end = start + text[start..].find(&end)? + end.len();
        let line_end = marker_end + usize::from(text[marker_end..].starts_with('\n'));
        Some(start..line_end)
    });
    match existing {
        Some(range) if text[range.clone()] == block => Cow::Borrowed(text),
        Some(range) => Cow::Owned(format!(
            "{}{block}{}",
            &text[..range.start],
            &text[range.end..]
        )),
        None => {
            let separator = match text {
                "" => "",
                _ if text.ends_with('\n') => "\n",
                _ => "\n\n",
            };
            Cow::Owned(format!("{text}{separator}{block}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_once_then_replaces_in_place() {
        let user = "font_size 12\n";
        let first = upsert(user, Comment::Hash, "include a.conf");
        assert_eq!(
            first,
            "font_size 12\n\n# >>> crownos-theme-engine >>>\ninclude a.conf\n# <<< crownos-theme-engine <<<\n"
        );
        assert!(matches!(
            upsert(&first, Comment::Hash, "include a.conf"),
            Cow::Borrowed(_)
        ));

        let edited = format!("{first}font_family Iosevka\n");
        let second = upsert(&edited, Comment::Hash, "include b.conf");
        assert!(second.starts_with("font_size 12\n\n# >>>"));
        assert!(
            second
                .contains("include b.conf\n# <<< crownos-theme-engine <<<\nfont_family Iosevka\n")
        );
        assert!(!second.contains("a.conf"));
    }

    #[test]
    fn css_markers_are_comments() {
        let css = upsert("", Comment::Css, "@define-color a #000;");
        assert!(css.starts_with("/* >>> crownos-theme-engine >>> */\n"));
    }
}
