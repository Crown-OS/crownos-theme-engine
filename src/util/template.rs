//! `{{variable}}` substitution in a single pass, without intermediate strings.

use crate::error::TemplateError;

/// `write_variable` appends the value of a trimmed variable name and returns
/// `false` when it does not know the name.
pub fn render(
    template: &str,
    mut write_variable: impl FnMut(&str, &mut String) -> bool,
) -> Result<String, TemplateError> {
    let mut out = String::with_capacity(template.len() + template.len() / 4);
    let mut rest = template;
    while let Some(open) = rest.find("{{") {
        out.push_str(&rest[..open]);
        let inner = &rest[open + 2..];
        let close = inner
            .find("}}")
            .ok_or(TemplateError::Unclosed(template.len() - rest.len() + open))?;
        let name = inner[..close].trim();
        if !write_variable(name, &mut out) {
            return Err(TemplateError::UnknownVariable(name.to_owned()));
        }
        rest = &inner[close + 2..];
    }
    out.push_str(rest);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn greet(name: &str, out: &mut String) -> bool {
        (name == "who").then(|| out.push_str("world")).is_some()
    }

    #[test]
    fn substitutes_and_keeps_surrounding_text() {
        let rendered = render("hello {{ who }}, {single} {{who}}!", greet).expect("valid");
        assert_eq!(rendered, "hello world, {single} world!");
    }

    #[test]
    fn reports_unknown_and_unclosed_placeholders() {
        assert!(matches!(
            render("{{nobody}}", greet),
            Err(TemplateError::UnknownVariable(name)) if name == "nobody"
        ));
        assert!(matches!(
            render("ab {{who", greet),
            Err(TemplateError::Unclosed(3))
        ));
    }
}
