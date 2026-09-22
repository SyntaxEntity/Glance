use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    pub id: String,
    pub level: u8,
    pub text: String,
    pub line: usize,
}

/// Extract ATX headings (`#` … `######`) and build stable anchors.
pub fn extract_headings(markdown: &str) -> Vec<Heading> {
    let mut out = Vec::new();
    let mut in_fence = false;
    let mut fence_marker = String::new();

    for (idx, line) in markdown.lines().enumerate() {
        let trimmed = line.trim_end();

        let t = trimmed.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            let marker: String = t.chars().take_while(|c| *c == '`' || *c == '~').collect();
            if !in_fence {
                in_fence = true;
                fence_marker = marker;
            } else if marker.starts_with(&fence_marker) && fence_marker.len() <= marker.len() {
                in_fence = false;
                fence_marker.clear();
            }
            continue;
        }
        if in_fence {
            continue;
        }

        if trimmed.starts_with("    ") {
            continue;
        }

        let bytes = trimmed.as_bytes();
        if bytes.first() != Some(&b'#') {
            continue;
        }

        let level = bytes.iter().take_while(|b| **b == b'#').count();
        if level == 0 || level > 6 {
            continue;
        }

        let rest = &trimmed[level..];
        if !rest.is_empty() && !rest.starts_with(' ') && !rest.starts_with('\t') {
            continue;
        }

        let text = rest.trim().trim_end_matches('#').trim().to_string();
        if text.is_empty() {
            continue;
        }

        let id = unique_id(&slugify(&text), &out);
        out.push(Heading {
            id,
            level: level as u8,
            text,
            line: idx,
        });
    }

    out
}

fn slugify(text: &str) -> String {
    let mut s = String::with_capacity(text.len());
    let mut last_dash = true;
    for ch in text.trim().to_lowercase().chars() {
        if ch.is_alphanumeric() {
            s.push(ch);
            last_dash = false;
        } else if !last_dash {
            s.push('-');
            last_dash = true;
        }
    }
    while s.ends_with('-') {
        s.pop();
    }
    if s.is_empty() {
        "sec".into()
    } else {
        s.chars().take(80).collect()
    }
}

fn unique_id(base: &str, existing: &[Heading]) -> String {
    if !existing.iter().any(|h| h.id == base) {
        return base.to_string();
    }
    for n in 1.. {
        let candidate = format!("{base}-{n}");
        if !existing.iter().any(|h| h.id == candidate) {
            return candidate;
        }
    }
    base.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_nested_and_skips_fences() {
        let md = r#"# Title
Some text
## Section A
```bash
# not a heading
```
### Detail
## Section A
"#;
        let hs = extract_headings(md);
        assert_eq!(hs.len(), 4);
        assert_eq!(hs[0].level, 1);
        assert_eq!(hs[0].id, "title");
        assert_eq!(hs[2].text, "Detail");
        assert_eq!(hs[3].id, "section-a-1");
    }
}
