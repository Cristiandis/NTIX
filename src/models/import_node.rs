use std::{cell::RefCell, path::PathBuf, rc::Rc};

pub(crate) struct ImportNodeBuilder {
    pub(crate) path: PathBuf,
    pub(crate) children: Vec<ImportNodeBuilderRef>,
}

pub(crate) type ImportNodeBuilderRef = Rc<RefCell<ImportNodeBuilder>>;

impl ImportNodeBuilder {
    pub(crate) fn new(path: impl Into<PathBuf>) -> ImportNodeBuilderRef {
        Rc::new(RefCell::new(Self {
            path: path.into(),
            children: Vec::new(),
        }))
    }

    /// Converts a builder node (and its whole subtree) into a plain, owned `ImportNode`.
    pub(crate) fn to_owned_tree(node: &ImportNodeBuilderRef) -> ImportNode {
        let node_ref = node.borrow();
        ImportNode {
            path: node_ref.path.clone(),
            children: node_ref
                .children
                .iter()
                .map(ImportNodeBuilder::to_owned_tree)
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportNode {
    pub path: PathBuf,
    pub children: Vec<ImportNode>,
}
