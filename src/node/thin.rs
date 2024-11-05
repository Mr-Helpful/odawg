use super::{ReadNode, WideNode};
use serde::{Deserialize, Serialize};

/// The number of character that thin Dawg nodes support.<br>
/// Can be up to 31 characters at most, due to implementation constraints.
pub const THIN_CHARS: usize = 26;
pub(crate) const CHILD_MASK: u32 = (1 << THIN_CHARS) - 1;
pub(crate) const END_MASK: u32 = 1 << THIN_CHARS;

/// Memory efficient Dawg nodes.<br>
/// Relies on the assumption that all children are contiguous.
// @note leaf nodes serialize to be empty
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThinNode {
    pub(crate) idx: usize,
    pub(crate) mask: u32,
}

impl ReadNode for ThinNode {
    type Idx = usize;

    fn len(&self) -> usize {
        (self.mask & CHILD_MASK).count_ones() as usize
    }

    fn is_empty(&self) -> bool {
        (self.mask & CHILD_MASK) == 0
    }

    fn is_end(&self) -> bool {
        (self.mask & END_MASK) > 0
    }

    fn has(&self, i: u8) -> bool {
        ((self.mask >> i) & 1) > 0
    }

    fn get(&self, i: u8) -> Self::Idx {
        if !self.has(i) {
            return 0;
        }

        // mask away all children "above" `i`
        let masked = self.mask & ((1 << i) - 1);
        self.idx + (masked.count_ones() as Self::Idx)
    }

    fn next_c(&self, c: u8) -> Option<u8> {
        let masked = self.mask & !((1 << c) - 1);
        (masked > 0).then(|| masked.trailing_zeros() as u8)
    }
}

#[derive(Clone, Debug)]
pub struct NonContiguous;

impl TryFrom<WideNode<26>> for ThinNode {
    type Error = NonContiguous;
    fn try_from(value: WideNode<26>) -> Result<Self, Self::Error> {
        let mut mask = if value.end { END_MASK } else { 0 };

        let mut iter = value.pairs();
        let Some((c, idx)) = iter.next() else {
            return Ok(Self { mask, idx: 0 });
        };
        mask |= 1 << c;

        let mut c_idx = idx;
        for (c, idx) in iter {
            if c_idx + 1 != idx {
                return Err(NonContiguous);
            }

            c_idx = idx;
            mask |= 1 << c;
        }

        Ok(Self { idx, mask })
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::{ReadNode, ThinNode, CHILD_MASK, THIN_CHARS};
    use proptest::{bits::u32::sampled, prelude::*};
    use std::collections::HashSet;

    pub fn thin_mask(mask: u32) -> ThinNode {
        ThinNode { idx: 0, mask }
    }

    pub fn thin_node() -> BoxedStrategy<ThinNode> {
        any::<(u32, usize)>()
            .prop_map(|(mask, idx)| ThinNode {
                idx,
                mask: mask & CHILD_MASK,
            })
            .boxed()
    }

    #[test]
    fn len_eq_no_1s() {
        assert_eq!(thin_mask(0b000).len(), 0);
        assert_eq!(thin_mask(0b001).len(), 1);
        assert_eq!(thin_mask(0b100).len(), 1);
        assert_eq!(thin_mask(0b011).len(), 2);
    }

    pub fn thin_children(num: usize) -> BoxedStrategy<ThinNode> {
        sampled(num..=num, 0..26)
            .prop_map(|mask| ThinNode { idx: 0, mask })
            .boxed()
    }

    pub fn len_node_pair() -> BoxedStrategy<(usize, ThinNode)> {
        (0..26usize)
            .prop_flat_map(|num| (Just(num), thin_children(num)))
            .boxed()
    }

    proptest! {
      #[test]
      fn len_eq_bits((len, node) in len_node_pair()) {
        assert_eq!(node.len(), len);
      }
    }

    #[test]
    fn only_0_bits_is_empty() {
        assert!(thin_mask(0b0).is_empty());
        assert!(!thin_mask(0b001).is_empty());
        assert!(!thin_mask(0b100).is_empty());
        assert!(!thin_mask(0b011).is_empty());
    }

    proptest! {
      #[test]
      fn len_0_is_empty(node in thin_node()) {
        assert_eq!(node.len() == 0, node.is_empty());
      }
    }

    proptest! {
      #[test]
      fn has_keys(node in thin_node()) {
        for c in node.keys() {
          assert!(node.has(c));
        }
      }

      #[test]
      fn get_nonzero(node in thin_node()) {
        for c in node.keys() {
          assert_ne!(node.get(c), 0);
        }
      }

      #[test]
      fn not_has_gets_0(node in thin_node()) {
        for c in (0..THIN_CHARS as u8).filter(|&c| !node.has(c)) {
          assert_eq!(node.get(c), 0);
        }
      }

      #[test]
      fn keys_match(node in thin_node()) {
        let keys0: HashSet<_> = node.keys().collect();
        let keys1: HashSet<_> = (0..THIN_CHARS as u8).filter(|&c| node.has(c)).collect();
        assert_eq!(keys0, keys1);
      }

      /// the `next_c` should return the same as the default implementation
      #[test]
      fn next_c_matches((node, c) in (thin_node(), 0u8..THIN_CHARS as u8)) {
        let next = (c..26).find(|&c| node.has(c));
        assert_eq!(node.next_c(c), next)
      }
    }
}
