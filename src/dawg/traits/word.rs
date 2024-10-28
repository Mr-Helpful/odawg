use crate::utils::convert::from_alpha;

pub trait IntoLetters {
  type LetterIter: Iterator<Item = u8>;
  fn letters(self) -> Self::LetterIter;
}

impl<'a> IntoLetters for &'a str {
  type LetterIter = StrLetters<'a>;
  fn letters(self) -> Self::LetterIter {
    StrLetters(self.chars())
  }
}

pub struct StrLetters<'a>(std::str::Chars<'a>);

impl<'a> Iterator for StrLetters<'a> {
  type Item = u8;
  fn next(&mut self) -> Option<Self::Item> {
    let c = self.0.next()?;
    Some(from_alpha(c))
  }
}

impl<'a> IntoLetters for &'a [u8] {
  type LetterIter = std::iter::Copied<std::slice::Iter<'a, u8>>;
  fn letters(self) -> Self::LetterIter {
    self.iter().copied()
  }
}
