use super::{ParseArgContext, StringKind};
use crate::{
    intern::{Interner, Symbol},
    parse::{
        Reader,
        errors::{
            InvalidStringCharsError, ParseBoolError, ParseDoubleError, ParseError, ParseFloatError,
            ParseIntegerError, QuotedSingleWordError, UnterminatedStringError,
        },
    },
    span::Span,
};

#[derive(Debug, Clone, Copy)]
pub struct Boolean {
    pub value: Option<bool>,
}

#[derive(Debug, Clone, Copy)]
pub struct Integer {
    pub value: Option<i32>,
}

impl Integer {
    pub const ZERO: Self = Self::new(0);

    pub const fn new(value: i32) -> Self {
        Self { value: Some(value) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Float {
    pub value: Option<f32>,
}

impl Float {
    pub const ZERO: Self = Self::new(0.0);

    pub const fn new(value: f32) -> Self {
        Self { value: Some(value) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Double {
    pub value: Option<f64>,
}

impl Double {
    pub const ZERO: Self = Self::new(0.0);

    pub const fn new(value: f64) -> Self {
        Self { value: Some(value) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Text {
    pub value: Option<Symbol>,
    pub is_quotable: bool,
}

pub fn parse_bool(ctx: &mut ParseArgContext<'_, '_>) -> Boolean {
    let range = ctx.reader.read_range_until(char::is_whitespace);
    let value = match &ctx.reader.get_src()[range.clone()] {
        "true" => Some(true),
        "false" => Some(false),
        _ => {
            ctx.errors
                .push(ParseError::ParseBool(ParseBoolError { span: range.into() }));
            None
        }
    };
    Boolean { value }
}

fn read_number_string<'src>(reader: &mut Reader<'src>) -> Result<(&'src str, Span), ParseError> {
    fn is_number_char(chr: char) -> bool {
        matches!(chr, '0'..='9' | '.' | '-')
    }

    let range = reader.read_range_until(char::is_whitespace);
    let span = range.clone().into();
    let string = &reader.get_src()[range.clone()];
    if !string.chars().all(is_number_char) {
        Err(ParseError::ParseInteger(ParseIntegerError { span }))
    } else {
        Ok((string, span))
    }
}

pub fn parse_integer(ctx: &mut ParseArgContext<'_, '_>) -> Integer {
    let mut value = None;
    match read_number_string(ctx.reader) {
        Ok((string, span)) => match string.parse() {
            Ok(number) => value = Some(number),
            Err(_) => ctx.error(ParseError::ParseInteger(ParseIntegerError { span })),
        },
        Err(err) => ctx.error(err),
    }
    Integer { value }
}

pub fn parse_float(ctx: &mut ParseArgContext<'_, '_>) -> Float {
    let mut value = None;
    match read_number_string(ctx.reader) {
        Ok((string, span)) => match string.parse() {
            Ok(number) => value = Some(number),
            Err(_) => ctx.error(ParseError::ParseFloat(ParseFloatError { span })),
        },
        Err(err) => ctx.error(err),
    }
    Float { value }
}

pub fn parse_double(ctx: &mut ParseArgContext<'_, '_>) -> Double {
    let mut value = None;
    match read_number_string(ctx.reader) {
        Ok((string, span)) => match string.parse() {
            Ok(number) => value = Some(number),
            Err(_) => ctx.error(ParseError::ParseDouble(ParseDoubleError { span })),
        },
        Err(err) => ctx.error(err),
    }
    Double { value }
}

pub fn parse_text(ctx: &mut ParseArgContext<'_, '_>, kind: StringKind) -> Result<Text, ParseError> {
    if kind == StringKind::GreedyPhrase {
        return parse_greedy_phrase(ctx);
    }

    let Some(quote @ ('"' | '\'')) = ctx.reader.peek() else {
        let string = parse_unquoted_string(ctx);

        return string;
    };

    let string_start = ctx.reader.get_pos();

    ctx.reader.advance();
    let content_start = ctx.reader.get_pos();

    while let Some(chr) = ctx.reader.peek() {
        if chr == quote {
            let string = &ctx.reader.get_src()[content_start..ctx.reader.get_pos()];
            ctx.reader.advance();

            if kind == StringKind::SingleWord {
                ctx.error(ParseError::QuotedSingleWord(QuotedSingleWordError {
                    span: Span::new(string_start, ctx.reader.get_pos()),
                }));
            }

            return Ok(Text {
                value: Some(ctx.interner.intern(string)),
                is_quotable: true,
            });
        } else if chr == '\\' {
            ctx.reader.advance();
        }
        ctx.reader.advance();
    }

    let span = string_start..ctx.reader.get_pos();
    Err(ParseError::UnterminatedString(UnterminatedStringError {
        span: span.into(),
    }))
}

fn parse_unquoted_string(ctx: &mut ParseArgContext<'_, '_>) -> Result<Text, ParseError> {
    fn is_string_char(chr: char) -> bool {
        matches!(chr, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' | '+')
    }

    let (range, string) = ctx
        .reader
        .parse_with_span(|reader| reader.read_until(char::is_whitespace));

    let value = if !string.chars().all(is_string_char) {
        ctx.error(ParseError::InvalidStringChars(InvalidStringCharsError {
            span: range.into(),
        }));
        None
    } else {
        Some(ctx.interner.intern(string))
    };

    Ok(Text {
        value,
        is_quotable: false,
    })
}

fn parse_greedy_phrase(ctx: &mut ParseArgContext<'_, '_>) -> Result<Text, ParseError> {
    let symbol = ctx.interner.intern(ctx.reader.remaining_src().trim_end());
    ctx.reader.set_pos(ctx.reader.get_src().len());
    Ok(Text {
        value: Some(symbol),
        is_quotable: false,
    })
}
