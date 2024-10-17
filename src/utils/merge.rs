pub struct MergeIter<I: Iterator> {
  next_tup: (Option<I::Item>, Option<I::Item>),
  iters: (I, I),
}

impl<I: Iterator> MergeIter<I> {
  pub fn new(ix: impl IntoIterator<IntoIter = I>, iy: impl IntoIterator<IntoIter = I>) -> Self {
    let mut ix = ix.into_iter();
    let mut iy = iy.into_iter();
    Self {
      next_tup: (ix.next(), iy.next()),
      iters: (ix, iy),
    }
  }
}

impl<I: IntoIterator> From<(I, I)> for MergeIter<I::IntoIter> {
  fn from((x, y): (I, I)) -> Self {
    Self::new(x, y)
  }
}

impl<I: Iterator> Iterator for MergeIter<I>
where
  I::Item: Ord,
{
  type Item = I::Item;
  fn next(&mut self) -> Option<Self::Item> {
    let (x, y) = &mut self.next_tup;
    let (ix, iy) = &mut self.iters;

    if (x.is_some() && x < y) || (x.is_some() && y.is_none()) {
      std::mem::replace(x, ix.next())
    } else {
      std::mem::replace(y, iy.next())
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let (x, y) = &self.next_tup;
    let ex = if x.is_some() { 1 } else { 0 };
    let ey = if y.is_some() { 1 } else { 0 };
    let extra = ex + ey;

    let (ix, iy) = &self.iters;
    let (min_x, max_x) = ix.size_hint();
    let (min_y, max_y) = iy.size_hint();
    (
      min_x + min_y + extra,
      max_x.zip(max_y).map(|(x, y)| x + y + extra),
    )
  }
}

#[cfg(test)]
mod test {
  use super::{super::IsSorted, MergeIter};
  use proptest::prelude::*;

  proptest! {
    #[test]
    fn preserves_len(vec1: Vec<usize>, vec2: Vec<usize>) {
      let (len1, len2) = (vec1.len(), vec2.len());
      let merged: Vec<_> = MergeIter::new(vec1, vec2).collect();
      assert_eq!(len1 + len2, merged.len());
    }

    #[test]
    fn merges_sorted(mut vec1: Vec<usize>, mut i: usize) {
      i = if vec1.is_empty() {0} else {i % vec1.len()};
      let mut vec2 = vec1.split_off(i);
      vec1.sort();
      vec2.sort();

      let merged: Vec<_> = MergeIter::new(vec1, vec2).collect();
      assert!(IsSorted::sorted(&mut merged.iter()));
    }
  }
}
