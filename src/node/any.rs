use super::ReadNode;
use std::array;

#[derive(Clone, Copy)]
pub struct AnyNode<const NUM: usize, N>(pub(crate) [Option<N>; NUM]);

impl<const NUM: usize, N: ReadNode<Idx = usize>> ReadNode for AnyNode<NUM, N> {
    type Idx = [Option<usize>; NUM];

    fn is_empty(&self) -> bool {
        self.0
            .iter()
            .all(|node| node.as_ref().is_some_and(ReadNode::is_empty))
    }

    fn is_end(&self) -> bool {
        self.0
            .iter()
            .any(|node| node.as_ref().is_some_and(ReadNode::is_end))
    }

    fn has(&self, c: u8) -> bool {
        self.0
            .iter()
            .any(|node| node.as_ref().is_some_and(|node| node.has(c)))
    }

    fn get(&self, c: u8) -> Self::Idx {
        array::from_fn(|i| {
            let node = self.0[i].as_ref()?;
            let i = node.get(c);
            (i != 0).then_some(i)
        })
    }
}

#[cfg(test)]
mod test {
    use super::{super::ThinNode, AnyNode};
    use crate::node::{
        thin::{test::thin_node, END_MASK},
        ReadNode,
    };
    use proptest::prelude::*;

    fn thin_mask(mask: u32) -> ThinNode {
        ThinNode { idx: 0, mask }
    }

    #[test]
    fn empty_union_is_empty() {
        let node0 = thin_mask(0b0);
        let node1 = thin_mask(0b0);
        let anode = AnyNode([Some(node0), Some(node1)]);
        assert!(anode.is_empty());
    }

    #[test]
    fn single_empty_not_empty() {
        let node0 = thin_mask(0b0);
        let node1 = thin_mask(0b1);
        let anode = AnyNode([Some(node0), Some(node1)]);
        assert!(!anode.is_empty());
    }

    #[test]
    fn no_overlap_not_empty() {
        let node0 = thin_mask(0b0010);
        let node1 = thin_mask(0b1001);
        let anode = AnyNode([Some(node0), Some(node1)]);
        assert!(!anode.is_empty());
    }

    #[test]
    fn overlap_not_empty() {
        let node0 = thin_mask(0b1011);
        let node1 = thin_mask(0b0110);
        let anode = AnyNode([Some(node0), Some(node1)]);
        assert!(!anode.is_empty());
    }

    #[test]
    fn neither_union_not_end() {
        let node0 = thin_mask(0b0);
        let node1 = thin_mask(0b0);
        let anode = AnyNode([Some(node0), Some(node1)]);
        assert!(!anode.is_end());
    }

    #[test]
    fn unend_union_is_end() {
        let node0 = thin_mask(0b0);
        let node1 = thin_mask(END_MASK);
        let anode = AnyNode([Some(node0), Some(node1)]);
        assert!(anode.is_end());
    }

    #[test]
    fn both_intersection_is_end() {
        let node0 = thin_mask(END_MASK);
        let node1 = thin_mask(END_MASK);
        let anode = AnyNode([Some(node0), Some(node1)]);
        assert!(anode.is_end());
    }

    proptest! {
      #[test]
      fn union_has_if_either(node0 in thin_node(), node1 in thin_node()) {
        let anode = AnyNode([Some(node0), Some(node1)]);
        for c in node0.keys().chain(node1.keys()) {
          assert!(anode.has(c));
        }
      }

      #[test]
      fn intersection_get_pair(node0 in thin_node(), node1 in thin_node()) {
        let anode = AnyNode([Some(node0), Some(node1)]);
        for c in node0.keys().filter(|&c| node1.has(c)) {
          assert!(anode.get(c) == [Some(node0.get(c)), Some(node1.get(c))]);
        }
      }

      #[test]
      fn intersection_keys(node0 in thin_node(), node1 in thin_node()) {
        let anode = AnyNode([Some(node0), Some(node1)]);
        for c in anode.keys() {
          assert!(node0.has(c) || node1.has(c));
        }
      }
    }
}
