use std::{iter, num::NonZeroUsize};

use super::{Node, NodeKind};
use crate::{
    parsing_tree::{ParsingNode, ParsingTree},
    smallstring::SmallString,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildNodeId(usize);

impl BuildNodeId {
    pub const ROOT: Self = Self(0);
    const INVALID: Self = Self(usize::MAX);

    fn new(index: usize) -> Self {
        assert_ne!(index, Self::INVALID.0);
        Self(index)
    }

    fn index(self) -> usize {
        self.0
    }
}

struct BuildNode {
    next_sibling: BuildNodeId,
    next: BuildNodeNext,
    parsing_tree_idx: usize,
    node: Node,
}

impl BuildNode {
    fn new(node: Node) -> Self {
        Self {
            next_sibling: BuildNodeId::INVALID,
            next: BuildNodeNext::Children {
                first_child: NonZeroUsize::MAX,
                count: 0,
            },
            parsing_tree_idx: usize::MAX,
            node,
        }
    }
}

enum BuildNodeNext {
    Children {
        first_child: NonZeroUsize,
        count: usize,
    },
    Redirect(usize),
}

pub struct BuildTree {
    nodes: Vec<BuildNode>,
}

impl Default for BuildTree {
    fn default() -> Self {
        Self {
            nodes: vec![BuildNode::new(Node::literal("ROOT"))],
        }
    }
}

impl BuildTree {
    fn get_node(&self, id: BuildNodeId) -> &BuildNode {
        &self.nodes[id.index()]
    }

    fn get_node_mut(&mut self, id: BuildNodeId) -> &mut BuildNode {
        &mut self.nodes[id.index()]
    }

    pub fn find_node_id<T: AsRef<str>>(
        &self,
        path: impl IntoIterator<Item = T>,
    ) -> Option<BuildNodeId> {
        let mut id = BuildNodeId::ROOT;

        'path_iter: for path_element in path {
            let mut child_id = loop {
                let parent = self.get_node(id);
                match parent.next {
                    BuildNodeNext::Children { count: 0, .. } => return None,
                    BuildNodeNext::Children { first_child, .. } => {
                        break BuildNodeId::new(first_child.get());
                    }
                    BuildNodeNext::Redirect(target_idx) => id = BuildNodeId::new(target_idx),
                }
            };

            while child_id != BuildNodeId::INVALID {
                let child = self.get_node(child_id);

                if child.node.name() == path_element.as_ref() {
                    id = child_id;
                    continue 'path_iter;
                }

                child_id = child.next_sibling;
            }

            return None;
        }

        Some(id)
    }

    pub fn insert(&mut self, parent_node_id: BuildNodeId, node: impl Into<Node>) -> BuildNodeId {
        fn inner(tree: &mut BuildTree, parent_node_id: BuildNodeId, node: Node) -> BuildNodeId {
            assert!(parent_node_id.index() < tree.nodes.len());

            let node_idx = NonZeroUsize::new(tree.nodes.len()).unwrap();
            let node_id = BuildNodeId::new(node_idx.get());
            let mut node = BuildNode::new(node);

            let parent = tree.get_node_mut(parent_node_id);

            let BuildNodeNext::Children { first_child, count } = &mut parent.next else {
                panic!("cannot add children to redirecting node");
            };

            node.next_sibling = BuildNodeId(first_child.get());
            *first_child = node_idx;
            *count += 1;

            tree.nodes.push(node);
            node_id
        }
        inner(self, parent_node_id, node.into())
    }

    pub fn clear_node(&mut self, node_id: BuildNodeId) {
        assert!(node_id.index() < self.nodes.len());

        let parent = self.get_node_mut(node_id);

        parent.next = BuildNodeNext::Children {
            first_child: NonZeroUsize::MAX,
            count: 0,
        };
    }

    pub fn redirect(&mut self, node_id: BuildNodeId, target: BuildNodeId) {
        // NOTE: cannot redirect to a redirecting node since that would break the parsing tree
        // construction algorithm.
        assert!(!matches!(
            self.get_node(target).next,
            BuildNodeNext::Redirect(_)
        ));

        let node = self.get_node_mut(node_id);
        match node.next {
            BuildNodeNext::Children { count: 0, .. } => {
                node.next = BuildNodeNext::Redirect(target.index());
            }
            BuildNodeNext::Children { .. } => panic!("cannot redirect node with children"),
            BuildNodeNext::Redirect(_) => panic!("node is already redirected"),
        }
    }

    pub fn into_parsing_tree(mut self) -> ParsingTree {
        fn insert_children(
            build_tree: &mut BuildTree,
            parsing_nodes: &mut Vec<ParsingNode>,
            redirected_nodes: &mut Vec<(usize, BuildNodeId)>,
            first_child: NonZeroUsize,
            count: usize,
        ) {
            if count == 0 {
                return;
            }

            let start = parsing_nodes.len();
            let end = start + count;

            parsing_nodes.extend(
                iter::repeat(ParsingNode {
                    node: Node::new(NodeKind::Literal(SmallString::default())),
                    children: 0..0,
                })
                .take(count),
            );

            let mut node_id = BuildNodeId(first_child.get());
            for i in start..end {
                assert_ne!(node_id, BuildNodeId::INVALID);

                let base = parsing_nodes.len();
                parsing_nodes[i].node = build_tree.get_node(node_id).node.clone();

                match build_tree.get_node(node_id).next {
                    BuildNodeNext::Children { first_child, count } => {
                        insert_children(
                            build_tree,
                            parsing_nodes,
                            redirected_nodes,
                            first_child,
                            count,
                        );

                        parsing_nodes[i].children = base..(base + count);
                    }
                    BuildNodeNext::Redirect(target) => {
                        redirected_nodes.push((i, BuildNodeId::new(target)));
                    }
                }

                let node = build_tree.get_node_mut(node_id);
                node.parsing_tree_idx = i;
                node_id = node.next_sibling;
            }

            assert_eq!(node_id, BuildNodeId::INVALID);

            let nodes = &mut parsing_nodes[start..(start + count)];

            // Put literal nodes before argument nodes, so they are checked first
            partition(nodes, |node| matches!(node.node.kind, NodeKind::Literal(_)));
        }

        let mut parsing_tree = ParsingTree::default();
        let mut redirected_nodes = Vec::new();

        let root_node = self.get_node(BuildNodeId::ROOT);
        let BuildNodeNext::Children { first_child, count } = root_node.next else {
            panic!("root node must not be redirected");
        };

        insert_children(
            &mut self,
            &mut parsing_tree.nodes,
            &mut redirected_nodes,
            first_child,
            count,
        );
        parsing_tree.num_roots = count;

        // NOTE: the `redirect` function guarantees that nodes never redirect to already
        // redirecting nodes, therefore the children ranges of the targets should be valid.
        for (parsing_node_idx, target_id) in redirected_nodes {
            if target_id == BuildNodeId::ROOT {
                parsing_tree.nodes[parsing_node_idx].children = 0..count;
            } else {
                let target_idx = self.get_node(target_id).parsing_tree_idx;
                assert!(target_idx != usize::MAX);
                parsing_tree.nodes[parsing_node_idx].children =
                    parsing_tree.nodes[target_idx].children.clone();
            }
        }

        parsing_tree
    }
}

/// Sorts the slice such that all elements, for which the predicate is true, are in the first half
/// of the slice and all other elements are in the second half. Returns the index of the first
/// element in the second half.
fn partition<T>(data: &mut [T], predicate: impl Fn(&T) -> bool) -> usize {
    let len = data.len();
    if len == 0 {
        return 0;
    }
    let (mut l, mut r) = (0, len - 1);
    loop {
        while l < len && predicate(&data[l]) {
            l += 1;
        }
        while r > 0 && !predicate(&data[r]) {
            r -= 1;
        }
        if l >= r {
            return l;
        }
        data.swap(l, r);
    }
}
