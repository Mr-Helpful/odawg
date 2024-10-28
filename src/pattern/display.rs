use super::Pattern;
use crate::utils::convert::into_alpha;
use std::fmt::{Display, Write};

impl Pattern {
  fn fmt_range(start: u8, end: u8, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if start == end {
      return f.write_char(into_alpha(start));
    }
    if start == 0 {
      return write!(f, "-{}", into_alpha(end));
    }
    if end == 25 {
      return write!(f, "{}-", into_alpha(start));
    }
    write!(f, "{}-{}", into_alpha(start), into_alpha(end))
  }

  fn fmt_mask(mut mask: u32, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if mask.count_ones() == 0 {
      return f.write_str("[]");
    }
    if mask.count_ones() == 1 {
      return f.write_char(into_alpha(
        mask
          .trailing_zeros()
          .try_into()
          .expect("no more than 255 `0`s in a u32"),
      ));
    }
    if mask.count_ones() == 26 {
      return f.write_char('-');
    }

    f.write_char('[')?;
    let mut i = 0;
    while mask.count_ones() > 0 {
      let s: u8 = mask
        .trailing_zeros()
        .try_into()
        .expect("no more than 255 `0`s in a u32");
      mask >>= s;
      let l: u8 = mask
        .trailing_ones()
        .try_into()
        .expect("no more than 255 `1`s in a u32");
      mask >>= l;

      i += s;
      Self::fmt_range(i, i + l - 1, f)?;
      i += l;
    }
    f.write_char(']')
  }
}

impl Display for Pattern {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for &mask in &self.0 {
      Self::fmt_mask(mask, f)?;
    }
    Ok(())
  }
}
