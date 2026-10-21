//! Small readers that the other modules share.

use std::{collections::BTreeMap, path::PathBuf};

pub fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

pub fn strip_json_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_string = false;
    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if c == '\\' {
                out.extend(chars.next());
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
            out.push(c);
        } else if c == '/' && chars.peek() == Some(&'/') {
            for next in chars.by_ref() {
                if next == '\n' {
                    out.push('\n');
                    break;
                }
            }
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut prev = ' ';
            for next in chars.by_ref() {
                if prev == '*' && next == '/' {
                    break;
                }
                prev = next;
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn frontmatter(text: &str) -> BTreeMap<String, String> {
    let mut meta = BTreeMap::new();
    let mut lines = text.lines();
    if lines.next().map(str::trim_end) != Some("---") {
        return meta;
    }
    let mut key: Option<String> = None;
    for line in lines {
        if line.trim_end() == "---" {
            break;
        }
        let indented = line.starts_with([' ', '\t']);
        match line.split_once(':') {
            Some((name, value)) if !indented && is_key(name) => {
                let value = value.trim();
                let value = if matches!(value, ">" | "|" | ">-" | "|-" | ">+" | "|+") {
                    ""
                } else {
                    unquote(value)
                };
                meta.insert(name.to_string(), value.to_string());
                key = Some(name.to_string());
            }
            _ if indented && !line.trim().is_empty() => {
                if let Some(value) = key.as_ref().and_then(|key| meta.get_mut(key)) {
                    if !value.is_empty() {
                        value.push(' ');
                    }
                    value.push_str(line.trim());
                }
            }
            _ => {}
        }
    }
    meta
}

fn is_key(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn unquote(value: &str) -> &str {
    for quote in ['"', '\''] {
        if value.len() >= 2 && value.starts_with(quote) && value.ends_with(quote) {
            return &value[1..value.len() - 1];
        }
    }
    value
}
