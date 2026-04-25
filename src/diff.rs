use similar::{ChangeTag, TextDiff};

#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineKind {
    Same,
    Added,
    Removed,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
}

pub fn compute(old: &str, new: &str) -> Vec<DiffLine> {
    let diff = TextDiff::from_lines(old, new);
    diff.iter_all_changes()
        .map(|change| {
            let kind = match change.tag() {
                ChangeTag::Equal => DiffLineKind::Same,
                ChangeTag::Insert => DiffLineKind::Added,
                ChangeTag::Delete => DiffLineKind::Removed,
            };
            DiffLine {
                kind,
                content: change.value().trim_end_matches('\n').to_string(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(lines: &[DiffLine]) -> Vec<DiffLineKind> {
        lines.iter().map(|l| l.kind.clone()).collect()
    }

    fn contents(lines: &[DiffLine]) -> Vec<&str> {
        lines.iter().map(|l| l.content.as_str()).collect()
    }

    #[test]
    fn empty_to_content() {
        let result = compute("", "a\nb\nc\n");
        assert_eq!(
            kinds(&result),
            vec![DiffLineKind::Added, DiffLineKind::Added, DiffLineKind::Added]
        );
        assert_eq!(contents(&result), vec!["a", "b", "c"]);
    }

    #[test]
    fn content_to_empty() {
        let result = compute("a\nb\nc\n", "");
        assert_eq!(
            kinds(&result),
            vec![
                DiffLineKind::Removed,
                DiffLineKind::Removed,
                DiffLineKind::Removed
            ]
        );
        assert_eq!(contents(&result), vec!["a", "b", "c"]);
    }

    #[test]
    fn no_change() {
        let result = compute("a\nb\nc\n", "a\nb\nc\n");
        assert!(kinds(&result).iter().all(|k| *k == DiffLineKind::Same));
        assert_eq!(contents(&result), vec!["a", "b", "c"]);
    }

    #[test]
    fn all_changed() {
        let result = compute("a\nb\nc\n", "x\ny\nz\n");
        assert!(kinds(&result)
            .iter()
            .all(|k| *k == DiffLineKind::Removed || *k == DiffLineKind::Added));
    }

    #[test]
    fn partial_change() {
        let result = compute("a\nb\nc\n", "a\nB\nc\n");
        assert_eq!(result[0].kind, DiffLineKind::Same);
        assert_eq!(result[0].content, "a");
        assert!(result.iter().any(|l| l.kind == DiffLineKind::Removed && l.content == "b"));
        assert!(result.iter().any(|l| l.kind == DiffLineKind::Added && l.content == "B"));
        assert_eq!(result.last().unwrap().kind, DiffLineKind::Same);
        assert_eq!(result.last().unwrap().content, "c");
    }

    #[test]
    fn both_empty() {
        let result = compute("", "");
        assert!(result.is_empty());
    }

    #[test]
    fn no_trailing_newline() {
        let result = compute("a\nb", "a\nb");
        assert!(kinds(&result).iter().all(|k| *k == DiffLineKind::Same));
        assert_eq!(contents(&result), vec!["a", "b"]);
    }
}
