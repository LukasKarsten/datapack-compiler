pub mod arguments;
mod build_tree;
pub mod cst;
pub mod diagnostics;
mod import;
mod intern;
mod node;
pub mod parse;
mod parsing_tree;
mod smallstring;
pub mod source;
pub mod span;

pub use build_tree::{BuildNodeId, BuildTree};
pub use node::{Node, NodeKind};
pub use parsing_tree::{ParsingNode, ParsingTree};
pub use smallstring::SmallString;

pub fn load_tree() -> ParsingTree {
    let mut build_tree = BuildTree::default();
    import::import(
        &std::fs::read_to_string("commands.json").unwrap(),
        &mut build_tree,
    );

    let execute_run_node = build_tree.find_node_id(["execute", "run"]).unwrap();
    build_tree.clear_node(execute_run_node);
    build_tree.insert(execute_run_node, Node::block());

    let return_run_node = build_tree.find_node_id(["return", "run"]).unwrap();
    build_tree.clear_node(return_run_node);
    build_tree.insert(return_run_node, Node::block());

    build_tree.into_parsing_tree()
}
