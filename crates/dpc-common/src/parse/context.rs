use std::sync::Arc;

use crate::{
    ParsingTree,
    intern::StaticInterner,
    parse::{cst::Block, errors::ParseError},
    source::SourceFile,
};

pub struct ParseContext<'src> {
    pub source: &'src SourceFile,
    pub tree: Arc<ParsingTree>,
    pub interner: StaticInterner,
}

impl<'src> ParseContext<'src> {
    pub fn new(source: &'src SourceFile, parse_tree: Arc<ParsingTree>) -> Self {
        Self {
            source,
            tree: parse_tree,
            interner: StaticInterner::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Block, ParseError> {
        Arc::clone(&self.tree).parse(self)
    }
}
