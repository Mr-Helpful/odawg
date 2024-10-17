use super::{IndexDawg, ReadDawg};
use crate::node::OrNode;
use std::array;

/// A DAWG that has words present in any contained DAWGs
pub struct OrDawg<const N: usize, D>([D; N]);

impl<const N: usize, D: IndexDawg<Idx = usize>> IndexDawg for OrDawg<N, D> {
  type Idx = [Option<D::Idx>; N];
  const ROOT_IDX: Self::Idx = [Some(0); N];

  type NodeRef<'a> = OrNode<N, D::NodeRef<'a>>
    where
      Self: 'a;
  fn index(&self, idxs: [Option<D::Idx>; N]) -> Self::NodeRef<'_> {
    OrNode(array::from_fn(|i| idxs[i].map(|idx| self.0[i].index(idx))))
  }
}

impl<const N: usize, D: IndexDawg<Idx = usize>> ReadDawg for OrDawg<N, D> {}
