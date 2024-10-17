use super::{IndexMutDawg, ReadDawg};

/// Methods for adding and removing words in a DAWG
///
/// ## Note
///
/// The default implementations of `union`, `remove` and `intersect`<br>
/// are all pretty inefficient: override them if at all possible.
pub trait WriteDawg: IndexMutDawg {
  /// Adds a single word the DAWG, returning whether it existed
  fn add(&mut self, word: &impl AsRef<[u8]>) -> bool;

  /// Adds multiple words to the DAWG
  fn add_all<W: AsRef<[u8]>>(&mut self, words: impl IntoIterator<Item = W>) {
    for word in words {
      self.add(&word);
    }
  }

  /// Adds all words in another DAWG to `self`.
  fn union<D: ReadDawg>(&mut self, dawg: &D) {
    self.add_all(dawg.words());
  }

  /// Removes a single word from the DAWG, returning whether it existed
  fn sub(&mut self, word: &impl AsRef<[u8]>) -> bool;

  /// Removes multiple words from the DAWG
  fn sub_all<W: AsRef<[u8]>>(&mut self, words: impl IntoIterator<Item = W>) {
    for word in words {
      self.sub(&word);
    }
  }

  /// Removes all words in another DAWG from `self`.
  fn remove<D: ReadDawg>(&mut self, dawg: &D) {
    self.sub_all(dawg.words());
  }

  /// Only keeps words specified by `f(word)` in the trie
  fn keep(&mut self, f: impl Fn(&[u8]) -> bool);

  /// Intersects another DAWG with `self`, only keeping words in both DAWGs.
  fn intersect<D: ReadDawg>(&mut self, dawg: &D) {
    self.keep(|word| dawg.has(word))
  }
}
