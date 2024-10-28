use super::{ReadNode, WideNode};
use serde::{Deserialize, Serialize};

/// The number of character that thin Dawg nodes support.<br>
/// Can be up to 31 characters at most, due to implementation constraints.
pub const THIN_CHARS: usize = 26;
const CHILD_MASK: u32 = (1 << THIN_CHARS) - 1;
const END_MASK: u32 = 1 << THIN_CHARS;

/// Memory efficient Dawg nodes.<br>
/// Relies on the assumption that all children are contiguous.
// @note leaf nodes serialize to be empty
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThinNode {
  pub(crate) idx: usize,
  pub(crate) mask: u32,
}

impl ReadNode for ThinNode {
  type Idx = usize;

  fn is_empty(&self) -> bool {
    (self.mask & CHILD_MASK) == 0
  }

  fn is_end(&self) -> bool {
    (self.mask & END_MASK) > 0
  }

  fn has(&self, i: u8) -> bool {
    ((self.mask >> i) & 1) > 0
  }

  fn get(&self, i: u8) -> Self::Idx {
    if !self.has(i) {
      return 0;
    }

    // mask away all children "above" `i`
    let masked = self.mask & ((1 << i) - 1);
    self.idx + (masked.count_ones() as Self::Idx)
  }

  fn next_c(&self, c: u8) -> Option<u8> {
    let masked = self.mask & ((1 << c) - 1);
    (masked > 0).then(|| masked.count_zeros() as u8)
  }
}

#[derive(Clone, Debug)]
pub struct NonContiguous;

impl TryFrom<WideNode<26>> for ThinNode {
  type Error = NonContiguous;
  fn try_from(value: WideNode<26>) -> Result<Self, Self::Error> {
    let mut mask = if value.end { END_MASK } else { 0 };

    let mut iter = value.pairs();
    let Some((c, idx)) = iter.next() else {
      return Ok(Self { mask, idx: 0 });
    };
    mask |= 1 << c;

    let mut c_idx = idx;
    for (c, idx) in iter {
      if c_idx + 1 != idx {
        return Err(NonContiguous);
      }

      c_idx = idx;
      mask |= 1 << c;
    }

    Ok(Self { idx, mask })
  }
}
