mod and;
pub use and::AndNode;
mod thin;
pub use thin::{ThinNode, THIN_CHARS};
mod wide;
pub use wide::WideNode;
mod or;
pub use or::OrNode;
mod iters;
use iters::{ChildIter, KeyIter, PairIter};

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
    KeyIter::new(self)
  }

  /// An iterator over the child indices
  fn iter(self) -> ChildIter<Self> {
    ChildIter::new(self)
  }

  /// An iterator over `(c, idx)` where:<br>
  /// - `c: u8` the child value
  /// - `idx: usize` the child's index
  fn pairs(self) -> PairIter<Self> {
    PairIter::new(self)
  }
}

impl<N: ReadNode> ReadNode for &N {
  type Idx = <N as ReadNode>::Idx;

  fn is_empty(&self) -> bool {
    N::is_empty(self)
  }
  fn is_end(&self) -> bool {
    N::is_end(self)
  }
  fn has(&self, c: u8) -> bool {
    N::has(self, c)
  }
  fn get(&self, c: u8) -> Self::Idx {
    N::get(self, c)
  }
}
impl<N: ReadNode> ReadNode for &mut N {
  type Idx = <N as ReadNode>::Idx;

  fn is_empty(&self) -> bool {
    N::is_empty(self)
  }
  fn is_end(&self) -> bool {
    N::is_end(self)
  }
  fn has(&self, c: u8) -> bool {
    N::has(self, c)
  }
  fn get(&self, c: u8) -> Self::Idx {
    N::get(self, c)
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
    N::is_end_mut(self)
  }
  fn get_mut(&mut self, c: u8) -> &mut Self::Idx {
    N::get_mut(self, c)
  }
}
