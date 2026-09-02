use std::{cell::RefCell, path::PathBuf, rc::Rc};

// Used only while building the import tree because it needs shared mutable access
// since a node's children get pushed to from inside the import() closure
// while other code still holds a reference to that same node.
pub struct ImportNodeBuilder {
    pub path: PathBuf,
    pub children: Vec<ImportNodeBuilderRef>,
}

pub type ImportNodeBuilderRef = Rc<RefCell<ImportNodeBuilder>>;

impl ImportNodeBuilder {
    pub fn new(path: impl Into<PathBuf>) -> ImportNodeBuilderRef {
        Rc::new(RefCell::new(Self {
            path: path.into(),
            children: Vec::new(),
        }))
    }

    /// Converts a builder node (and its whole subtree) into a plain, owned `ImportNode`.
    pub fn to_owned_tree(node: &ImportNodeBuilderRef) -> ImportNode {
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
