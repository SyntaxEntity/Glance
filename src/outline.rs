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

    #[test]
    fn ignores_headings_inside_tilde_fence() {
        let md = "# Real\n~~~\n# fake\n~~~\n## Also real\n";
        let hs = extract_headings(md);
        assert_eq!(hs.len(), 2);
        assert_eq!(hs[0].id, "real");
        assert_eq!(hs[1].text, "Also real");
    }

    #[test]
    fn requires_space_after_hashes() {
        // `#NoSpace` is not an ATX heading; `#` alone has no text either.
        let hs = extract_headings("#NoSpace\n#\n##\n# Valid\n");
        assert_eq!(hs.len(), 1);
        assert_eq!(hs[0].text, "Valid");
    }

    #[test]
    fn caps_level_at_six() {
        let hs = extract_headings("####### seven\n###### six\n");
        assert_eq!(hs.len(), 1);
        assert_eq!(hs[0].level, 6);
        assert_eq!(hs[0].text, "six");
    }

    #[test]
    fn strips_closing_hashes() {
        let hs = extract_headings("## Closed ##\n# Trailing #\n");
        assert_eq!(hs[0].text, "Closed");
        assert_eq!(hs[1].text, "Trailing");
    }

    #[test]
    fn keeps_cjk_text_and_slugs() {
        let hs = extract_headings("# 中文标题\n## 第二节\n");
        assert_eq!(hs[0].text, "中文标题");
        assert_eq!(hs[0].id, "中文标题");
        assert_eq!(hs[1].id, "第二节");
    }

    #[test]
    fn slug_collapses_punctuation_and_falls_back() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("  multiple   spaces  "), "multiple-spaces");
        assert_eq!(slugify("a---b"), "a-b");
        assert_eq!(slugify("!!!"), "sec");
        assert_eq!(slugify(""), "sec");
        // 80-char cap applies to the slug, not the source text.
        assert_eq!(slugify(&"x".repeat(200)).len(), 80);
    }

    #[test]
    fn line_numbers_are_zero_based_and_accurate() {
        let hs = extract_headings("intro\n\n# One\n\n## Two\n");
        assert_eq!(hs[0].line, 2);
        assert_eq!(hs[1].line, 4);
    }

    #[test]
    fn same_text_at_different_levels_gets_unique_ids() {
        let hs = extract_headings("# Note\n## Note\n### Note\n");
        assert_eq!(hs[0].id, "note");
        assert_eq!(hs[1].id, "note-1");
        assert_eq!(hs[2].id, "note-2");
    }

    #[test]
    fn empty_and_punctuation_only_headings_are_skipped() {
        let hs = extract_headings("# \n##   \n### !!!\n# Kept\n");
        // `### !!!` has text, so it is kept with the "sec" fallback slug.
        assert_eq!(hs.len(), 2);
        assert_eq!(hs[0].id, "sec");
        assert_eq!(hs[1].id, "kept");
    }
}
