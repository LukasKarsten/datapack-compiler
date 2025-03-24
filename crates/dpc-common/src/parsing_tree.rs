use std::{cmp::Ordering, fmt, iter, ops::Range};

use smallvec::SmallVec;

use crate::{
    cst::{Argument, ArgumentValue, Block, Command, Item},
    parse::{
        ParseContext, Reader,
        argument::ParseArgContext,
        errors::{
            IndentationError, IndentationErrorKind, InvalidLiteralError, ParseError,
            TooManyArgumentsError,
        },
    },
    span::Span,
};

use super::{Node, NodeKind};

#[derive(Debug, Clone)]
pub struct ParsingNode {
    pub(super) node: Node,
    pub(super) children: Range<usize>,
}

#[derive(Default)]
pub struct ParsingTree {
    pub(super) nodes: Vec<ParsingNode>,
    pub(super) num_roots: usize,
}

struct ParseResult {
    value: Argument,
    next: Option<Box<Result<ParseResult, ParseError>>>,
}

impl ParsingTree {
    pub fn get_node(&self, idx: usize) -> Option<&Node> {
        self.nodes.get(idx).map(|lin_node| &lin_node.node)
    }

    pub fn parse(&self, ctx: &mut ParseContext<'_>) -> Result<Block, ParseError> {
        self.parse_commands(Reader::new(ctx.source.text()), 0, ctx)
    }

    fn parse_commands(
        &self,
        reader: Reader<'_>,
        indent: usize,
        ctx: &mut ParseContext<'_>,
    ) -> Result<Block, ParseError> {
        let groups = group(reader.get_src(), reader.get_pos(), indent)?;

        Ok(Block {
            items: groups
                .into_iter()
                .filter_map(|(range, kind)| match kind {
                    GroupKind::Comment => Some(Item::Comment(range.into())),
                    GroupKind::Command => self
                        .parse_command(Reader::with_range(reader.get_src(), range), ctx)
                        .map(Item::Command),
                })
                .collect(),
        })
    }

    fn parse_command(&self, reader: Reader<'_>, ctx: &mut ParseContext<'_>) -> Option<Command> {
        let result = self.parse_children(reader, 0..self.num_roots, ctx)?;

        let mut command = Command {
            args: Vec::new(),
            error: None,
        };

        let mut curr_node = Some(result);
        loop {
            match curr_node {
                None => break,
                Some(Ok(argument)) => {
                    command.args.push(argument.value);
                    curr_node = argument.next.map(|next| *next);
                }
                Some(Err(err)) => {
                    command.error = Some(err);
                    break;
                }
            }
        }

        Some(command)
    }

    fn parse_children(
        &self,
        mut reader: Reader<'_>,
        children: Range<usize>,
        ctx: &mut ParseContext<'_>,
    ) -> Option<Result<ParseResult, ParseError>> {
        reader.skip_whitespace();
        if !reader.has_more() {
            return None;
        }
        // make reader immutable
        let reader = reader;

        if children.is_empty() {
            let range = reader.get_pos()..reader.get_src().trim_end().len();
            return Some(Err(ParseError::TooManyArguments(TooManyArgumentsError {
                span: range.into(),
            })));
        }

        // All literal nodes always come before any argument nodes, so if the first node is not a
        // literal node, there are no other literal nodes.
        // If there are literal nodes, we already read in the potential literal here
        let current_literal = match &self.nodes[children.start].node.kind {
            NodeKind::Literal(_) => Some(reader.clone().parse_with_span(Reader::read_literal)),
            _ => None,
        };

        let mut candidates = Vec::new();

        for child_idx in children.clone() {
            let child = &self.nodes[child_idx];
            let mut child_reader = reader.clone();

            match &child.node.kind {
                NodeKind::Literal(literal) => {
                    let (span, value) = current_literal
                        .clone()
                        .expect("parsing tree is not correctly sorted");
                    if &**literal == value {
                        child_reader.set_pos(span.end);
                        return Some(Ok(ParseResult {
                            value: Argument {
                                span: span.into(),
                                lin_node_id: child_idx,
                                value: ArgumentValue::Literal,
                                errors: SmallVec::new(),
                            },
                            next: self
                                .parse_children(child_reader, child.children.clone(), ctx)
                                .map(Box::new),
                        }));
                    }
                }
                NodeKind::Argument { arg, .. } => {
                    let (span, (value, errors)) = child_reader.parse_with_span(|reader| {
                        let mut parse_arg_ctx = ParseArgContext {
                            reader,
                            interner: &mut ctx.interner,
                            errors: SmallVec::new(),
                        };
                        let value = arg.parse(&mut parse_arg_ctx);
                        (value, parse_arg_ctx.errors)
                    });
                    let result = match value {
                        Ok(value) => {
                            assert!(child_reader.peek().is_none_or(char::is_whitespace));
                            let next = self
                                .parse_children(child_reader, child.children.clone(), ctx)
                                .map(Box::new);

                            Ok(ParseResult {
                                value: Argument {
                                    span: span.into(),
                                    lin_node_id: child_idx,
                                    value,
                                    errors,
                                },
                                next,
                            })
                        }
                        Err(err) => Err(err),
                    };

                    // TODO: If the current child is a redirecting node, we should return with the
                    // current parsed node

                    candidates.push(result);
                }
                NodeKind::Block => {
                    let block = match get_indent(&child_reader) {
                        None => Ok(Block {
                            items: self
                                .parse_command(child_reader.clone(), ctx)
                                .map(|command| vec![Item::Command(command)])
                                .unwrap_or_default(),
                        }),
                        Some((line_start, indent)) => {
                            child_reader.set_pos(line_start);
                            self.parse_commands(child_reader.clone(), indent, ctx)
                        }
                    };

                    let span = Span::new(child_reader.get_pos(), child_reader.get_src().len());

                    return match block {
                        Ok(block) => Some(Ok(ParseResult {
                            value: Argument {
                                span,
                                lin_node_id: child_idx,
                                value: ArgumentValue::Block(block),
                                errors: SmallVec::new(),
                            },
                            next: None,
                        })),
                        Err(err) => Some(Err(err)),
                    };
                }
            }
        }

        if candidates.is_empty() {
            if let Some((span, _)) = current_literal {
                candidates.push(Err(ParseError::InvalidLiteral(InvalidLiteralError {
                    span: span.into(),
                    valid_literals: children.clone(),
                })));
            }
        }

        candidates.sort_by(|a, b| match (a, b) {
            (Ok(_), Err(_)) => Ordering::Less,
            (Err(_), Ok(_)) => Ordering::Greater,
            (Ok(a), Ok(b)) => match (a.value.has_errors(), b.value.has_errors()) {
                (false, true) => Ordering::Less,
                (true, false) => Ordering::Greater,
                _ => Ordering::Equal,
            },
            _ => Ordering::Equal,
        });

        Some(candidates.swap_remove(0))
    }
}

impl fmt::Debug for ParsingTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn print_children(
            f: &mut fmt::Formatter<'_>,
            nodes: &[ParsingNode],
            parent: Option<usize>,
            children_range: Range<usize>,
            indent: usize,
        ) -> fmt::Result {
            if parent.is_some_and(|parent| parent > children_range.start) {
                if children_range.start == 0 {
                    return f.write_str(" -> <ROOT>");
                }
                return match nodes.iter().find(|node| node.children == children_range) {
                    Some(target) => write!(f, " -> {:?}", target.node),
                    None => f.write_str(" -> <UNKNOWN>"),
                };
            }

            if children_range.len() == 1 {
                let child_idx = children_range.start;
                let child = &nodes[child_idx];
                write!(f, " {:?}", child.node)?;
                print_children(f, nodes, Some(child_idx), child.children.clone(), indent)?;
            } else {
                for child_idx in children_range {
                    let child = &nodes[child_idx];
                    write!(f, "\n{:indent$}{:?}", "", child.node)?;
                    print_children(
                        f,
                        nodes,
                        Some(child_idx),
                        child.children.clone(),
                        indent + 2,
                    )?;
                }
            }
            Ok(())
        }

        print_children(f, &self.nodes, None, 0..self.num_roots, 0)
    }
}

fn get_indent(reader: &Reader) -> Option<(usize, usize)> {
    let string = reader.get_src();
    let pos = reader.get_pos();

    let mut indent = 0;
    for (i, chr) in string[..pos].char_indices().rev() {
        match chr {
            ' ' => indent += 1,
            '\n' => {
                let line_start = i + 1;
                return Some((line_start, indent));
            }
            _ => break,
        }
    }

    None
}

enum GroupKind {
    Command,
    Comment,
}

fn group(
    string: &str,
    offset: usize,
    common_indent: usize,
) -> Result<Vec<(Range<usize>, GroupKind)>, ParseError> {
    let mut current_group_range: Option<Range<usize>> = None;
    let lines = string[offset..]
        .char_indices()
        // Find all line endings
        .filter_map(|(i, chr)| (chr == '\n').then_some((offset + i, chr)))
        // Add a line ending at the end of the string
        .chain(iter::once((string.len(), '\n')))
        // Work out line ranges
        .map({
            let mut next_line_start = offset;
            move |(linebreak_idx, linebreak_chr)| {
                let line_range = next_line_start..linebreak_idx;
                next_line_start = linebreak_idx + linebreak_chr.len_utf8();
                line_range
            }
        })
        // Work out line indentation and remove blank lines
        .filter_map(|line_range| {
            string[line_range.clone()]
                .find(|chr| chr != ' ')
                .map(|indent| (line_range, indent))
        });

    let mut groups = Vec::new();

    for (line_range, indent) in lines {
        let first_char = string[line_range.clone()][indent..].chars().next().unwrap();

        if first_char == '#' && indent <= common_indent {
            if let Some(group_range) = current_group_range.take() {
                groups.push((group_range, GroupKind::Command));
            }
            groups.push((line_range, GroupKind::Comment));
            continue;
        }

        if first_char.is_whitespace() {
            return Err(ParseError::Indentation(IndentationError {
                span: line_range.into(),
                kind: IndentationErrorKind::MixedWhitespace,
            }));
        }

        if indent < common_indent {
            return Err(ParseError::Indentation(IndentationError {
                span: line_range.into(),
                kind: IndentationErrorKind::InvalidIndentation,
            }));
        }

        if indent > common_indent {
            let Some(current_group_range) = &mut current_group_range else {
                return Err(ParseError::Indentation(IndentationError {
                    span: line_range.into(),
                    kind: IndentationErrorKind::InvalidIndentation,
                }));
            };

            current_group_range.end = line_range.end;
            continue;
        }

        if let Some(group_range) = current_group_range.clone() {
            groups.push((group_range, GroupKind::Command));
        }

        current_group_range = Some(line_range.clone());
    }

    if let Some(group_range) = current_group_range {
        groups.push((group_range, GroupKind::Command));
    }

    Ok(groups)
}
