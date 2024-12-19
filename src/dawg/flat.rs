use crate::node::NonContiguous;

use super::{
    IndexDawg, IndexMutDawg, ReadDawg, ReadNode, ThinNode, WideNode, WordIter, WriteDawg,
    WriteNode, THIN_CHARS,
};

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Display, Write},
    hash::Hash,
};

/// A DAWG stored in a flattened list, where nodes store indexes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlatDawg<N = WideNode>(pub Vec<N>);

impl<N: Default> Default for FlatDawg<N> {
    fn default() -> Self {
        Self(vec![Default::default()])
    }
}

impl<N: Display> Display for FlatDawg<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pad = (self.0.len() as f64).log10().ceil() as usize;
        for (i, node) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_char('\n')?;
            }
            write!(f, "{i: >pad$} {node}")?;
        }
        Ok(())
    }
}

impl<N: ReadNode<Idx = usize>> FlatDawg<N> {
    /// Returns whether the DAWG has a cycle in it.<br>
    /// This can and **should** be used to catch errors.
    pub fn is_cyclic(&self) -> bool {
        let mut path = vec![];
        let mut idx = Self::ROOT_IDX;
        let mut keys: Vec<_> = self.index(Self::ROOT_IDX).keys().collect();

        loop {
            if let Some(&key) = keys.first() {
                // we've seen this node in the path => cycle!
                if path.iter().any(|&(i, _)| i == idx) {
                    return true;
                }

                path.push((idx, keys));
                idx = self.index(idx).get(key);
                keys = self.index(idx).keys().collect();
                continue;
            }

            // last item empty => backtrack
            let Some(item) = path.pop() else { break };
            (idx, keys) = item;
            keys.pop(); // we've fully explored this branch, remove it.
        }

        false
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
        pub(crate) fn on_backtrack(&mut self, mut f: impl FnMut(usize, &mut N, &[u8]))
        where
            N: Clone,
        {
            let mut word = vec![0];
            let mut stack = vec![(Self::ROOT_IDX, self.index(Self::ROOT_IDX).clone())];

            while let Some((idx, mut node)) = stack.pop() {
                word.pop();

                if let Some((c, c_idx)) = node.pop() {
                    word.push(c);
                    stack.push((idx, node));
                    word.push(0);
                    stack.push((c_idx, self.index(c_idx).clone()));
                } else {
                    // no nodes left to explore on current node
                    // => we're currently backtracking
                    f(idx, self.index_mut(idx), &word);
                };
            }
        }
    }

    impl<N: WriteNode<Idx = usize> + Default + Clone> WriteDawg for FlatDawg<N> {
        fn add(&mut self, word: impl AsRef<[u8]>) -> bool {
            let word = word.as_ref();

            let mut idx = 0;
            for &c in word {
                let node = self.index(idx);
                if node.get(c) == 0 {
                    let n_idx = self.insert();
                    let node = self.index_mut(idx);
                    *node.get_mut(c) = n_idx;
                }
                idx = self.index(idx).get(c);
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
                    stack.push((self.index(idx0).get(c), idx));
                }
            }
        }

        fn sub(&mut self, word: impl AsRef<[u8]>) -> bool {
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
                        stack.push((idx, node1.get(c)));
                    }
                }
            }
        }

        fn keep(&mut self, f: impl Fn(&[u8]) -> bool) {
            self.on_backtrack(|_, node, word| {
                if node.is_end() {
                    *node.is_end_mut() = f(word);
                }
            });
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
                        stack.push((idx, node1.get(c)));
                    } else {
                        *node0.get_mut(c) = 0;
                    }
                }
            }
        }
    }

    impl<W: AsRef<[u8]>, N: WriteNode<Idx = usize> + Default + Clone> Extend<W> for FlatDawg<N> {
        fn extend<T: IntoIterator<Item = W>>(&mut self, iter: T) {
            self.add_all(iter);
        }
    }

    impl<W: AsRef<[u8]>, N: WriteNode<Idx = usize> + Default + Clone> FromIterator<W> for FlatDawg<N> {
        fn from_iter<T: IntoIterator<Item = W>>(iter: T) -> Self {
            let mut dawg = Self::default();
            dawg.extend(iter);
            dawg
        }
    }
}

impl<N: WriteNode<Idx = usize> + Clone + std::fmt::Debug + std::fmt::Display> FlatDawg<N> {
    /// Disconnects any nodes that don't have a marked end downstream.<br>
    /// Returns `self.empty()` on the resulting DAWG.
    pub fn unlink(&mut self) -> bool {
        debug_assert!(!self.0.is_empty());
        debug_assert!(!self.is_cyclic(), "dawg is cyclic!\n{self}");
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

            stack.push(empty);
        });
        stack[0]
    }

    /// Minimises the size of the DAWG by reusing nodes whenever possible
    pub fn minimise(&mut self)
    where
        N: Hash + Eq + Clone,
    {
        debug_assert!(!self.0.is_empty());
        debug_assert!(!self.is_cyclic(), "dawg is cyclic!\n{self}");
        // @note this could potentially be `HashMap<&N, usize>` to remove the
        // need to clone, but it lead to really wacky borrow checker issues
        // around the interior `for (c, mut idx0)` loop and the use of `entry`
        let mut seen: HashMap<N, usize> = HashMap::new();
        let mut stack = vec![(0, self.index(0).clone())];

        while let Some(&mut (idx, ref mut node)) = stack.last_mut() {
            if let Some((_, c_idx)) = node.pop() {
                stack.push((c_idx, self.index(c_idx).clone()));
                continue;
            }
            stack.pop();

            // backtracking, perform minimisation
            for (c, c_idx) in self.index(idx).clone().pairs() {
                if let Some(&n_idx) = seen.get(self.index(c_idx)) {
                    *(self.index_mut(idx).get_mut(c)) = n_idx;
                }
            }

            if !seen.contains_key(self.index(idx)) {
                seen.insert(self.index(idx).clone(), idx);
            }
        }
    }

    /// Sorts the nodes of a DAWG in rough breadth first order<br>
    /// and then removes any nodes not connected to the root node.
    pub fn trim(&mut self) {
        // Look, there's a fair few comments in here,<br>
        // I've left headers to give the rough outline:
        // 1. Generate breadth first order indices
        // 2. Reorder nodes according to indices
        // 3. Update indices to point to correct nodes
        //
        // The majority of the other comments are safety justifications,<br>
        // these use some loose propositional logic to prove safety.
        debug_assert!(!self.0.is_empty(), "dawg is empty");
        debug_assert!(!self.is_cyclic(), "dawg is cyclic!\n{self}");
        let mut idx_map = vec![0; self.0.len()];
        let mut idxs = vec![Self::ROOT_IDX];

        // # Generate a breadth first order
        for i in 0.. {
            let Some(&idx) = idxs.get(i) else { break };

            for c_idx in self.index(idx).iter() {
                if idx_map[c_idx] > 0 {
                    continue;
                }
                idx_map[c_idx] = idxs.len();
                idxs.push(c_idx);
            }
        }
        // As idx_map items are set from idxs.len() and idxs.len()<br>
        // changes every time it is set:
        // 1. all 1..idxs.len() are present in idx_map
        // 2. each 1..idxs.len() is only present once in idx_map

        // # Reorder nodes and update links
        //
        // @note this could theoretically be done with only safe behaviour,
        // by swapping the nodes and updating in place, however:
        // 1. I've tried this a lot of times, with no success
        // 2. I think this implementation may possibly be faster
        let mut nodes = Vec::with_capacity(idxs.len());
        let slot_ptr: *mut N = nodes.as_mut_ptr();

        let mut pairs = idx_map.iter().zip(self.0.drain(..));
        let (_, node) = pairs.next().unwrap(/* nodes is non-empty */);
        unsafe {
            // # Safety
            //
            // idxs is initialised to vec![Self::ROOT_IDX]
            // & idxs length is never reduced (only `push` is called)
            // -> 0 < idxs.len()
            // -> 0 < slots.capacity()
            // -> slots[0] is reserved
            // -> slot_ptr is reserved
            // -> slot_ptr is safe to write to
            *slot_ptr = node
        }

        for (&i, node) in pairs {
            if i == 0 {
                continue;
            }

            unsafe {
                // # Safety
                //
                // (i > 0, _) in pairs
                // -> i > 0 in idx_map
                //   {by #1 after breadth first search}
                // -> i in 1..idxs.len()
                // -> i < idxs.len()
                // -> i < slots.capacity()
                // -> slots[i] is reserved
                // -> slot_ptr.add(i) is reserved
                // -> slot_ptr.add(i) is safe to write to
                *slot_ptr.add(i) = node
            }
        }

        unsafe {
            // # Safety
            //
            //   {by for loop above}
            // pairs.all(|(i, _)| i > 0 -> slots[i] has been initialised)
            //   {by definition of pairs}
            // -> idx_map.all(|i| i > 0 -> slots[i] has been initialised)
            //   {by #2, all i in idx_map are < idxs.len()}
            // -> idx_map.all(|i| i in 1..idxs.len() -> slot[i] has been initialised)
            //   {by #1, all i in 1..idxs.len() are present in idx_map}
            // -> (1..idxs.len()).all(|i| slots[i] has been initialised)
            //   {by manually setting slots[0]}
            // -> (0..idxs.len()).all(|i| slots[i] has been initialised)
            //   {slice defintions}
            // -> slots[0..idxs.len()] is initialised
            //   {by safety in `set_len`, with old_len = 0}
            // -> slots.set_len(idxs.len()) is safe
            nodes.set_len(idxs.len())
        }

        // # Update indices
        for node in nodes.iter_mut() {
            for (k, c_idx) in node.clone().pairs() {
                *node.get_mut(k) = idx_map[c_idx];
            }
        }
        self.0 = nodes;
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
        if self.unlink() {
            // if we end up with an empty DAWG by unlinking
            // we can short ciruit by setting to a Default
            *self = FlatDawg::default();
            return;
        }
        self.minimise();
        self.trim();
    }
}

impl From<FlatDawg<ThinNode>> for FlatDawg<WideNode<THIN_CHARS>> {
    fn from(value: FlatDawg<ThinNode>) -> Self {
        FlatDawg(value.0.into_iter().map(From::from).collect())
    }
}

impl TryFrom<FlatDawg<WideNode<THIN_CHARS>>> for FlatDawg<ThinNode> {
    type Error = NonContiguous;
    fn try_from(mut value: FlatDawg<WideNode<THIN_CHARS>>) -> Result<Self, Self::Error> {
        value.trim();
        Ok(FlatDawg(
            value
                .0
                .into_iter()
                .map(|node| node.try_into())
                .collect::<Result<_, _>>()?,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::FlatDawg;
    use crate::{
        utils::convert::{from_word, into_word},
        ReadDawg, WideNode, WriteDawg,
    };
    use prop::collection::vec;
    use proptest::prelude::*;

    fn dawg_word() -> BoxedStrategy<Vec<u8>> {
        vec(0..26u8, 0..100).boxed()
    }
    fn dawg_words() -> BoxedStrategy<Vec<Vec<u8>>> {
        vec(dawg_word(), 0..100).boxed()
    }

    proptest! {
      #[test]
      fn collect(words in dawg_words()) {
        let _: FlatDawg<WideNode> = words.into_iter().collect();
      }

      #[test]
      fn len(words in dawg_words()) {
        let dawg: FlatDawg = words.clone().into_iter().collect();
        let words_: Vec<_> = dawg.words().collect();

        let strings: Vec<_> = dawg.words().map(into_word).collect();
        assert_eq!(dawg.len(), words_.len(), "all words = {strings:?}")
      }

      #[test]
      fn collect_iter(words in dawg_words()) {
        let dawg: FlatDawg = words.clone().into_iter().collect();

        let mut i_words: Vec<_> = words.into_iter().map(into_word).collect();
        let mut d_words: Vec<_> = dawg.words().map(into_word).collect();
        i_words.sort();
        i_words.dedup();
        d_words.sort();
        assert_eq!(i_words, d_words);
      }

      #[test]
      fn minimise_len(words in dawg_words()) {
        let mut dawg: FlatDawg = words.clone().into_iter().collect();
        dawg.minimise();
        dawg.trim();
        debug_assert!(!dawg.is_cyclic(), "dawg is cyclic!\n{dawg}");

        let mut i_words: Vec<_> = words.into_iter().map(into_word).collect();
        let mut d_words: Vec<_> = dawg.words().map(into_word).collect();
        i_words.sort();
        i_words.dedup();
        d_words.sort();
        assert_eq!(i_words, d_words)
      }
    }

    #[test]
    fn minimise_deletes_duplicates() {
        let mut dawg: FlatDawg = Default::default();
        dawg.add(from_word("cat"));
        dawg.add(from_word("cut"));

        dawg.minimise();
        dawg.trim();
        assert_eq!(dawg.0.len(), 4);
    }
}
