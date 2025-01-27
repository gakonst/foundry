//! Transform solidity AST

use crate::solang_ext::{CodeLocationExt, CommentExt, StatementExt};
use solang_parser::pt::{Comment, EventDefinition, Loc, SourceUnitPart, Statement};
use std::fmt;

const SHADOW_PREFIX: &str = "shadow:";

/// A transformer that enables "shadow" elements in the source content of a file.
///
/// First this filters all parsed comments for comments that start with [SHADOW_PREFIX] prefix and
/// transforms them into code. Then [ShadowTransformer::transform] replaces the comments with the
/// transformed code.
#[derive(Debug)]
pub struct ShadowTransformer {
    /// Parsed comments from the source code.
    comments: Vec<TransformedComment>,
    offset: isize,
}

impl ShadowTransformer {
    /// Create a new shadow transformer.
    pub fn new(comments: Vec<Comment>) -> Self {
        // parse into Statements
        let comments: Vec<_> = comments
            .into_iter()
            .filter_map(|comment| TransformedComment::parse_if_prefixed(SHADOW_PREFIX, comment))
            .collect();
        Self { comments, offset: 0 }
    }

    /// Returns `true` if there are no comments to transform.
    pub fn is_empty(&self) -> bool {
        self.comments.is_empty()
    }

    /// Transforms the comments into code statements and injects them into the source code.
    pub fn transform(self, content: &mut String) {
        let Self { comments, mut offset } = self;
        for comment in comments {
            let (start, end) = comment.range_with_offset(offset);
            let statement = comment.value.to_string();
            let comment_len = comment.comment_len();
            offset += statement.len() as isize - comment_len as isize;
            content.replace_range(start..end, &statement);
        }
    }
}

/// A comment that was parsed into a statement.
#[derive(Debug)]
struct TransformedComment {
    comment: Comment,
    value: TransformedValue,
}

impl TransformedComment {
    fn parse_if_prefixed(prefix: &str, comment: Comment) -> Option<Self> {
        if let Some(value) = comment.clean_value().trim_start().strip_prefix(prefix) {
            let value = transform_value(value.trim_start())?;
            Some(Self { comment, value })
        } else {
            None
        }
    }

    /// Returns the range of the comment in the source code adjusted with the offset.
    fn range_with_offset(&self, offset: isize) -> (usize, usize) {
        let start = self.comment.loc().start() as isize + offset;
        let end = self.comment.loc().end() as isize + offset;
        (start as usize, end as usize)
    }

    fn loc(&self) -> Loc {
        self.comment.loc()
    }

    fn comment_len(&self) -> usize {
        self.loc().end() - self.loc().start()
    }
}

#[derive(Debug)]
enum TransformedValue {
    Statement(Statement),
    EventDefinition(Box<EventDefinition>),
}

impl fmt::Display for TransformedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformedValue::Statement(st) => st.fmt(f),
            TransformedValue::EventDefinition(ev) => ev.fmt(f),
        }
    }
}

/// solang-parser only generates parser for source units (file level elements) to keep codegen
/// small.
///
/// If the value is not a source unit that is allowed to appear on the top level, we wrap it into
/// something that forms a valid source unit, for example an `emit` Statement into a function and
/// then extract the parsed statement from that source unit.
fn transform_value(value: &str) -> Option<TransformedValue> {
    if value.starts_with("event ") {
        let (mut unit, _) = solang_parser::parse(value, 0).ok()?;
        return match unit.0.pop()? {
            SourceUnitPart::EventDefinition(ev) => Some(TransformedValue::EventDefinition(ev)),
            _ => None,
        }
    }

    let s = format!("function f() {{{}}}", value);
    let (mut unit, _) = solang_parser::parse(&s, 0).ok()?;
    match unit.0.pop()? {
        SourceUnitPart::FunctionDefinition(f) => {
            // flatten the body to get rid of the block
            Some(TransformedValue::Statement(f.body?.flattened()))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_statement() {
        let s = "emit Foo();";
        let stmt = transform_value(s).unwrap();
        assert_eq!(stmt.to_string(), s);
    }

    #[test]
    fn transform_emit() {
        let mut content = r#"
contract Foo {
    function foo() public {
      // shadow: emit Foo();
    }
}
"#
        .to_string();
        let (_unit, comments) = solang_parser::parse(&content, 0).unwrap();

        let transformer = ShadowTransformer::new(comments);
        transformer.transform(&mut content);
        assert_eq!(
            content,
            r#"
contract Foo {
    function foo() public {
      emit Foo();
    }
}
"#
        );
    }

    #[test]
    fn transform_event_emit() {
        let mut content = r#"
/// shadow: event Foo();
contract Foo {
    function foo() public {
      // shadow: emit Foo();
    }
}
"#
        .to_string();
        let (_unit, comments) = solang_parser::parse(&content, 0).unwrap();

        let transformer = ShadowTransformer::new(comments);
        transformer.transform(&mut content);
        pretty_assertions::assert_eq!(
            content,
            r#"
event Foo();
contract Foo {
    function foo() public {
      emit Foo();
    }
}
"#
        );
    }
}
