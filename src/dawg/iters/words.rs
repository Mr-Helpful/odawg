use super::{IndexDawg, ReadDawg, ReadNode};

pub struct WordIter<D: IndexDawg> {
  dawg: D,
  stack: Vec<(D::Idx, Vec<u8>)>,
  word: Vec<u8>,
}

impl<D: ReadDawg> From<D> for WordIter<D> {
  fn from(value: D) -> Self {
    let pair = (D::ROOT_IDX, value.index(D::ROOT_IDX).keys().collect());
    WordIter {
      dawg: value,
      stack: vec![pair],
      word: vec![],
    }
  }
}

impl<D: IndexDawg> Iterator for WordIter<D> {
  type Item = Vec<u8>;
  fn next(&mut self) -> Option<Vec<u8>> {
    while let Some((idx, mut keys)) = self.stack.pop() {
      if let Some(c) = keys.pop() {
        self.stack.push((idx.clone(), keys));

        let c_idx = self.dawg.index(idx.clone()).get(c);
        let keys = self.dawg.index(c_idx.clone()).keys().collect();

        self.word.push(c);
        self.stack.push((c_idx, keys));
        continue;
      }

      let word = self.word.clone();
      self.word.pop();
      if self.dawg.index(idx.clone()).is_end() {
        // no nodes left to explore on current node
        // => we're currently backtracking
        return Some(word);
      }
    }

    None
  }
}
