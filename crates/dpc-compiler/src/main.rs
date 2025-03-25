use std::{fs, path::PathBuf, sync::Arc};

use clap::Parser;
use dpc_common::{
    parse::{
        ParseContext, cst,
        errors::{EmitDiagnostic, ParseError},
    },
    source::SourceFile,
};

/// Datapack Compiler
#[derive(clap::Parser)]
struct Options {
    /// The file to compile
    file: PathBuf,
}

fn main() {
    let options = Options::parse();

    let tree = Arc::new(dpc_common::load_tree());
    println!("{tree:?}");

    let source = fs::read_to_string(&options.file).unwrap();
    let file_name = options.file.to_string_lossy().into_owned();
    let source_file = SourceFile::new(Some(options.file), source);
    let mut ctx = ParseContext::new(&source_file, Arc::clone(&tree));

    let block = ctx.parse();
    println!("{block:#?}");

    struct ParseErrorVisitor<'a> {
        ctx: &'a ParseContext<'a>,
    }

    impl cst::Visitor for ParseErrorVisitor<'_> {
        fn visit_parse_error(&mut self, error: &ParseError) {
            let file_name = self.ctx.source.path().unwrap().to_str().unwrap();
            let diag = error.emit(self.ctx);
            diag.to_ariadne_report(file_name)
                .eprint((file_name, ariadne::Source::from(self.ctx.source.text())))
                .unwrap()
        }
    }

    match block {
        Ok(block) => {
            let mut visitor = ParseErrorVisitor { ctx: &ctx };
            cst::walk_block(&mut visitor, &block);
        }
        Err(err) => err
            .emit(&ctx)
            .to_ariadne_report(&file_name)
            .eprint((
                file_name.as_str(),
                ariadne::Source::from(source_file.text()),
            ))
            .unwrap(),
    }
}
