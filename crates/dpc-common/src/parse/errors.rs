use std::{fmt, ops::Range};

use ariadne::{Color, Fmt};

use crate::{
    diagnostics::{Diagnostic, Label},
    parse::ParseContext,
    span::Span,
};

pub trait EmitDiagnostic: std::fmt::Debug + Send + Sync {
    fn emit(&self, ctx: &ParseContext<'_>) -> Diagnostic;
}

impl<T: EmitDiagnostic> EmitDiagnostic for &T {
    fn emit(&self, ctx: &ParseContext<'_>) -> Diagnostic {
        T::emit(self, ctx)
    }
}

#[derive(Debug)]
pub enum ParseError {
    Indentation(IndentationError),
    InvalidLiteral(InvalidLiteralError),
    TooManyArguments(TooManyArgumentsError),
    ParseBool(ParseBoolError),
    ParseDouble(ParseDoubleError),
    ParseFloat(ParseFloatError),
    ParseInteger(ParseIntegerError),
    UnterminatedString(UnterminatedStringError),
    InvalidStringChars(InvalidStringCharsError),
    QuotedSingleWord(QuotedSingleWordError),
    IncompleteLocalCoordinates(IncompleteLocalCoordinatesError),
    ExpectedLocalCoordinate(ExpectedLocalCoordinateError),
    MixedCoordinates(MixedCoordiantesError),
}

impl EmitDiagnostic for ParseError {
    fn emit(&self, ctx: &ParseContext<'_>) -> Diagnostic {
        match self {
            Self::Indentation(error) => error.emit(ctx),
            Self::InvalidLiteral(error) => error.emit(ctx),
            Self::TooManyArguments(error) => error.emit(ctx),
            Self::ParseBool(error) => error.emit(ctx),
            Self::ParseDouble(error) => error.emit(ctx),
            Self::ParseFloat(error) => error.emit(ctx),
            Self::ParseInteger(error) => error.emit(ctx),
            Self::UnterminatedString(error) => error.emit(ctx),
            Self::InvalidStringChars(error) => error.emit(ctx),
            Self::QuotedSingleWord(error) => error.emit(ctx),
            Self::IncompleteLocalCoordinates(error) => error.emit(ctx),
            Self::ExpectedLocalCoordinate(error) => error.emit(ctx),
            Self::MixedCoordinates(error) => error.emit(ctx),
        }
    }
}

#[derive(Debug)]
pub struct IndentationError {
    pub span: Span,
    pub kind: IndentationErrorKind,
}

#[derive(Debug)]
pub enum IndentationErrorKind {
    MixedWhitespace,
    InvalidIndentation,
}

impl EmitDiagnostic for IndentationError {
    fn emit(&self, _: &ParseContext<'_>) -> Diagnostic {
        Diagnostic::error(self.span, "Indentation error").with_label(Label::new(
            self.span,
            match self.kind {
                IndentationErrorKind::MixedWhitespace => "Must only use spaces for indentation",
                IndentationErrorKind::InvalidIndentation => "Invalid indentation",
            },
        ))
    }
}

#[derive(Debug)]
pub struct InvalidLiteralError {
    pub span: Span,
    pub valid_literals: Range<usize>,
}

impl EmitDiagnostic for InvalidLiteralError {
    fn emit(&self, ctx: &ParseContext<'_>) -> Diagnostic {
        let mut valid_literals: Vec<_> = self
            .valid_literals
            .clone()
            .map(|node_id| ctx.tree.get_node(node_id).unwrap().name())
            .collect();
        valid_literals.sort();

        let mut diagnostic =
            Diagnostic::error(self.span, "Invalid literal").with_label(Label::new(
                self.span,
                format!(
                    "Expected one of {}",
                    valid_literals
                        .iter()
                        .map(|lit| lit.fg(Color::BrightGreen).surrounded('`', '`'))
                        .delimited(", ", " or ")
                ),
            ));

        let input = &ctx.source.text()[self.span.as_range()];
        let likely_literal = valid_literals
            .into_iter()
            .map(|lit| (lit, strsim::normalized_damerau_levenshtein(lit, input)))
            .filter(|entry| entry.1 > 0.5)
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());

        if let Some((likely_literal, _)) = likely_literal {
            diagnostic = diagnostic.with_help(format!(
                "Did you mean {}?",
                likely_literal.fg(Color::BrightGreen).surrounded('`', '`')
            ));
        }

        diagnostic
    }
}

#[derive(Debug)]
pub struct TooManyArgumentsError {
    pub span: Span,
}

impl EmitDiagnostic for TooManyArgumentsError {
    fn emit(&self, ctx: &ParseContext<'_>) -> Diagnostic {
        Diagnostic::error(self.span, "Too many arguments").with_label(Label::new(
            self.span,
            match ctx.source.text()[self.span.as_range()].contains(char::is_whitespace) {
                true => "These arguments were not expected",
                false => "This argument was not expected",
            },
        ))
    }
}

#[derive(Debug)]
pub struct ParseBoolError {
    pub span: Span,
}

impl EmitDiagnostic for ParseBoolError {
    fn emit(&self, _: &ParseContext<'_>) -> Diagnostic {
        Diagnostic::error(self.span, "Invalid boolean").with_label(Label::new(
            self.span,
            format!(
                "Expected either `{}` or `{}`",
                "true".fg(Color::BrightGreen),
                "false".fg(Color::BrightGreen),
            ),
        ))
    }
}

#[derive(Debug)]
pub struct ParseDoubleError {
    pub span: Span,
}

impl EmitDiagnostic for ParseDoubleError {
    fn emit(&self, _: &ParseContext<'_>) -> Diagnostic {
        Diagnostic::error(self.span, "Invalid double").with_label(Label::new(
            self.span,
            format!("Expected a {}", "double".fg(Color::Magenta)),
        ))
    }
}

#[derive(Debug)]
pub struct ParseFloatError {
    pub span: Span,
}

impl EmitDiagnostic for ParseFloatError {
    fn emit(&self, _: &ParseContext<'_>) -> Diagnostic {
        Diagnostic::error(self.span, "Invalid float").with_label(Label::new(
            self.span,
            format!("Expected a {}", "float".fg(Color::Magenta)),
        ))
    }
}

#[derive(Debug)]
pub struct ParseIntegerError {
    pub span: Span,
}

impl EmitDiagnostic for ParseIntegerError {
    fn emit(&self, _: &ParseContext<'_>) -> Diagnostic {
        Diagnostic::error(self.span, "Invalid integer").with_label(Label::new(
            self.span,
            format!("Expected an {}", "integer".fg(Color::Magenta)),
        ))
    }
}

#[derive(Debug)]
pub struct UnterminatedStringError {
    pub span: Span,
}

impl EmitDiagnostic for UnterminatedStringError {
    fn emit(&self, _: &ParseContext<'_>) -> Diagnostic {
        Diagnostic::error(self.span, "Unterminated string")
            .with_label(Label::new(self.span, "Missing closing quotation mark"))
    }
}

#[derive(Debug)]
pub struct InvalidStringCharsError {
    pub span: Span,
}

impl EmitDiagnostic for InvalidStringCharsError {
    fn emit(&self, _: &ParseContext<'_>) -> Diagnostic {
        Diagnostic::error(self.span, "Invalid characters in string")
    }
}

#[derive(Debug)]
pub struct QuotedSingleWordError {
    pub span: Span,
}

impl EmitDiagnostic for QuotedSingleWordError {
    fn emit(&self, _: &ParseContext<'_>) -> Diagnostic {
        Diagnostic::error(self.span, "Cannot quote single-word strings")
            .with_label(Label::new(self.span, "This string must not be quoted"))
    }
}

#[derive(Debug)]
pub struct IncompleteLocalCoordinatesError {
    pub span: Span,
}

impl EmitDiagnostic for IncompleteLocalCoordinatesError {
    fn emit(&self, _: &ParseContext<'_>) -> Diagnostic {
        Diagnostic::error(self.span, "Incomplete local coordinates")
    }
}

#[derive(Debug)]
pub struct ExpectedLocalCoordinateError {
    pub span: Span,
}

impl EmitDiagnostic for ExpectedLocalCoordinateError {
    fn emit(&self, _: &ParseContext<'_>) -> Diagnostic {
        Diagnostic::error(self.span, "Expected local coordinate")
    }
}

#[derive(Debug)]
pub struct MixedCoordiantesError {
    pub span: Span,
}

impl EmitDiagnostic for MixedCoordiantesError {
    fn emit(&self, _: &ParseContext<'_>) -> Diagnostic {
        Diagnostic::error(self.span, "Cannot mix world and local coordinates")
    }
}

struct Surrounded<L, T, R> {
    left: L,
    inner: T,
    right: R,
}

impl<L: fmt::Display, T: fmt::Display, R: fmt::Display> fmt::Display for Surrounded<L, T, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.left.fmt(f)?;
        self.inner.fmt(f)?;
        self.right.fmt(f)
    }
}

struct Delimited<T, D, L> {
    delimiter: D,
    last_delimiter: L,
    values: T,
}

impl<T, D, L, I> fmt::Display for Delimited<T, D, L>
where
    T: IntoIterator<Item = I> + Clone,
    I: fmt::Display,
    D: fmt::Display,
    L: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.values.clone().into_iter();
        let mut next = iter.next();
        let mut first = true;

        while let Some(current) = next {
            next = iter.next();
            if !first {
                match next {
                    Some(_) => self.delimiter.fmt(f)?,
                    None => self.last_delimiter.fmt(f)?,
                }
            }
            first = false;

            current.fmt(f)?;
        }

        Ok(())
    }
}

trait FmtExt: Sized {
    fn surrounded<L, R>(self, left: L, right: R) -> Surrounded<L, Self, R>;

    fn delimited<D, L>(self, delimiter: D, last_delimiter: L) -> Delimited<Self, D, L>;
}

impl<T> FmtExt for T {
    fn surrounded<L, R>(self, left: L, right: R) -> Surrounded<L, Self, R> {
        Surrounded {
            left,
            inner: self,
            right,
        }
    }

    fn delimited<D, L>(self, delimiter: D, last_delimiter: L) -> Delimited<Self, D, L> {
        Delimited {
            delimiter,
            last_delimiter,
            values: self,
        }
    }
}
