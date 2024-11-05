use super::{ReadNode, ThinNode, WriteNode, THIN_CHARS};
use crate::utils::{convert::into_alpha, serde_array};
use serde::{Deserialize, Serialize};
use std::{array, fmt::Display};

/// A full width node, capable of representing children<br>
/// that are potentially non-contiguous.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WideNode<const N: usize = THIN_CHARS> {
  pub(crate) end: bool,
  #[serde(with = "serde_array")]
  pub(crate) children: [usize; N],
}

impl<const N: usize> Default for WideNode<N> {
  fn default() -> Self {
    Self {
      end: false,
      children: [0; N],
    }
  }
}

impl<const N: usize> Display for WideNode<N> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let end_str = if self.is_end() { "end]" } else { "   ]" };
    write!(f, "{end_str}")?;
    for c in self.keys() {
      write!(f, " {}: {}", into_alpha(c), self.get(c))?;
    }
    Ok(())
  }
}

impl<const N: usize> ReadNode for WideNode<N> {
  type Idx = usize;

  fn len(&self) -> usize {
    self.children.iter().map(|&idx| usize::from(idx > 0)).sum()
  }

  fn is_empty(&self) -> bool {
    self.children == [0; N]
  }

  fn is_end(&self) -> bool {
    self.end
  }

  fn has(&self, c: u8) -> bool {
    self.children[c as usize] > 0
  }

  fn get(&self, c: u8) -> Self::Idx {
    self.children[c as usize]
  }
}

impl<const N: usize> From<ThinNode> for WideNode<N> {
  fn from(value: ThinNode) -> Self {
    Self {
      end: value.is_end(),
      children: array::from_fn(|i| value.get(i as u8)),
    }
  }
}

impl<const N: usize> WriteNode for WideNode<N> {
  fn is_end_mut(&mut self) -> &mut bool {
    &mut self.end
  }

  fn get_mut(&mut self, c: u8) -> &mut Self::Idx {
    &mut self.children[c as usize]
  }
}

#[cfg(test)]
pub(crate) mod test {
  use super::{ReadNode, WideNode};
  use proptest::{bits::u32::sampled, collection::vec, prelude::*, sample::SizeRange};
  use std::{array, collections::HashSet};

  fn wide_children(num: impl Into<SizeRange>) -> BoxedStrategy<WideNode> {
    (sampled(num, 0..26), vec(1usize.., 26..=26))
      .prop_map(|(mask, idxs)| array::from_fn(|i| if (mask >> i) & 1 > 0 { idxs[i] } else { 0 }))
      .prop_flat_map(|idxs| (any::<bool>(), Just(idxs)))
      .prop_map(|(end, idxs)| WideNode {
        end,
        children: idxs,
      })
      .boxed()
  }

  fn len_node_pair() -> BoxedStrategy<(usize, WideNode)> {
    (0..=26usize)
      .prop_flat_map(|num| (Just(num), wide_children(num..=num)))
      .boxed()
  }

  pub fn wide_node() -> BoxedStrategy<WideNode> {
    wide_children(26)
  }

  proptest! {
    #[test]
    fn len_matches((len, node) in len_node_pair()) {
      assert_eq!(node.len(), len);
    }
  }

  proptest! {
    #[test]
    fn len_0_is_empty(node in wide_children(26)) {
      assert_eq!(node.len() == 0, node.is_empty());
    }
  }

  proptest! {
    #[test]
    fn has_keys(node in wide_node()) {
      for c in node.keys() {
        assert!(node.has(c));
      }
    }

    #[test]
    fn get_nonzero(node in wide_node()) {
      for c in node.keys() {
        assert_ne!(node.get(c), 0);
      }
    }

    #[test]
    fn not_has_gets_0(node in wide_node()) {
      for c in (0..26).filter(|&c| !node.has(c)) {
        assert_eq!(node.get(c), 0);
      }
    }

    #[test]
    fn keys_match(node in wide_node()) {
      let keys0: HashSet<_> = node.keys().collect();
      let keys1: HashSet<_> = (0..26).filter(|&c| node.has(c)).collect();
      assert_eq!(keys0, keys1);
    }
  }
}
