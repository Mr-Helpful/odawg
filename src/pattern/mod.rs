use crate::utils::convert::from_alpha;
use serde::{Deserialize, Serialize};

/// A word-like pattern, defined by the acceptable letters at each index.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pattern(Vec<u32>);

impl Pattern {
  pub fn len(&self) -> usize {
    if self.0.is_empty() {
      return 0;
    }
    self
      .0
      .iter()
      .map(|mask| mask.count_ones() as usize)
      .product()
  }

  fn valid_at(&self, idx: usize, c: char) -> bool {
    (self.0[idx] >> from_alpha(c) & 1) > 0
  }

  pub fn has<W: AsRef<str> + ?Sized>(&self, word: &W) -> bool {
    let word = word.as_ref();
    if word.is_empty() {
      return false;
    }
    if word.len() != self.0.len() {
      return false;
    }

    for (i, c) in word.chars().enumerate() {
      if !self.valid_at(i, c) {
        return false;
      }
    }
    true
  }

  pub fn words(&self) -> iters::WordIter<'_> {
    iters::WordIter::new(self)
  }
}

mod convert;
mod display;
mod iters;
mod parse;

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
