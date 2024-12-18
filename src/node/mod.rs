mod all;
pub use all::AllNode;
mod thin;
pub use thin::{NonContiguous, ThinNode, THIN_CHARS};
mod wide;
pub use wide::WideNode;
mod any;
pub use any::AnyNode;
mod iters;
use iters::{ChildIter, KeyIter, PairIter};

/// A node of a DAWG that can have it children read.
pub trait ReadNode: Sized {
    type Idx;

    /// How many children `self` has
    fn len(&self) -> usize {
        self.keys().count()
    }

    /// Whether `self` has no children
    fn is_empty(&self) -> bool {
        self.keys().next().is_none()
    }

    /// Whether `self` represents the end of a word
    fn is_end(&self) -> bool;

    /// Whether `self` has the specified child
    fn has(&self, c: u8) -> bool;

    /// The index of a given child `c`
    fn get(&self, c: u8) -> Self::Idx;

    /// Helper method for implementing iterators
    fn next_c(&self, c: u8) -> Option<u8> {
        (c..THIN_CHARS as u8).find(|&c| self.has(c))
    }

    /// An iterator over keys used to access children
    fn keys(&self) -> KeyIter<&Self> {
        KeyIter::new(self)
    }

    /// An iterator over the child indices
    fn iter(&self) -> ChildIter<&Self> {
        ChildIter::new(self)
    }

    /// An iterator over `(c, idx)` where:<br>
    /// - `c: u8` the child value
    /// - `idx: usize` the child's index
    fn pairs(&self) -> PairIter<&Self> {
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

    /// Pops the next child pair from this node
    fn pop(&mut self) -> Option<(u8, Self::Idx)>
    where
        Self::Idx: Default,
    {
        let c = self.next_c(0)?;
        let idx = std::mem::take(self.get_mut(c));
        Some((c, idx))
    }
}

impl<N: WriteNode> WriteNode for &mut N {
    fn is_end_mut(&mut self) -> &mut bool {
        N::is_end_mut(self)
    }
    fn get_mut(&mut self, c: u8) -> &mut Self::Idx {
        N::get_mut(self, c)
    }
}
