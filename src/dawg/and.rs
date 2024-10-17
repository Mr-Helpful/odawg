use super::{IndexDawg, ReadDawg};
use crate::node::AndNode;
use std::array;

/// A DAWG that only has words present in all contained DAWGs
pub struct AndDawg<const N: usize, D>([D; N]);

impl<const N: usize, D: IndexDawg<Idx = usize>> IndexDawg for AndDawg<N, D> {
  type Idx = [D::Idx; N];
  const ROOT_IDX: Self::Idx = [0; N];

  type NodeRef<'a> = AndNode<N, D::NodeRef<'a>>
    where
      Self: 'a;
  fn index(&self, idxs: [D::Idx; N]) -> Self::NodeRef<'_> {
    AndNode(array::from_fn(|i| self.0[i].index(idxs[i])))
  }
}

impl<const N: usize, D: IndexDawg<Idx = usize>> ReadDawg for AndDawg<N, D> {}
