use crate::node::ReadNode;
use crate::utils::convert::into_alpha;
use crate::utils::convert::ALPHA_CHARS;
use std::fmt::Display;
use std::fmt::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node<'a> {
    pub(crate) mask: &'a u32,
    pub(crate) next: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Letter<'a>(pub(crate) Option<Node<'a>>);

impl<'a> Letter<'a> {
    pub fn new(mask: &'a u32, next: usize) -> Self {
        Self(Some(Node { mask, next }))
    }
}

impl<'a> Letter<'a> {
    pub(crate) fn fmt_range(
        start: u8,
        end: u8,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
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
}

impl<'a> Display for Letter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(node) = self.0 else {
            return Ok(());
        };
        let mut mask = *node.mask;

        if mask.count_ones() == 0 {
            return f.write_str("[]");
        }
        if mask.count_ones() == 1 {
            return f.write_char(into_alpha(
                mask.trailing_zeros()
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

pub const CHILD_MASK: u32 = (1 << ALPHA_CHARS) - 1;

impl<'a> ReadNode for Letter<'a> {
    type Idx = usize;

    fn is_empty(&self) -> bool {
        !self.0.is_some_and(|node| (node.mask & CHILD_MASK) > 0)
    }
    fn is_end(&self) -> bool {
        self.0.is_none()
    }

    fn has(&self, i: u8) -> bool {
        self.0.is_some_and(|node| (node.mask >> i) & 1 > 0)
    }

    fn get(&self, i: u8) -> Self::Idx {
        self.0.map_or(
            0,
            |Node { mask, next }| if (mask >> i) & 1 > 0 { next } else { 0 },
        )
    }

    fn next_c(&self, c: u8) -> Option<u8> {
        let Node { mask, .. } = self.0?;
        let masked = mask & ((1 << c) - 1);
        (masked > 0).then(|| masked.count_zeros() as u8)
    }
}
