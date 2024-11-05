use super::{IndexDawg, NodeIter, ReadNode, WordIter};
use crate::utils::convert::IntoLetters;

/// Methods for reading words from a DAWG
///
/// Almost all of these methods can have pretty decent default implementations.
pub trait ReadDawg: Sized + IndexDawg {
    /// Whether the Dawg is empty
    fn is_empty(&self) -> bool {
        NodeIter::from(self).all(|node| !node.is_end())
    }

    /// How many words are stored in the dag
    fn len(&self) -> usize {
        NodeIter::from(self).filter(ReadNode::is_end).count()
    }

    /// Whether this DAWG contains a given word
    fn has(&self, word: impl IntoLetters) -> bool {
        let mut idx = Self::ROOT_IDX;
        for c in word.letters() {
            let node = self.index(idx);
            if !node.has(c) {
                return false;
            }
            idx = node.get(c);
        }
        self.index(idx).is_end()
    }

    /// All the words contained in a given DAWG
    fn words(&self) -> impl Iterator<Item = Vec<u8>> {
        WordIter::from(self)
    }
}

impl<D: ReadDawg> ReadDawg for &D {
    fn is_empty(&self) -> bool {
        <D as ReadDawg>::is_empty(self)
    }
    fn len(&self) -> usize {
        <D as ReadDawg>::len(self)
    }
    fn has(&self, word: impl IntoLetters) -> bool {
        <D as ReadDawg>::has(self, word)
    }
    fn words(&self) -> impl Iterator<Item = Vec<u8>> {
        <D as ReadDawg>::words(self)
    }
}
