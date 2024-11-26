use super::ReadNode;
use std::array;

#[derive(Clone, Copy)]
pub struct AllNode<const NUM: usize, N>(pub(crate) [N; NUM]);

impl<const NUM: usize, N: ReadNode<Idx = usize>> ReadNode for AllNode<NUM, N> {
  type Idx = [usize; NUM];

  fn is_empty(&self) -> bool {
    self.0.iter().any(ReadNode::is_empty) || self.len() == 0
  }

  fn is_end(&self) -> bool {
    self.0.iter().all(ReadNode::is_end)
  }

  fn has(&self, c: u8) -> bool {
    self.0.iter().all(|node| node.has(c))
  }

  fn get(&self, c: u8) -> Self::Idx {
    array::from_fn(|i| self.0[i].get(c))
  }
}

#[cfg(test)]
mod test {
  use super::{super::ThinNode, AllNode};
  use crate::node::{
    thin::{test::thin_node, END_MASK},
    ReadNode,
  };
  use proptest::prelude::*;

  fn thin_mask(mask: u32) -> ThinNode {
    ThinNode { idx: 0, mask }
  }

  #[test]
  fn empty_intersect_is_empty() {
    let node0 = thin_mask(0b0);
    let node1 = thin_mask(0b0);
    let anode = AllNode([node0, node1]);
    assert!(anode.is_empty());
  }

  #[test]
  fn single_empty_is_empty() {
    let node0 = thin_mask(0b0);
    let node1 = thin_mask(0b1010);
    let node2 = thin_mask(0b1110);
    let anode = AllNode([node0, node1, node2]);
    assert!(anode.is_empty());
  }

  #[test]
  fn no_overlap_is_empty() {
    let node0 = thin_mask(0b0010);
    let node1 = thin_mask(0b1001);
    let anode = AllNode([node0, node1]);
    assert!(anode.is_empty());
  }

  #[test]
  fn overlap_not_empty() {
    let node0 = thin_mask(0b1011);
    let node1 = thin_mask(0b0110);
    let anode = AllNode([node0, node1]);
    assert!(!anode.is_empty());
  }

  #[test]
  fn unend_intersection_not_end() {
    let node0 = thin_mask(0b0);
    let node1 = thin_mask(END_MASK);
    let anode = AllNode([node0, node1]);
    assert!(!anode.is_end());
  }

  #[test]
  fn both_intersection_is_end() {
    let node0 = thin_mask(END_MASK);
    let node1 = thin_mask(END_MASK);
    let anode = AllNode([node0, node1]);
    assert!(anode.is_end());
  }

  proptest! {
    #[test]
    fn intersection_has_if_both(node0 in thin_node(), node1 in thin_node()) {
      let anode = AllNode([node0, node1]);
      for c in node0.keys().filter(|&c| node1.has(c)) {
        assert!(anode.has(c));
      }
    }

    #[test]
    fn intersection_get_pair(node0 in thin_node(), node1 in thin_node()) {
      let anode = AllNode([node0, node1]);
      for c in node0.keys().filter(|&c| node1.has(c)) {
        assert!(anode.get(c) == [node0.get(c), node1.get(c)]);
      }
    }

    #[test]
    fn intersection_keys(node0 in thin_node(), node1 in thin_node()) {
      let anode = AllNode([node0, node1]);
      for c in anode.keys() {
        assert!(node0.has(c) && node1.has(c));
        assert!(anode.get(c) == [node0.get(c), node1.get(c)]);
      }
    }
  }
}
