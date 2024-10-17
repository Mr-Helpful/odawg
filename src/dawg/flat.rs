use super::{
  IndexDawg, IndexMutDawg, ReadDawg, ReadNode, ThinNode, WideNode, WordIter, WriteDawg, WriteNode,
  THIN_CHARS,
};

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

/// A DAWG stored in a flattened list, where nodes store indexes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlatDawg<N>(pub(crate) Vec<N>);

impl<N: Default> Default for FlatDawg<N> {
  fn default() -> Self {
    Self(vec![Default::default()])
  }
}

mod read {
  use super::{FlatDawg, IndexDawg, ReadDawg, ReadNode, WordIter};

  impl<N: ReadNode<Idx = usize>> IndexDawg for FlatDawg<N> {
    type Idx = usize;
    const ROOT_IDX: Self::Idx = 0;

    type NodeRef<'a> = &'a N
    where
      Self: 'a;
    fn index(&self, idx: Self::Idx) -> Self::NodeRef<'_> {
      &self.0[idx]
    }
  }

  impl<N: ReadNode<Idx = usize>> ReadDawg for FlatDawg<N> {}

  impl<N: ReadNode<Idx = usize>> IntoIterator for FlatDawg<N> {
    type Item = Vec<u8>;
    type IntoIter = WordIter<FlatDawg<N>>;
    fn into_iter(self) -> Self::IntoIter {
      WordIter::from(self)
    }
  }
}

mod write {
  use super::{FlatDawg, IndexDawg, IndexMutDawg, ReadDawg, ReadNode, WriteDawg, WriteNode};

  impl<N: WriteNode<Idx = usize>> IndexMutDawg for FlatDawg<N> {
    type NodeMut<'a> = &'a mut N
    where
      Self: 'a;
    fn index_mut(&mut self, idx: Self::Idx) -> Self::NodeMut<'_> {
      &mut self.0[idx]
    }
  }

  impl<N: WriteNode<Idx = usize>> FlatDawg<N> {
    /// Helper function to insert an empty node and return the index
    pub(crate) fn insert(&mut self) -> usize
    where
      N: Default,
    {
      let i = self.0.len();
      self.0.push(Default::default());
      i
    }

    /// Helper function to call `f(idx, node, word)` every time a<br>
    /// depth first search would backtrack to a node `node`.
    pub(crate) fn on_backtrack(&mut self, mut f: impl FnMut(usize, &mut N, &[u8])) {
      let mut word = vec![0];
      let mut stack = vec![0];

      while let Some((idx, c)) = stack.pop().zip(word.pop()) {
        let node = self.index_mut(idx);
        if let Some(c) = node.next_c(c) {
          word.push(c);
          stack.push(idx);
          word.push(0);
          stack.push(node.get(c));
        } else {
          // no nodes left to explore on current node
          // => we're currently backtracking
          f(idx, node, &word);
          continue;
        };
      }
    }
  }

  impl<N: WriteNode<Idx = usize> + Default> WriteDawg for FlatDawg<N> {
    fn add(&mut self, word: &impl AsRef<[u8]>) -> bool {
      let word = word.as_ref();

      let mut idx = 0;
      for &c in word {
        let node = self.index(idx);
        idx = node.get(c);
        if idx == 0 {
          idx = self.insert();
        }
      }

      std::mem::replace(self.index_mut(idx).is_end_mut(), true)
    }

    fn union<D: ReadDawg>(&mut self, dawg: &D) {
      let mut stack = vec![(Self::ROOT_IDX, D::ROOT_IDX)];

      while let Some((idx0, idx1)) = stack.pop() {
        let node1 = dawg.index(idx1);

        *self.index_mut(idx0).is_end_mut() |= node1.is_end();
        for (c, idx) in node1.pairs() {
          if !self.index(idx0).has(c) {
            *self.index_mut(idx0).get_mut(c) = self.insert();
          }
          stack.push((self.index(idx0).get(c), idx))
        }
      }
    }

    fn sub(&mut self, word: &impl AsRef<[u8]>) -> bool {
      let word = word.as_ref();

      let mut idx = 0;
      for &c in word {
        idx = self.index(idx).get(c);
        if idx == 0 {
          return false;
        }
      }

      std::mem::replace(self.index_mut(idx).is_end_mut(), false)
    }

    fn remove<D: ReadDawg>(&mut self, dawg: &D) {
      let mut stack = vec![(Self::ROOT_IDX, D::ROOT_IDX)];

      while let Some((idx0, idx1)) = stack.pop() {
        let node0 = self.index_mut(idx0);
        let node1 = dawg.index(idx1);

        *node0.is_end_mut() &= !node1.is_end();
        let pairs: Vec<_> = node0.pairs().collect();
        for (c, idx) in pairs {
          if node1.has(c) {
            stack.push((idx, node1.get(c)))
          }
        }
      }
    }

    fn keep(&mut self, f: impl Fn(&[u8]) -> bool) {
      self.on_backtrack(|_, node, word| {
        if node.is_end() {
          *node.is_end_mut() = f(word)
        }
      })
    }

    fn intersect<D: ReadDawg>(&mut self, dawg: &D) {
      let mut stack = vec![(Self::ROOT_IDX, D::ROOT_IDX)];

      while let Some((idx0, idx1)) = stack.pop() {
        let node0 = self.index_mut(idx0);
        let node1 = dawg.index(idx1);

        *node0.is_end_mut() &= node1.is_end();
        let pairs: Vec<_> = node0.pairs().collect();
        for (c, idx) in pairs {
          if node1.has(c) {
            stack.push((idx, node1.get(c)))
          } else {
            *node0.get_mut(c) = 0
          }
        }
      }
    }
  }

  impl<W: AsRef<[u8]>, N: WriteNode<Idx = usize> + Default> Extend<W> for FlatDawg<N> {
    fn extend<T: IntoIterator<Item = W>>(&mut self, iter: T) {
      self.add_all(iter);
    }
  }

  impl<W: AsRef<[u8]>, N: WriteNode<Idx = usize> + Default> FromIterator<W> for FlatDawg<N> {
    fn from_iter<T: IntoIterator<Item = W>>(iter: T) -> Self {
      let mut dawg = Self::default();
      dawg.extend(iter);
      dawg
    }
  }
}

impl<N: WriteNode<Idx = usize>> FlatDawg<N> {
  /// Disconnects any nodes that don't have a marked end downstream.<br>
  /// Returns `self.empty()` on the resulting DAWG.
  pub fn trim(&mut self) -> bool {
    let mut stack = vec![];
    self.on_backtrack(|_, node, _| {
      let mut empty = !node.is_end();
      let keys: Vec<_> = node.keys().collect();

      for k in keys.into_iter().rev() {
        let c_empty = stack.pop().expect("should have emptiness info");
        if c_empty {
          *node.get_mut(k) = 0;
        }
        empty &= c_empty;
      }

      stack.push(empty)
    });
    !stack[0]
  }

  /// Minimises the size of the DAWG by reusing nodes whenever possible
  pub fn minimise(&mut self)
  where
    N: Hash + Eq + Clone,
  {
    // @note this could potentially be `HashMap<&N, usize>` to remove the
    // need to clone, but it lead to really wacky borrow checker issues
    // around the interior `while let Some((c0, mut idx0))` loop.
    let mut seen: HashMap<N, usize> = HashMap::new();
    self.on_backtrack(|idx, node, _| {
      for (c, mut idx0) in node.clone().pairs() {
        idx0 = *seen.entry(node.clone()).or_insert(idx0);
        *node.get_mut(c) = idx0;
      }
      seen.insert(node.clone(), idx);
    })
  }

  /// Makes all nodes connected to the root contiguous.<br>
  /// i.e. all children will appear besides each other.<br>
  /// Removes any nodes not connected to the root.
  pub fn make_contiguous(&mut self) {
    let mut idx_max = 1;
    let mut idx = 0;

    while idx < idx_max {
      let node = self.index(idx);
      let idxs: Vec<_> = node.iter().collect();
      let n_idxs = idxs.len();

      for (i, j) in (idx_max..).zip(idxs) {
        self.0.swap(i, j);
      }
      idx_max += n_idxs;
      idx += 1;
    }

    self.0.truncate(idx)
  }

  /// Makes the DAWG take up as little space as possible, by:
  /// 1. trimming any nodes that don't have a leaf node marked as an end.
  /// 2. minimising the number of nodes the DAWG uses by removing<br>
  ///    any duplicated nodes.
  /// 3. making all nodes contiguous, s.t. children are located together.
  /// 4. removing all nodes not connected to the root node.
  ///
  /// ## Warning
  ///
  /// If you're concerned about memory, this can double the amount of<br>
  /// memory the DAWG takes up. The DAWG will *usually* be significantly<br>
  /// reduced in size afterwards, but if you're in an environment with<br>
  /// limited memory, be careful with this one.
  pub fn clean(&mut self)
  where
    N: Default + Hash + Eq + Clone,
  {
    if self.trim() {
      // if we end up with an empty DAWG by trimming
      // we can short ciruit by setting to a Default
      *self = Default::default();
      return;
    }
    self.minimise();
    self.make_contiguous();
  }
}

impl From<FlatDawg<ThinNode>> for FlatDawg<WideNode<THIN_CHARS>> {
  fn from(value: FlatDawg<ThinNode>) -> Self {
    FlatDawg(value.0.into_iter().map(From::from).collect())
  }
}

impl From<FlatDawg<WideNode<THIN_CHARS>>> for FlatDawg<ThinNode> {
  fn from(mut value: FlatDawg<WideNode<THIN_CHARS>>) -> Self {
    value.make_contiguous();
    FlatDawg(
      value
        .0
        .into_iter()
        .map(|node| node.try_into().expect("nodes should be contiguous"))
        .collect(),
    )
  }
}
