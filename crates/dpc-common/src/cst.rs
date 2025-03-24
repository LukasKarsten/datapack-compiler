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
    fn visit_item(&mut self, item: &Item) {
        walk_item(self, item);
    }
    fn visit_comment(&mut self, _comment: &Span) {}
    fn visit_command(&mut self, command: &Command) {
        walk_command(self, command);
    }
    fn visit_argument(&mut self, argument: &Argument) {
        walk_argument(self, argument);
    }
    fn visit_block(&mut self, block: &Block) {
        walk_block(self, block);
    }
    fn visit_boolean(&mut self, _boolean: &Boolean) {}
    fn visit_integer(&mut self, _integer: &Integer) {}
    fn visit_float(&mut self, _float: &Float) {}
    fn visit_double(&mut self, _double: &Double) {}
    fn visit_string(&mut self, _string: &String) {}
    fn visit_angle(&mut self, _angle: &Angle) {}
    fn visit_parse_error(&mut self, _error: &ParseError) {}
}

pub fn walk_item(visitor: &mut impl Visitor, item: &Item) {
    match item {
        Item::Command(command) => visitor.visit_command(command),
        Item::Comment(comment) => visitor.visit_comment(comment),
    }
}

pub fn walk_command(visitor: &mut impl Visitor, command: &Command) {
    for argument in &command.args {
        visitor.visit_argument(argument);
    }
    if let Some(error) = &command.error {
        visitor.visit_parse_error(error);
    }
}

pub fn walk_argument(visitor: &mut impl Visitor, argument: &Argument) {
    match &argument.value {
        ArgumentValue::Literal => (),
        ArgumentValue::Block(block) => visitor.visit_block(block),
        ArgumentValue::Boolean(boolean) => visitor.visit_boolean(boolean),
        ArgumentValue::Integer(integer) => visitor.visit_integer(integer),
        ArgumentValue::Float(float) => visitor.visit_float(float),
        ArgumentValue::Double(double) => visitor.visit_double(double),
        ArgumentValue::String(string) => visitor.visit_string(string),
        ArgumentValue::Angle(string) => visitor.visit_angle(string),
    }

    for error in &argument.errors {
        visitor.visit_parse_error(error);
    }
}

pub fn walk_block(visitor: &mut impl Visitor, block: &Block) {
    for item in &block.items {
        visitor.visit_item(item);
    }
}

pub fn walk_angle(visitor: &mut impl Visitor, angle: &Angle) {
    visitor.visit_float(&angle.value);
}
