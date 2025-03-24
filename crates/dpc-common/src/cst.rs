use smallvec::SmallVec;

use crate::{intern::Symbol, parse::errors::ParseError, span::Span};

#[derive(Debug)]
pub enum Item {
    Command(Command),
    Comment(Span),
}

#[derive(Debug)]
pub struct Command {
    pub args: Vec<Argument>,
    pub error: Option<ParseError>,
}

#[derive(Debug)]
pub struct Argument {
    pub span: Span,
    pub lin_node_id: usize,
    pub value: ArgumentValue,
    pub errors: SmallVec<[ParseError; 1]>,
}

impl Argument {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

#[derive(Debug)]
pub enum ArgumentValue {
    Literal,
    Block(Block),
    Boolean(Boolean),
    Integer(Integer),
    Float(Float),
    Double(Double),
    String(String),
    Angle(Angle),
    Coordinates2(Coordinates<2>),
    Coordinates3(Coordinates<3>),
}

#[derive(Debug)]
pub struct Block {
    pub items: Vec<Item>,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringKind {
    Bare,
    Quotable,
}

#[derive(Debug, Clone, Copy)]
pub struct String {
    pub value: Option<Symbol>,
    pub kind: StringKind,
}

#[derive(Debug)]
pub struct Angle {
    pub value: Float,
    pub relative: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct WorldCoordinate {
    pub value: Double,
    pub relative: bool,
}

#[derive(Debug)]
pub enum Coordinates<const N: usize> {
    World([WorldCoordinate; N]),
    Local([Double; N]),
}

pub trait Visitor: Sized {
    fn visit_comment(&mut self, _comment: &Span) {}
    fn visit_argument(&mut self, argument: &Argument) {
        walk_argument(self, argument);
    }
    fn visit_parse_error(&mut self, _error: &ParseError) {}
}

pub fn walk_item(visitor: &mut impl Visitor, item: &Item) {
    match item {
        Item::Command(command) => walk_command(visitor, command),
        Item::Comment(comment) => visitor.visit_comment(comment),
    }
}

pub fn walk_command(visitor: &mut impl Visitor, command: &Command) {
    if let Some(error) = &command.error {
        visitor.visit_parse_error(error);
    }
    for argument in &command.args {
        visitor.visit_argument(argument);
    }
}

pub fn walk_argument(visitor: &mut impl Visitor, argument: &Argument) {
    for error in &argument.errors {
        visitor.visit_parse_error(error);
    }

    if let ArgumentValue::Block(block) = &argument.value {
        walk_block(visitor, block);
    }
}

pub fn walk_block(visitor: &mut impl Visitor, block: &Block) {
    for item in &block.items {
        walk_item(visitor, item);
    }
}
