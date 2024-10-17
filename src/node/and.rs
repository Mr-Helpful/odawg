use super::ReadNode;
use std::array;

#[derive(Clone, Copy)]
pub struct AndNode<const NUM: usize, N>(pub(crate) [N; NUM]);

impl<const NUM: usize, N: ReadNode<Idx = usize>> ReadNode for AndNode<NUM, N> {
  type Idx = [usize; NUM];

  fn is_empty(&self) -> bool {
    self.0.iter().all(|node| node.is_empty())
  }

  fn is_end(&self) -> bool {
    self.0.iter().all(|node| node.is_end())
  }

  fn has(&self, c: u8) -> bool {
    self.0.iter().all(|node| node.has(c))
  }

  fn get(&self, c: u8) -> Self::Idx {
    array::from_fn(|i| self.0[i].get(c))
  }
}
