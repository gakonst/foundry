use solang_parser::pt::{Comment, Statement};
use std::borrow::Cow;

/// Extension trait for [`Statement`].
pub trait StatementExt {
    /// Flattens the statement if it is a block.
    fn flattened(self) -> Self;
}

impl StatementExt for Statement {
    fn flattened(self) -> Self {
        match self {
            Statement::Block { loc, unchecked, mut statements } => {
                if statements.len() == 1 {
                    statements.pop().unwrap().flattened()
                } else {
                    Statement::Block { loc, unchecked, statements }
                }
            }
            _ => self,
        }
    }
}

/// Extension trait for [`Comment`].
pub trait CommentExt {
    /// Returns the comment's value without the comment
    fn clean_value(&self) -> Cow<'_, str>;
}

impl CommentExt for Comment {
    fn clean_value(&self) -> Cow<'_, str> {
        fn line_comment_value(comment: &str, offset: usize) -> Cow<'_, str> {
            let leading = comment.find(|c: char| c != '/' && !c.is_whitespace()).unwrap_or(offset);
            let comment = comment[leading..].trim_end();
            Cow::Borrowed(comment)
        }

        fn block_comment_value(comment: &str, offset: usize) -> Cow<'_, str> {
            // remove the leading /** and tailing */
            let mut grouped_comments = Vec::new();
            let len = comment.len();
            for s in comment[offset..len - offset - 1].lines() {
                if let Some((i, _)) =
                    s.char_indices().find(|(_, ch)| !ch.is_whitespace() && *ch != '*')
                {
                    grouped_comments.push(s[i..].trim_end());
                }
            }
            Cow::Owned(grouped_comments.join("\n"))
        }

        match self {
            Comment::Line(_, comment) => line_comment_value(comment, 2),
            Comment::Block(_, comment) => block_comment_value(comment, 2),
            Comment::DocLine(_, comment) => line_comment_value(comment, 3),
            Comment::DocBlock(_, comment) => block_comment_value(comment, 3),
        }
    }
}
