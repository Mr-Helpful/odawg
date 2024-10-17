use super::{IndexDawg, ReadDawg, ReadNode};

pub struct WordIter<D: IndexDawg> {
  dawg: D,
  idxs: Vec<D::Idx>,
  word: Vec<u8>,
}

impl<D: ReadDawg> From<D> for WordIter<D> {
  fn from(value: D) -> Self {
    WordIter {
      dawg: value,
      idxs: vec![D::ROOT_IDX],
      word: vec![0],
    }
  }
}

impl<D: IndexDawg> Iterator for WordIter<D> {
  type Item = Vec<u8>;
  fn next(&mut self) -> Option<Vec<u8>> {
    loop {
      let (idx, c) = self.idxs.pop().zip(self.word.pop())?;
      let node = self.dawg.index(idx.clone());
      let Some(c) = node.next_c(c) else {
        // no nodes left to explore on current node
        // => we're currently backtracking
        if node.is_end() {
          return Some(self.word.clone());
        }
        continue;
      };

      self.word.push(c);
      self.idxs.push(idx);
      self.word.push(0);
      self.idxs.push(node.get(c));
    }
  }
}
