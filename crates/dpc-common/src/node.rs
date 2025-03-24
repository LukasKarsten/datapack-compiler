use std::fmt;

use crate::{parse::argument::Argument, smallstring::SmallString};

#[derive(Clone)]
pub enum NodeKind {
    Literal(SmallString),
    Argument { name: SmallString, arg: Argument },
    Block,
}

impl fmt::Debug for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(literal) => write!(f, "{literal}"),
            Self::Argument { name, arg } => write!(f, "<{name}: {arg:?}>"),
            Self::Block => f.write_str("{BLOCK}"),
        }
    }
}

#[derive(Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub executable: bool,
    pub usable: bool,
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            executable: false,
            usable: true,
        }
    }

    pub fn literal(literal: impl Into<SmallString>) -> Self {
        Self::new(NodeKind::Literal(literal.into()))
    }

    pub fn argument(name: impl Into<SmallString>, arg: Argument) -> Self {
        Self::new(NodeKind::Argument {
            name: name.into(),
            arg,
        })
    }

    pub fn block() -> Self {
        Self::new(NodeKind::Block).executable()
    }

    pub fn executable(self) -> Self {
        Self {
            executable: true,
            ..self
        }
    }

    pub fn name(&self) -> &str {
        match &self.kind {
            NodeKind::Block => "{BLOCK}",
            NodeKind::Literal(name) => name,
            NodeKind::Argument { name, .. } => name,
        }
    }
}

impl<L: Into<SmallString>> From<L> for Node {
    fn from(literal: L) -> Self {
        Self::literal(literal)
    }
}

impl<N: Into<SmallString>> From<(N, Argument)> for Node {
    fn from((name, arg): (N, Argument)) -> Self {
        Self::argument(name, arg)
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)?;
        if self.executable {
            f.write_str(" $")?;
        }
        Ok(())
    }
}
