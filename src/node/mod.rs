use std::ops::{Deref, DerefMut};

mod and;
pub use and::AndNode;
mod thin;
pub use thin::{ThinNode, THIN_CHARS};
mod wide;
pub use wide::WideNode;
mod or;
pub use or::OrNode;

/// A node of a DAWG that can have it children read.
pub trait ReadNode: Sized {
  type Idx;

  /// Whether `self` has no children
  fn is_empty(&self) -> bool;

  /// Whether `self` represents the end of a word
  fn is_end(&self) -> bool;

  /// Whether `self` has the specified child
  fn has(&self, c: u8) -> bool;

  /// The index of a given child `c`
  fn get(&self, c: u8) -> Self::Idx;

  /// Helper method for implementing iterators
  fn next_c(&self, mut c: u8) -> Option<u8> {
    while (c as usize) < THIN_CHARS {
      if self.has(c) {
        return Some(c);
      }
      c += 1;
    }
    None
  }

  /// An iterator over keys used to access children
  fn keys(self) -> KeyIter<Self> {
    KeyIter { node: self, c: 0 }
  }

  /// An iterator over the child indices
  fn iter(self) -> ChildIter<Self> {
    ChildIter { node: self, c: 0 }
  }

  /// An iterator over `(c, idx)` where:<br>
  /// - `c: u8` the child value
  /// - `idx: usize` the child's index
  fn pairs(self) -> PairIter<Self> {
    PairIter { node: self, c: 0 }
  }
}

impl<N: ReadNode> ReadNode for &N {
  type Idx = <N as ReadNode>::Idx;

  fn is_empty(&self) -> bool {
    <N as ReadNode>::is_empty(self)
  }
  fn is_end(&self) -> bool {
    <N as ReadNode>::is_end(self)
  }
  fn has(&self, c: u8) -> bool {
    <N as ReadNode>::has(self, c)
  }
  fn get(&self, c: u8) -> Self::Idx {
    <N as ReadNode>::get(self, c)
  }
}
impl<N: ReadNode> ReadNode for &mut N {
  type Idx = <N as ReadNode>::Idx;

  fn is_empty(&self) -> bool {
    <N as ReadNode>::is_empty(self)
  }
  fn is_end(&self) -> bool {
    <N as ReadNode>::is_end(self)
  }
  fn has(&self, c: u8) -> bool {
    <N as ReadNode>::has(self, c)
  }
  fn get(&self, c: u8) -> Self::Idx {
    <N as ReadNode>::get(self, c)
  }
}

pub struct KeyIter<N> {
  node: N,
  c: u8,
}

impl<N> Deref for KeyIter<N> {
  type Target = N;
  fn deref(&self) -> &Self::Target {
    &self.node
  }
}
impl<N> DerefMut for KeyIter<N> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.node
  }
}

impl<N: ReadNode> Iterator for KeyIter<N> {
  type Item = u8;
  fn next(&mut self) -> Option<Self::Item> {
    let c = self.node.next_c(self.c)?;
    self.c = c + 1;
    Some(c)
  }
}

pub struct ChildIter<N> {
  node: N,
  c: u8,
}

impl<N> Deref for ChildIter<N> {
  type Target = N;
  fn deref(&self) -> &Self::Target {
    &self.node
  }
}
impl<N> DerefMut for ChildIter<N> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.node
  }
}

impl<N: ReadNode> Iterator for ChildIter<N> {
  type Item = N::Idx;
  fn next(&mut self) -> Option<Self::Item> {
    let c = self.node.next_c(self.c)?;
    let idx = self.node.get(c);
    self.c = c + 1;
    Some(idx)
  }
}

pub struct PairIter<N> {
  node: N,
  c: u8,
}

impl<N> Deref for PairIter<N> {
  type Target = N;
  fn deref(&self) -> &Self::Target {
    &self.node
  }
}
impl<N> DerefMut for PairIter<N> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.node
  }
}

impl<N: ReadNode> Iterator for PairIter<N> {
  type Item = (u8, N::Idx);
  fn next(&mut self) -> Option<Self::Item> {
    let c = self.node.next_c(self.c)?;
    let idx = self.node.get(c);
    self.c = c + 1;
    Some((c, idx))
  }
}

/// A node that can set have its contents modified.
pub trait WriteNode: ReadNode {
  /// Sets whether the current node is the end of a word.<br>
  /// Returns the previous value of the end flag.
  fn is_end_mut(&mut self) -> &mut bool;

  /// Sets the index of a given child `c`.<br>
  /// Returns the previous index of the child.
  fn get_mut(&mut self, c: u8) -> &mut Self::Idx;
}

impl<N: WriteNode> WriteNode for &mut N {
  fn is_end_mut(&mut self) -> &mut bool {
    <N as WriteNode>::is_end_mut(self)
  }
  fn get_mut(&mut self, c: u8) -> &mut Self::Idx {
    <N as WriteNode>::get_mut(self, c)
  }
}
