use super::ReadNode;
use std::array;

#[derive(Clone, Copy)]
pub struct OrNode<const NUM: usize, N>(pub(crate) [Option<N>; NUM]);

impl<const NUM: usize, N: ReadNode<Idx = usize>> ReadNode for OrNode<NUM, N> {
  type Idx = [Option<usize>; NUM];

  fn is_empty(&self) -> bool {
    self
      .0
      .iter()
      .any(|node| node.as_ref().is_some_and(|node| node.is_empty()))
  }

  fn is_end(&self) -> bool {
    self
      .0
      .iter()
      .any(|node| node.as_ref().is_some_and(|node| node.is_end()))
  }

  fn has(&self, c: u8) -> bool {
    self
      .0
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
