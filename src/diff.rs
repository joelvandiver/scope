use similar::{ChangeTag, TextDiff};

/// A single span within a changed line, indicating whether this run of
/// characters is new relative to the previous output.
#[derive(Debug, Clone)]
pub struct DiffSpan {
    pub changed: bool,
    pub content: String,
}

/// A line to display. Only lines present in the *current* output are
/// represented — removed lines are not shown.
#[derive(Debug, Clone)]
pub enum DiffLine {
    /// Identical to the previous run.
    Same(String),
    /// Present in the current run, with character-level spans showing what
    /// changed relative to the previous run. If the line is entirely new
    /// (no old counterpart), one span with `changed: true` covers the whole line.
    Changed(Vec<DiffSpan>),
}

pub fn compute(old: &str, new: &str) -> Vec<DiffLine> {
    let diff = TextDiff::from_lines(old, new);
    let all_changes: Vec<_> = diff.iter_all_changes().collect();

    let mut result = Vec::new();
    let mut i = 0;

    while i < all_changes.len() {
        match all_changes[i].tag() {
            ChangeTag::Equal => {
                let content = all_changes[i].value().trim_end_matches('\n').to_string();
                result.push(DiffLine::Same(content));
                i += 1;
            }
            ChangeTag::Delete | ChangeTag::Insert => {
                // Collect the full run of removes and inserts before the next Equal.
                let mut removed: Vec<String> = Vec::new();
                let mut added: Vec<String> = Vec::new();

                while i < all_changes.len() {
                    match all_changes[i].tag() {
                        ChangeTag::Delete => {
                            removed.push(all_changes[i].value().trim_end_matches('\n').to_string());
                            i += 1;
                        }
                        ChangeTag::Insert => {
                            added.push(all_changes[i].value().trim_end_matches('\n').to_string());
                            i += 1;
                        }
                        ChangeTag::Equal => break,
                    }
                }

                // Pair removed+added lines for character-level inline diffs.
                let pairs = removed.len().min(added.len());
                for j in 0..pairs {
                    result.push(DiffLine::Changed(char_diff(&removed[j], &added[j])));
                }

                // Purely added lines (no old counterpart): mark entirely as changed.
                for a in &added[pairs..] {
                    result.push(DiffLine::Changed(vec![DiffSpan {
                        changed: true,
                        content: a.clone(),
                    }]));
                }

                // Purely removed lines are not shown — we only display current output.
            }
        }
    }

    result
}

/// Compute character-level diff between two strings, returning spans for the
/// *new* line only. Deleted characters are skipped; inserted and equal
/// characters are grouped into spans tagged as changed or unchanged.
fn char_diff(old: &str, new: &str) -> Vec<DiffSpan> {
    let diff = TextDiff::from_chars(old, new);
    let mut spans: Vec<DiffSpan> = Vec::new();

    for change in diff.iter_all_changes() {
        let changed = change.tag() == ChangeTag::Insert;
        if change.tag() == ChangeTag::Delete {
            continue;
        }
        let ch = change.value();
        if let Some(last) = spans.last_mut() {
            if last.changed == changed {
                last.content.push_str(ch);
                continue;
            }
        }
        spans.push(DiffSpan {
            changed,
            content: ch.to_string(),
        });
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_change() {
        let result = compute("a\nb\nc\n", "a\nb\nc\n");
        assert!(result.iter().all(|l| matches!(l, DiffLine::Same(_))));
        let contents: Vec<_> = result
            .iter()
            .map(|l| match l {
                DiffLine::Same(s) => s.as_str(),
                _ => "",
            })
            .collect();
        assert_eq!(contents, vec!["a", "b", "c"]);
    }

    #[test]
    fn empty_to_content() {
        let result = compute("", "a\nb\nc\n");
        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|l| matches!(l, DiffLine::Changed(_))));
    }

    #[test]
    fn content_to_empty() {
        // Removed lines are not shown — result should be empty.
        let result = compute("a\nb\nc\n", "");
        assert!(result.is_empty());
    }

    #[test]
    fn partial_change() {
        let result = compute("a\nb\nc\n", "a\nB\nc\n");
        assert!(matches!(&result[0], DiffLine::Same(s) if s == "a"));
        assert!(matches!(&result[1], DiffLine::Changed(_)));
        assert!(matches!(&result[2], DiffLine::Same(s) if s == "c"));
    }

    #[test]
    fn inline_changed_chars() {
        // "count: 100" → "count: 200": only "1"→"2" should be marked changed.
        let result = compute("count: 100\n", "count: 200\n");
        if let DiffLine::Changed(spans) = &result[0] {
            let unchanged: String = spans
                .iter()
                .filter(|s| !s.changed)
                .map(|s| s.content.as_str())
                .collect();
            let changed: String = spans
                .iter()
                .filter(|s| s.changed)
                .map(|s| s.content.as_str())
                .collect();
            assert_eq!(unchanged, "count: 00");
            assert_eq!(changed, "2");
        } else {
            panic!("expected Changed line");
        }
    }

    #[test]
    fn both_empty() {
        assert!(compute("", "").is_empty());
    }

    #[test]
    fn no_trailing_newline() {
        let result = compute("a\nb", "a\nb");
        assert!(result.iter().all(|l| matches!(l, DiffLine::Same(_))));
    }
}
