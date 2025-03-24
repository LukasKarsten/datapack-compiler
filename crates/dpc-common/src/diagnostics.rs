use std::{borrow::Cow, ops::Range};

use crate::span::Span;

#[derive(Debug)]
pub struct Diagnostic {
    level: Level,
    span: Span,
    message: Cow<'static, str>,
    labels: Vec<Label>,
    sub_diagnostics: Vec<SubDiagnostic>,
}

impl Diagnostic {
    pub fn new(level: Level, span: Span, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            level,
            span,
            message: message.into(),
            labels: Vec::new(),
            sub_diagnostics: Vec::new(),
        }
    }

    pub fn error(span: Span, message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(Level::Error, span, message)
    }

    pub fn warn(span: Span, message: impl Into<Cow<'static, str>>) -> Self {
        Self::new(Level::Error, span, message)
    }

    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    pub fn with_sub(mut self, level: Level, message: impl Into<Cow<'static, str>>) -> Self {
        self.sub_diagnostics.push(SubDiagnostic {
            level,
            message: message.into(),
        });
        self
    }

    pub fn with_help(self, message: impl Into<Cow<'static, str>>) -> Self {
        self.with_sub(Level::Help, message)
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn sub(&self) -> &[SubDiagnostic] {
        &self.sub_diagnostics
    }

    pub fn to_ariadne_report<'a>(
        &self,
        filename: &'a str,
    ) -> ariadne::Report<'static, (&'a str, Range<usize>)> {
        use ariadne::{Color, Report, ReportKind};

        let (kind, color) = match self.level {
            Level::Error => (ReportKind::Error, Color::Red),
            Level::Warn => (ReportKind::Warning, Color::Yellow),
            Level::Info => (ReportKind::Custom("Info", Color::BrightBlue), Color::Blue),
            Level::Help => (ReportKind::Custom("Help", Color::Green), Color::Green),
        };

        let span = (filename, self.span.into());

        let mut report = Report::build(kind, span);
        report.set_message(self.message.clone());

        for label in &self.labels {
            report.add_label(
                ariadne::Label::new((filename, label.span.into()))
                    .with_message(label.message.clone())
                    .with_color(color),
            );
        }

        for sub in &self.sub_diagnostics {
            match sub.level {
                Level::Info => report.add_note(sub.message.clone()),
                Level::Help => report.set_help(sub.message.clone()),
                _ => (),
            }
        }

        report.finish()
    }
}

#[derive(Debug)]
pub struct SubDiagnostic {
    level: Level,
    message: Cow<'static, str>,
}

impl SubDiagnostic {
    pub fn level(&self) -> Level {
        self.level
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug)]
pub struct Label {
    span: Span,
    message: Cow<'static, str>,
}

impl Label {
    pub fn new(span: Span, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warn,
    Info,
    Help,
}
