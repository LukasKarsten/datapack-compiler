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
}

#[derive(Debug)]
pub struct Block {
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub struct Boolean {
    pub value: Option<bool>,
}

#[derive(Debug)]
pub struct Integer {
    pub value: Option<i32>,
}

#[derive(Debug)]
pub struct Float {
    pub value: Option<f32>,
}

#[derive(Debug)]
pub struct Double {
    pub value: Option<f64>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StringKind {
    Bare,
    Quotable,
}

#[derive(Debug)]
pub struct String {
    pub value: Option<Symbol>,
    pub kind: StringKind,
}

#[derive(Debug)]
pub struct Angle {
    pub value: Float,
    pub relative: bool,
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
