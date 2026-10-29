//! Template syntax: nodes, `{if}`/`{elsif}`/`{else}` arms, and the
//! recursive-descent parser that scans directives while keeping every node
//! offset in original-template coordinates (they feed the upstream caret
//! report for placeholder failures).

/// Orca template node: literal text or an `{if}`/`{elsif}`/`{else}` branch.
/// Directives are consumed with their condition text (including any newline
/// inside a multiline condition); every other byte, newlines included, is
/// preserved verbatim, so whole-line and inline directives share one engine.
pub(super) enum Node {
    /// Verbatim template text with its byte offset in the template (the
    /// offset feeds the upstream caret report for placeholder failures).
    Text { text: String, offset: usize },
    /// An `{if}`/`{elsif}`/`{else}` branch.
    Branch { arms: Vec<Arm> },
}

/// One `{if}`/`{elsif}`/`{else}` arm; `None` condition is the catch-all
/// `{else}`.
pub(super) struct Arm {
    pub(super) condition: Option<String>,
    pub(super) nodes: Vec<Node>,
}

pub(super) fn parse(template: &str) -> Result<Vec<Node>, String> {
    let mut parser = Parser {
        template,
        offset: 0,
    };
    // The skipper runs before the grammar (`ascii_char_skipper_parser`,
    // `PlaceholderParser.cpp:1963-1975`); skipping here keeps every node
    // offset in original-template coordinates.
    parser.skip_leading_whitespace();
    let (nodes, terminator) = parser.parse_branch()?;
    match terminator {
        None => Ok(nodes),
        Some(_) => Err("unexpected template directive".to_owned()),
    }
}

struct Parser<'a> {
    template: &'a str,
    offset: usize,
}

#[expect(
    clippy::excessive_nesting,
    reason = "recursive descent keeps directive scanning ordered"
)]
impl Parser<'_> {
    /// Consumes the leading ASCII whitespace the phrase_parse skipper
    /// would skip before the grammar (`PlaceholderParser.cpp:1963-1975`:
    /// ' ', '\t', '\r', '\n'), advancing `offset` so it never reaches the
    /// output while node offsets stay original.
    fn skip_leading_whitespace(&mut self) {
        let bytes = self.template.as_bytes();
        while matches!(bytes.get(self.offset), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.offset += 1;
        }
    }

    /// Parses nodes until a branch terminator (`elsif`/`else`/`endif`), which
    /// is consumed and reported without turning it into a node.
    fn parse_branch(&mut self) -> Result<(Vec<Node>, Option<Directive>), String> {
        let mut nodes = Vec::new();
        loop {
            let Some((start, end, directive)) = self.next_directive() else {
                let text = &self.template[self.offset..];
                if !text.is_empty() {
                    nodes.push(Node::Text {
                        text: text.to_owned(),
                        offset: self.offset,
                    });
                }
                self.offset = self.template.len();
                return Ok((nodes, None));
            };
            if start > self.offset {
                nodes.push(Node::Text {
                    text: self.template[self.offset..start].to_owned(),
                    offset: self.offset,
                });
            }
            self.offset = end;
            match directive {
                Directive::If(condition) => {
                    let arms = self.parse_arms(condition)?;
                    nodes.push(Node::Branch { arms });
                }
                terminator => return Ok((nodes, Some(terminator))),
            }
        }
    }

    /// Parses the `{if}`/`{elsif}`/`{else}` arms that follow the opening
    /// condition, consuming the closing `{endif}`.
    fn parse_arms(&mut self, condition: String) -> Result<Vec<Arm>, String> {
        let mut arms = vec![Arm {
            condition: (!condition.is_empty()).then_some(condition),
            nodes: Vec::new(),
        }];
        let (nodes, terminator) = self.parse_branch()?;
        arms.last_mut().unwrap().nodes = nodes;
        let mut terminator = terminator;
        loop {
            match terminator {
                Some(Directive::Elsif(condition)) => {
                    let (nodes, next) = self.parse_branch()?;
                    arms.push(Arm {
                        condition: (!condition.is_empty()).then_some(condition),
                        nodes,
                    });
                    terminator = next;
                }
                Some(Directive::Else) => {
                    let (nodes, next) = self.parse_branch()?;
                    if !matches!(next, Some(Directive::Endif)) {
                        return Err("missing endif".to_owned());
                    }
                    arms.push(Arm {
                        condition: None,
                        nodes,
                    });
                    return Ok(arms);
                }
                Some(Directive::Endif) => return Ok(arms),
                _ => return Err("missing endif".to_owned()),
            }
        }
    }

    /// Finds the next directive token and returns `(start, end, directive)`
    /// where `end` is just past the closing `}`. `self.offset` is not
    /// advanced; the caller keeps the text before `start`.
    fn next_directive(&self) -> Option<(usize, usize, Directive)> {
        let bytes = self.template.as_bytes();
        let mut index = self.offset;
        while index < bytes.len() {
            if bytes[index] != b'{' {
                index += 1;
                continue;
            }
            let rest = &self.template[index + 1..];
            let trimmed = rest.trim_start_matches([' ', '\t']);
            let keyword_start = index + 1 + (rest.len() - trimmed.len());
            let (name, length) = if trimmed
                .strip_prefix("if")
                .is_some_and(|tail| tail.starts_with([' ', '\t', '(']) || tail.starts_with('}'))
            {
                ("if", "if".len())
            } else if trimmed
                .strip_prefix("elsif")
                // `kw["elsif"] > bool_expr_eval` accepts `elsif(`
                // without whitespace (`PlaceholderParser.cpp:2110,2182`).
                .is_some_and(|tail| tail.starts_with([' ', '\t', '(']) || tail.starts_with('}'))
            {
                ("elsif", "elsif".len())
            } else if trimmed
                .strip_prefix("else")
                .is_some_and(|tail| tail.trim_start().starts_with('}'))
            {
                ("else", "else".len())
            } else if trimmed
                .strip_prefix("endif")
                .is_some_and(|tail| tail.trim_start().starts_with('}'))
            {
                ("endif", "endif".len())
            } else {
                index += 1;
                continue;
            };
            let end = self.directive_end(keyword_start)?;
            let condition = self.template[keyword_start + length..end].trim().to_owned();
            if name == "if" {
                return Some((index, end + 1, Directive::If(condition)));
            }
            let directive = match name {
                "elsif" => Directive::Elsif(condition),
                "else" => Directive::Else,
                _ => Directive::Endif,
            };
            return Some((index, end + 1, directive));
        }
        None
    }

    /// Index of the `}` closing a directive opened at `start` (already past
    /// `{`). Square-bracket subscripts may contain `}`-free indices but are
    /// still skipped for safety.
    fn directive_end(&self, start: usize) -> Option<usize> {
        let mut square_depth: u32 = 0;
        for (offset, character) in self.template[start..].char_indices() {
            match character {
                '[' => square_depth += 1,
                ']' => square_depth = square_depth.saturating_sub(1),
                '}' if square_depth == 0 => return Some(start + offset),
                _ => {}
            }
        }
        None
    }
}

enum Directive {
    If(String),
    Elsif(String),
    Else,
    Endif,
}
