use crate::comments::{CommentState, CommentStringExt, Comments};
use itertools::Itertools;
use solang_parser::pt::Loc;
use std::{fmt, str::FromStr};

/// An inline config item
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy)]
pub enum InlineConfigItem {
    /// Disables the next code item regardless of newlines
    DisableNextItem,
    /// Disables formatting between the next newline and the newline after
    DisableNextLine,
    /// Disables formatting for any code that follows this and before the next "disable-end"
    DisableStart,
    /// Disables formatting for any code that precedes this and after the previous "disable-start"
    DisableEnd,
}

impl FromStr for InlineConfigItem {
    type Err = InvalidInlineConfigItem;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "disable-next-item" => InlineConfigItem::DisableNextItem,
            "disable-next-line" => InlineConfigItem::DisableNextLine,
            "disable-start" => InlineConfigItem::DisableStart,
            "disable-end" => InlineConfigItem::DisableEnd,
            s => return Err(InvalidInlineConfigItem(s.into())),
        })
    }
}

#[derive(Debug)]
pub struct InvalidInlineConfigItem(String);

impl fmt::Display for InvalidInlineConfigItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Invalid inline config item: {}", self.0))
    }
}

impl Comments {
    /// Parse all comments to return a list of inline config items. This will return an iterator of
    /// results of parsing comments which start with `forgefmt:`
    pub fn parse_inline_config_items(
        &self,
    ) -> impl Iterator<Item = Result<(Loc, InlineConfigItem), (Loc, InvalidInlineConfigItem)>> + '_
    {
        self.iter()
            .filter_map(|comment| {
                Some((comment, comment.contents().trim_start().strip_prefix("forgefmt:")?.trim()))
            })
            .map(|(comment, item)| {
                let loc = comment.loc;
                item.parse().map(|out| (loc, out)).map_err(|out| (loc, out))
            })
    }
}

/// A disabled formatting range. `loose` designates that the range includes any loc which
/// may start in between start and end, whereas the strict version requires that
/// `range.start >= loc.start <=> loc.end <= range.end`
#[derive(Debug)]
struct DisabledRange {
    start: usize,
    end: usize,
    loose: bool,
}

impl DisabledRange {
    fn includes(&self, loc: Loc) -> bool {
        loc.start() >= self.start && (if self.loose { loc.start() } else { loc.end() } <= self.end)
    }
}

/// An inline config. Keeps track of disabled ranges.
///
/// This is a list of Inline Config items for locations in a source file. This is
/// usually acquired by parsing the comments for an `forgefmt:` items. See
/// [`Comments::parse_inline_config_items`] for details.
#[derive(Debug)]
pub struct InlineConfig {
    disabled_ranges: Vec<DisabledRange>,
}

impl InlineConfig {
    /// Build a new inline config with an iterator of inline config items and their locations in a
    /// source file
    pub fn new(items: impl IntoIterator<Item = (Loc, InlineConfigItem)>, src: &str) -> Self {
        let mut disabled_ranges = vec![];
        let mut disabled_range_start = None;
        let mut disabled_depth = 0usize;
        for (loc, item) in items.into_iter().sorted_by_key(|(loc, _)| loc.start()) {
            match item {
                InlineConfigItem::DisableNextItem => {
                    let offset = loc.end();
                    let mut char_indices = src[offset..]
                        .comment_state_char_indices()
                        .filter_map(|(state, idx, ch)| match state {
                            CommentState::None => Some((idx, ch)),
                            _ => None,
                        })
                        .skip_while(|(_, ch)| ch.is_whitespace());
                    if let Some((mut start, _)) = char_indices.next() {
                        start += offset;
                        let end = char_indices
                            .find(|(_, ch)| !ch.is_whitespace())
                            .map(|(idx, _)| offset + idx)
                            .unwrap_or(src.len());
                        disabled_ranges.push(DisabledRange { start, end, loose: true });
                    }
                }
                InlineConfigItem::DisableNextLine => {
                    let offset = loc.end();
                    let mut char_indices =
                        src[offset..].char_indices().skip_while(|(_, ch)| *ch != '\n').skip(1);
                    if let Some((mut start, _)) = char_indices.next() {
                        start += offset;
                        let end = char_indices
                            .find(|(_, ch)| *ch == '\n')
                            .map(|(idx, _)| offset + idx)
                            .unwrap_or(src.len());
                        disabled_ranges.push(DisabledRange { start, end, loose: false });
                    }
                }
                InlineConfigItem::DisableStart => {
                    if disabled_depth == 0 {
                        disabled_range_start = Some(loc.end());
                    }
                    disabled_depth += 1;
                }
                InlineConfigItem::DisableEnd => {
                    disabled_depth = disabled_depth.saturating_sub(1);
                    if disabled_depth == 0 {
                        if let Some(start) = disabled_range_start.take() {
                            disabled_ranges.push(DisabledRange {
                                start,
                                end: loc.start(),
                                loose: false,
                            })
                        }
                    }
                }
            }
        }
        if let Some(start) = disabled_range_start.take() {
            disabled_ranges.push(DisabledRange { start, end: src.len(), loose: false })
        }
        Self { disabled_ranges }
    }

    /// Check if the location is in a disabled range
    pub fn is_disabled(&self, loc: Loc) -> bool {
        self.disabled_ranges.iter().any(|range| range.includes(loc))
    }
}
