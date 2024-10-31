use super::MergeIter;

/// Trait for collections that support sorting the end into the collection.
pub trait EndSort<T> {
  /// Sorts all items in `self[from..]` into `self[..from]`
  fn sort_end(&mut self, from: usize)
  where
    T: Ord;
}

impl<T> EndSort<T> for Vec<T> {
  /// Sorts values from `from..len` into values from `0..from` in a vector.<br>
  /// Assumes that `0..from` has already been sorted.
  ///
  /// This can be acheived in `O(n log n + (m + n))` time, where:
  /// - `n`: the #of new values added `= len - from`
  /// - `m`: the #of existing values `= from`
  ///
  /// This is primarily intended for cases where `m >> n`,
  /// in which it acheives near-linear performance.
  ///
  /// However, the current version of this also uses `O(m + n)` space.<br>
  /// This is mostly because I'm **nowhere near** smart enough to implement in place merging.<br>
  /// If you do want to try implementing this, here's a few sources:<br>
  /// - [Practical Stackoverflow](https://stackoverflow.com/questions/2571049/how-to-sort-in-place-using-the-merge-sort-algorithm)
  /// - [Theoretic Stackoverflow](https://cstheory.stackexchange.com/questions/33913/most-efficient-inplace-merge-algorithms-stable-and-unstable)
  /// - [Stable Merging](https://academic.oup.com/comjnl/article/38/8/681/335248)
  /// - [Possible Rust implementation](https://gvelim.github.io/CSX0003RUST/merge_in_place.html)
  ///   - Seems to use virtual memory like techniques, which use extra memory?
  /// - [Literature Overview](https://nms.kcl.ac.uk/informatics/techreports/papers/TR-04-05.pdf)
  /// - [Pretty decent paper](https://dl.acm.org/doi/pdf/10.1145/42392.42403)
  ///
  /// Also, using an in-place merge would make it easier to generalise to
  /// `&mut impl AsMut<[T]>` rather than `&mut Vec<T>`. I'm pretty sure this is
  /// currently possible, but I can't work it out either.
  ///
  /// ```
  /// # use odawg::EndSort;
  /// //                       sorted | unsorted
  /// let values = vec![1, 2, 4, 7, 8 , 5, 3, 6];
  ///
  /// let mut values_ = values.clone();
  /// values_.sort_end(4);
  /// assert_eq!(values_, vec![1, 2, 3, 4, 5, 6, 7, 8]);
  ///
  /// let mut values_ = values.clone();
  /// values_.sort_end(5);
  /// assert_eq!(values_, vec![1, 2, 3, 4, 5, 6, 7, 8]);
  ///
  /// let mut values_ = values.clone();
  /// values_.sort_end(6);
  /// assert_eq!(values_, vec![1, 2, 3, 4, 6, 7, 8, 5]);
  /// ```
  fn sort_end(&mut self, from: usize)
  where
    T: Ord,
  {
    let mut tail = self.split_off(from);
    tail.sort();
    *self = MergeIter::new(self.drain(..), tail.drain(..)).collect();
  }
}

#[cfg(test)]
mod test {
  use crate::utils::{EndSort, IsSorted};
  use proptest::prelude::*;

  proptest! {
    #[test]
    fn sorted_is_sorted(mut vec: Vec<usize>) {
      vec.sort();
      assert!(IsSorted::sorted(&mut vec.iter()));
    }

    #[test]
    fn sort_end_matches_sort(mut vec: Vec<usize>, mut i: usize) {
      i = if vec.is_empty() {0} else {i % vec.len()};
      let mut vec1 = vec.clone();
      vec1[0..i].sort();
      vec1.sort_end(i);

      vec.sort();
      assert_eq!(vec, vec1);
    }
  }
}
