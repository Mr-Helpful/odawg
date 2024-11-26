use crate::{dawg::IndexDawg, utils::convert::IntoLetters, ReadDawg};
use serde::{Deserialize, Serialize};

mod letter;
use letter::{Letter, Node, CHILD_MASK};
mod convert;
mod display;
mod parse;

/// A word-like pattern, defined by the acceptable letters at each index.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pattern(Vec<u32>);

impl IndexDawg for Pattern {
  type Idx = usize;
  const ROOT_IDX: Self::Idx = 0;

  type NodeRef<'a> = Letter<'a>;
  fn index(&self, idx: Self::Idx) -> Self::NodeRef<'_> {
    Letter((idx < self.0.len()).then(|| Node {
      mask: &self.0[idx],
      next: idx + 1,
    }))
  }
}

impl ReadDawg for Pattern {
  fn is_empty(&self) -> bool {
    if self.0.is_empty() {
      return true;
    }
    self.0.iter().any(|mask| (mask & CHILD_MASK) == 0)
  }

  fn len(&self) -> usize {
    if self.0.is_empty() {
      return 0;
    }
    self
      .0
      .iter()
      .map(|mask| (mask & CHILD_MASK).count_ones() as usize)
      .product()
  }

  fn has(&self, word: impl IntoLetters) -> bool {
    let mut letters = word.letters();

    // any of masks doesn't contain letter => fail
    for mask in self.0.iter() {
      // iterators should have same length
      // => we should be able to fetch a letter
      let Some(c) = letters.next() else {
        return false;
      };
      if (mask & (1 << c)) == 0 {
        return false;
      }
    }

    // iterators should have same length
    // => we should have completely consumed letters
    letters.next().is_none()
  }
}

#[cfg(test)]
mod strategies {
  use super::*;
  use prop::{bits::u32::sampled, collection::vec};
  use proptest::prelude::*;

  const ALPHA_MASK: u32 = (1 << 26) - 1;

  /// Generates an arbitrary pattern.
  pub fn pattern() -> BoxedStrategy<Pattern> {
    any::<Vec<u32>>()
      .prop_map(|masks| Pattern(masks.into_iter().map(|mask| mask & ALPHA_MASK).collect()))
      .boxed()
  }

  /// Generates a pattern that contains at most `len` words.<br>
  /// This is an approximation at best, it'll often generate<br>
  /// way fewer than `len`, due to the cartesian product<br>
  /// nature of patterns.
  ///
  /// This can be used to ensure that tests that need to iterate<br>
  /// over the `.words()` iterator will terminate.
  pub fn pattern_len(max_len: usize) -> BoxedStrategy<Pattern> {
    let f64_len = max_len as f64;
    vec(0..26.min(max_len), 0..f64_len.log2() as usize)
      .prop_flat_map(move |nums| -> Vec<_> {
        let prod = nums.iter().product::<usize>() as f64;
        let frac = (f64_len.log2() / prod.log2()).min(1.0);
        nums
          .into_iter()
          .map(|num| (num as f64).powf(frac) as usize)
          .map(|norm_num| sampled(norm_num.min(26), 0..26))
          .collect()
      })
      .prop_map(Pattern)
      .boxed()
  }
}
