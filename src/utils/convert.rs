use std::ops::Range;

pub const ALPHA_START: u8 = b'a';
pub const ALPHA_CHARS: u8 = 26;
const ALPHA_RANGE: Range<u8> = ALPHA_START..(ALPHA_START + ALPHA_CHARS);

/// Translates from an lowercase alphabetic character to a `u8` letter
pub fn from_alpha(c: char) -> u8 {
  debug_assert!(
    ALPHA_RANGE.contains(&(c as u8)),
    "Character '{c}' out the alphabetic range \"a-z\"",
  );
  c as u8 - ALPHA_START
}

/// Translates from a word comprised of alphabetic characters to `u8` letters
pub fn from_word(s: impl AsRef<str>) -> Vec<u8> {
  s.as_ref().chars().map(from_alpha).collect()
}

/// Translates from a `u8` letter to a alphabetic character
pub fn into_alpha(v: u8) -> char {
  debug_assert!(
    (0..ALPHA_CHARS).contains(&v),
    "Value {v} out of supported range 0-25"
  );
  (v + ALPHA_START) as char
}

/// Translates from a `u8` slice to an alphabetic word
pub fn into_word(v: impl AsRef<[u8]>) -> String {
  v.as_ref().iter().copied().map(into_alpha).collect()
}

/// A type that can be represented as a
/// collection of items convertible into u8 letters
pub trait IntoLetters {
  type LetterIter<'a>: Iterator<Item = u8> + 'a
  where
    Self: 'a;
  fn letters(&self) -> Self::LetterIter<'_>;
}

impl<I: IntoLetters> IntoLetters for &I {
  type LetterIter<'b> = I::LetterIter<'b> where Self: 'b;
  fn letters(&self) -> Self::LetterIter<'_> {
    I::letters(self)
  }
}

impl IntoLetters for &[u8] {
  type LetterIter<'b> = std::iter::Copied<std::slice::Iter<'b, u8>> where Self: 'b;
  fn letters(&self) -> Self::LetterIter<'_> {
    self.iter().copied()
  }
}
impl IntoLetters for Vec<u8> {
  type LetterIter<'b> = std::iter::Copied<std::slice::Iter<'b, u8>> where Self: 'b;
  fn letters(&self) -> Self::LetterIter<'_> {
    self.iter().copied()
  }
}

impl IntoLetters for &str {
  type LetterIter<'b> = ConvertIter<'b> where Self: 'b;
  fn letters(&self) -> Self::LetterIter<'_> {
    ConvertIter(self.chars())
  }
}
impl IntoLetters for String {
  type LetterIter<'b> = ConvertIter<'b> where Self: 'b;
  fn letters(&self) -> Self::LetterIter<'_> {
    ConvertIter(self.chars())
  }
}

pub struct ConvertIter<'a>(std::str::Chars<'a>);

impl<'a> Iterator for ConvertIter<'a> {
  type Item = u8;
  fn next(&mut self) -> Option<Self::Item> {
    self.0.next().map(from_alpha)
  }
}
