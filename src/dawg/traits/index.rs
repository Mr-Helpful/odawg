use super::{ReadNode, WriteNode};

/// Utility trait for raw indexing into DAWGs
pub trait IndexDawg {
  /// Indexes into the DAWGs nodes
  type Idx: Clone;

  /// The index of the root node, through which other nodes can be accessed
  const ROOT_IDX: Self::Idx;

  /// A reference to a node in the DAWG
  type NodeRef<'a>: ReadNode<Idx = Self::Idx>
  where
    Self: 'a;

  /// Returns an immutable reference to a node item
  fn index(&self, idx: Self::Idx) -> Self::NodeRef<'_>;
}

impl<D: IndexDawg> IndexDawg for &D {
  type Idx = D::Idx;
  const ROOT_IDX: Self::Idx = D::ROOT_IDX;
  type NodeRef<'a> = D::NodeRef<'a> where Self: 'a;
  fn index(&self, idx: Self::Idx) -> Self::NodeRef<'_> {
    <D as IndexDawg>::index(self, idx)
  }
}
impl<D: IndexDawg> IndexDawg for &mut D {
  type Idx = D::Idx;
  const ROOT_IDX: Self::Idx = D::ROOT_IDX;
  type NodeRef<'a> = D::NodeRef<'a> where Self: 'a;
  fn index(&self, idx: Self::Idx) -> Self::NodeRef<'_> {
    <D as IndexDawg>::index(self, idx)
  }
}

/// Utility trait for mutable indexing into DAWGs
pub trait IndexMutDawg: IndexDawg {
  /// A mutable reference to a node in the DAWG
  type NodeMut<'a>: WriteNode<Idx = Self::Idx>
  where
    Self: 'a;

  /// Returns a mutable reference to a node item
  fn index_mut(&mut self, idx: Self::Idx) -> Self::NodeMut<'_>;
}

impl<D: IndexMutDawg> IndexMutDawg for &mut D {
  type NodeMut<'a> = D::NodeMut<'a> where Self: 'a;
  fn index_mut(&mut self, idx: Self::Idx) -> Self::NodeMut<'_> {
    <D as IndexMutDawg>::index_mut(self, idx)
  }
}
