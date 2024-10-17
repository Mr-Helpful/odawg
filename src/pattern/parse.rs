use super::Pattern;
use crate::utils::from_alpha;
use std::{fmt::Debug, str::FromStr};

pub struct ParseError {
  input: Box<str>,
  idx: usize,
  kind: ErrorKind,
}

/// The size of the view that's displayed in an error message
const VIEW_SIZE: usize = 50;
const VIEW_HALF: usize = VIEW_SIZE / 2;

impl Debug for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let (mut ps, mut pe) = (self.idx, self.input.len() - self.idx);
    (ps, pe) = match () {
      () if (ps < VIEW_HALF) & (pe < VIEW_HALF) => (ps, pe),
      () if (ps < VIEW_HALF) => (ps, VIEW_SIZE - ps),
      () if (pe < VIEW_HALF) => (VIEW_SIZE - pe, pe),
      () => (VIEW_HALF, VIEW_HALF),
    };

    writeln!(f, "{:?}\n", self.kind)?;
    writeln!(f, "{}", &self.input[self.idx - ps..self.idx + pe])?;
    writeln!(f, "{}", " ".repeat(ps) + "^")
  }
}

pub enum ErrorKind {
  UnclosedGroup,
  ReclosedGroup,
  UnclosedRange,
  Unexpected,
}

impl Debug for ErrorKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let message = match self {
      Self::UnclosedGroup => "Group was not closed",
      Self::ReclosedGroup => "Closed non-existant group",
      Self::UnclosedRange => "Range was not closed with a character",
      Self::Unexpected => "Unexpected character in Pattern",
    };
    write!(f, "{}", message)
  }
}

impl Pattern {
  fn range_mask(s: u8, e: u8) -> u32 {
    (1 << (e + 1)) - (1 << s)
  }

  fn parse_group(
    s: &str,
    chars: &mut impl Iterator<Item = (usize, char)>,
  ) -> Result<u32, ParseError> {
    let input = s.into();
    let mut mask = 0;
    let mut start = None;
    let mut is_range = false;

    for (idx, c) in chars {
      match c {
        'a'..='z' if is_range => {
          let v = from_alpha(c);
          let s = start.replace(v).unwrap_or(0);
          mask |= Self::range_mask(s, v);
          is_range = false
        }
        'a'..='z' => {
          let v = from_alpha(c);
          if let Some(v) = start.replace(v) {
            mask |= Self::range_mask(v, v)
          }
        }
        '-' if !is_range => is_range = true,

        // end of group, cleanup previous char/range
        ']' => {
          if is_range {
            mask |= Self::range_mask(start.unwrap_or(0), 25)
          } else if let Some(v) = start {
            mask |= Self::range_mask(v, v)
          }
          return Ok(mask);
        }

        // error state handling
        '-' if is_range => {
          return Err(ParseError {
            input,
            idx,
            kind: ErrorKind::UnclosedRange,
          })
        }
        '[' => {
          return Err(ParseError {
            input,
            idx,
            kind: ErrorKind::UnclosedGroup,
          })
        }
        _ => {
          return Err(ParseError {
            input,
            idx,
            kind: ErrorKind::Unexpected,
          })
        }
      }
    }

    Err(ParseError {
      input,
      idx: s.len(),
      kind: ErrorKind::UnclosedGroup,
    })
  }
}

impl FromStr for Pattern {
  type Err = ParseError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let input = s.into();
    let mut masks = vec![];

    let mut chars = s.chars().enumerate();
    while let Some((idx, c)) = chars.next() {
      match c {
        'a'..='z' => {
          let v = from_alpha(c);
          masks.push(Self::range_mask(v, v))
        }
        '-' => masks.push(Self::range_mask(0, 25)),
        '[' => masks.push(Self::parse_group(s, &mut chars)?),

        // error state handling
        ']' => {
          return Err(ParseError {
            input,
            idx,
            kind: ErrorKind::ReclosedGroup,
          })
        }
        _ => {
          return Err(ParseError {
            input,
            idx,
            kind: ErrorKind::Unexpected,
          })
        }
      }
    }

    Ok(Pattern(masks))
  }
}

#[cfg(test)]
mod test {
  use super::{super::strategies::pattern, Pattern};
  use proptest::prelude::*;

  proptest! {
    #[test]
    fn any_pattern(pat in pattern()) {
      let s = pat.to_string();
      let pat_: Pattern = s.parse().unwrap();
      assert_eq!(pat, pat_, "{} != {}", pat, pat_);
    }
  }

  #[test]
  fn empty() {
    let pat: Pattern = "".parse().unwrap();
    assert_eq!(pat.len(), 0);
    assert!(!pat.has("a"));
  }

  #[test]
  fn word() {
    let pat: Pattern = "jab".parse().unwrap();
    assert_eq!(pat.len(), 1);
    assert!(pat.has("jab"));
    assert!(!pat.has("fab"));
  }

  #[test]
  fn any() {
    let pat: Pattern = "-ab".parse().unwrap();
    assert_eq!(pat.len(), 26);
    assert!(pat.has("jab"));
    assert!(pat.has("fab"));
    assert!(pat.has("tab"));
    assert!(!pat.has("fib"));
  }

  #[test]
  fn group() {
    let pat: Pattern = "[fjt]ab".parse().unwrap();
    assert_eq!(pat.len(), 3);
    assert!(pat.has("jab"));
    assert!(pat.has("fab"));
    assert!(pat.has("tab"));
    assert!(!pat.has("fib"));
  }

  #[test]
  fn range() {
    let pat: Pattern = "[r-t]at".parse().unwrap();
    assert_eq!(pat.len(), 3);
    assert!(pat.has("rat"));
    assert!(pat.has("sat"));
    assert!(pat.has("tat"));
    assert!(!pat.has("sit"));
  }

  #[test]
  fn multi() {
    let pat: Pattern = "[b-cr-t]at".parse().unwrap();
    assert_eq!(pat.len(), 5);
    assert!(pat.has("bat"));
    assert!(pat.has("cat"));
    assert!(pat.has("rat"));
    assert!(pat.has("sat"));
    assert!(pat.has("tat"));
    assert!(!pat.has("sit"));
  }

  #[test]
  fn mixed() {
    let pat: Pattern = "[b-cpr-t]at".parse().unwrap();
    assert_eq!(pat.len(), 6);
    assert!(pat.has("bat"));
    assert!(pat.has("cat"));
    assert!(pat.has("pat"));
    assert!(pat.has("rat"));
    assert!(pat.has("sat"));
    assert!(pat.has("tat"));
    assert!(!pat.has("sit"));
  }

  #[test]
  fn many() {
    let pat: Pattern = "[r-t][ai]t".parse().unwrap();
    assert_eq!(pat.len(), 6);
    assert!(pat.has("rat"));
    assert!(pat.has("sat"));
    assert!(pat.has("tat"));
    assert!(pat.has("rit"));
    assert!(pat.has("sit"));
    assert!(pat.has("tit"));
    assert!(!pat.has("wit"));
  }

  #[test]
  fn start() {
    let pat: Pattern = "[-b]ye".parse().unwrap();
    assert_eq!(pat.len(), 2);
    assert!(pat.has("aye"));
    assert!(pat.has("bye"));
    assert!(!pat.has("dye"));
  }

  #[test]
  fn end() {
    let pat: Pattern = "[y-]ap".parse().unwrap();
    assert_eq!(pat.len(), 2);
    assert!(pat.has("yap"));
    assert!(pat.has("zap"));
    assert!(!pat.has("map"));
  }

  #[test]
  fn all() {
    let pat: Pattern = "[-cmr-ty-]ap".parse().unwrap();
    assert_eq!(pat.len(), 9);
    // I know this one isn't a word, I tried my best okay :(
    assert!(pat.has("aap"));
    assert!(pat.has("bap"));
    assert!(pat.has("cap"));
    assert!(pat.has("map"));
    assert!(pat.has("rap"));
    assert!(pat.has("sap"));
    assert!(pat.has("tap"));
    assert!(pat.has("yap"));
    assert!(pat.has("zap"));
    assert!(!pat.has("bat"));
  }
}
