use super::{IndexDawg, ReadDawg, ReadNode};

pub struct NodeIter<'a, D: IndexDawg> {
    dawg: &'a D,
    idxs: Vec<D::Idx>,
}

impl<'a, D: ReadDawg> From<&'a D> for NodeIter<'a, D> {
    fn from(value: &'a D) -> Self {
        NodeIter {
            dawg: value,
            idxs: vec![D::ROOT_IDX],
        }
    }
}

impl<'a, D: IndexDawg> Iterator for NodeIter<'a, D> {
    type Item = D::NodeRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idxs.pop()?;
        let node = self.dawg.index(idx);
        self.idxs.extend(node.iter());
        Some(node)
    }
}
