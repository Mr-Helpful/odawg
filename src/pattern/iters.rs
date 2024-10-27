use super::Pattern;

#[derive(Clone, Debug)]
pub struct WordIter<'a> {
  pat: &'a Pattern,
  masks: Vec<u32>,
  word: Vec<u8>,
}

impl<'a> WordIter<'a> {
  pub fn new(pat: &'a Pattern) -> Self {
    let mut new = Self {
      pat,
      masks: vec![],
      word: vec![],
    };
    new.rebuild_word();
    new
  }

  /// Gets the next character from a given mask
  fn next_c(mask: &mut u32) -> Option<u8> {
    if mask == &0 {
      return None;
    }
    let c: u8 = mask
      .trailing_zeros()
      .try_into()
      .expect("no more than 255 `0`s in a u32");
    *mask &= !(1 << c);
    Some(c)
  }

  /// Fills the word to the length of the pattern, using the pattern masks
  fn rebuild_word(&mut self) -> Option<Vec<u8>> {
    let Self { pat, word, masks } = self;
    for mut mask in pat.0[word.len()..pat.0.len()].iter().copied() {
      word.push(Self::next_c(&mut mask)?);
      masks.push(mask);
    }
    Some(word.clone())
  }

  /// Removes all empty masks from the end of the word and
  /// pops the next character for the last non-empty mask.
  fn increment_last(&mut self) {
    let Self { masks, word, .. } = self;

    // pop all non-empty masks from the back
    while let Some(&mask) = masks.last() {
      if mask > 0 {
        break;
      }
      word.pop();
      masks.pop();
    }

    if let Some((c, mask)) = word.last_mut().zip(masks.last_mut()) {
      *c = Self::next_c(mask).expect("by partition => mask > 0");
    }
  }
}

impl<'a> Iterator for WordIter<'a> {
  type Item = Vec<u8>;
  fn next(&mut self) -> Option<Self::Item> {
    if self.word.is_empty() {
      return None;
    }
    let word = self.rebuild_word()?;
    self.increment_last();
    Some(word)
  }
}

#[cfg(test)]
mod test {
  use super::{super::strategies::pattern_len, Pattern};
  use crate::utils::into_word;
  use proptest::prelude::*;

  proptest! {
    #[test]
    fn len(pat in pattern_len(10_000)) {
      let len = pat.len();
      let words: Vec<_> = pat.words().map(into_word).collect();
      assert_eq!(
        len, words.len(),
        "\n\
        ({pat}).len() = {len}\n\
        ({pat}).words() = {words:?}\
        "
      );
      for word in words {
        assert!(pat.has(&word));
      }
    }
  }

  #[test]
  fn empty() {
    let pat: Pattern = "".parse().unwrap();
    let words: Vec<_> = pat.words().map(into_word).collect();
    assert_eq!(words, vec![""; 0]);
  }

  #[test]
  fn word() {
    let pat: Pattern = "jab".parse().unwrap();
    let words: Vec<_> = pat.words().map(into_word).collect();
    assert_eq!(words, vec!["jab"]);
  }

  #[test]
  fn any() {
    let pat: Pattern = "-ab".parse().unwrap();
    let words: Vec<_> = pat.words().map(into_word).collect();
    assert_eq!(
      words,
      vec![
        "aab", "bab", "cab", "dab", "eab", "fab", "gab", "hab", "iab", "jab", "kab", "lab", "mab",
        "nab", "oab", "pab", "qab", "rab", "sab", "tab", "uab", "vab", "wab", "xab", "yab", "zab"
      ]
    )
  }

  #[test]
  fn all() {
    let pat: Pattern = "[-cmr-ty-]ap".parse().unwrap();
    let words: Vec<_> = pat.words().map(into_word).collect();
    assert_eq!(
      words,
      vec!["aap", "bap", "cap", "map", "rap", "sap", "tap", "yap", "zap"]
    )
  }
}
