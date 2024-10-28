use super::Pattern;
use crate::{FlatDawg, ThinNode, WideNode, THIN_CHARS};
use std::array;

impl From<Pattern> for FlatDawg<ThinNode> {
  fn from(value: Pattern) -> Self {
    let mut nodes = vec![];
    let mut repeats = 1;

    for mask in value.0 {
      let node = ThinNode {
        idx: nodes.len() + repeats,
        mask,
      };
      for _ in 0..repeats {
        nodes.push(node);
      }
      repeats = mask.count_ones() as usize;
    }

    let end_node = ThinNode {
      idx: 0,
      mask: 1 << THIN_CHARS,
    };
    for _ in 0..repeats {
      nodes.push(end_node);
    }

    Self(nodes)
  }
}

impl From<Pattern> for FlatDawg<WideNode<THIN_CHARS>> {
  fn from(value: Pattern) -> Self {
    let mut nodes = vec![];

    for mask in value.0 {
      let next = nodes.len() + 1;
      nodes.push(WideNode {
        end: false,
        children: array::from_fn(|i| (mask >> i & 1) as usize * next),
      });
    }

    nodes.push(WideNode {
      end: true,
      children: [0; THIN_CHARS],
    });
    Self(nodes)
  }
}

#[cfg(test)]
mod test {
  use super::{super::strategies::pattern_len, Pattern};
  use crate::{FlatDawg, ReadDawg, ThinNode, WideNode};
  use proptest::prelude::*;

  proptest! {
    #[test]
    fn any_pattern(pat in pattern_len(100_000)) {
      let thin_dawg: FlatDawg<ThinNode> = pat.clone().into();
      let wide_dawg: FlatDawg<WideNode> = pat.clone().into();

      for word in pat.words() {
        assert!(thin_dawg.has(&word));
        assert!(wide_dawg.has(&word));
      }

      for word in thin_dawg.words() {
        assert!(pat.has(&word));
      }
    }
  }

  #[test]
  fn empty() {
    let pat: Pattern = "".parse().unwrap();
    let thin_dawg: FlatDawg<ThinNode> = pat.clone().into();
    let wide_dawg: FlatDawg<WideNode> = pat.clone().into();

    assert!(!thin_dawg.has("jab"));
    assert!(!wide_dawg.has("jab"));
  }

  #[test]
  fn word() {
    let pat: Pattern = "jab".parse().unwrap();
    let thin_dawg: FlatDawg<ThinNode> = pat.clone().into();
    let wide_dawg: FlatDawg<WideNode> = pat.clone().into();

    for word in pat.words() {
      assert!(thin_dawg.has(&word));
      assert!(wide_dawg.has(&word));
    }
  }

  #[test]
  fn any() {
    let pat: Pattern = "-ab".parse().unwrap();
    let thin_dawg: FlatDawg<ThinNode> = pat.clone().into();
    let wide_dawg: FlatDawg<WideNode> = pat.clone().into();

    for word in pat.words() {
      assert!(thin_dawg.has(word.as_slice()));
      assert!(wide_dawg.has(word.as_slice()));
    }
  }

  #[test]
  fn group() {
    let pat: Pattern = "[fjt]ab".parse().unwrap();
    let thin_dawg: FlatDawg<ThinNode> = pat.clone().into();
    let wide_dawg: FlatDawg<WideNode> = pat.clone().into();

    for word in pat.words() {
      assert!(thin_dawg.has(&word[..]));
      assert!(wide_dawg.has(&word[..]));
    }
  }

  #[test]
  fn range() {
    let pat: Pattern = "[r-t]at".parse().unwrap();
    let thin_dawg: FlatDawg<ThinNode> = pat.clone().into();
    let wide_dawg: FlatDawg<WideNode> = pat.clone().into();

    for word in pat.words() {
      assert!(thin_dawg.has(&word[..]));
      assert!(wide_dawg.has(&word[..]));
    }
  }

  #[test]
  fn multi() {
    let pat: Pattern = "[b-cr-t]at".parse().unwrap();
    let thin_dawg: FlatDawg<ThinNode> = pat.clone().into();
    let wide_dawg: FlatDawg<WideNode> = pat.clone().into();

    for word in pat.words() {
      assert!(thin_dawg.has(&word[..]));
      assert!(wide_dawg.has(&word[..]));
    }
  }

  #[test]
  fn mixed() {
    let pat: Pattern = "[b-cpr-t]at".parse().unwrap();
    let thin_dawg: FlatDawg<ThinNode> = pat.clone().into();
    let wide_dawg: FlatDawg<WideNode> = pat.clone().into();

    for word in pat.words() {
      assert!(thin_dawg.has(&word[..]));
      assert!(wide_dawg.has(&word[..]));
    }
  }

  #[test]
  fn many() {
    let pat: Pattern = "[r-t][ai]t".parse().unwrap();
    let thin_dawg: FlatDawg<ThinNode> = pat.clone().into();
    let wide_dawg: FlatDawg<WideNode> = pat.clone().into();

    for word in pat.words() {
      assert!(thin_dawg.has(&word[..]));
      assert!(wide_dawg.has(&word[..]));
    }
  }

  #[test]
  fn start() {
    let pat: Pattern = "[-b]ye".parse().unwrap();
    let thin_dawg: FlatDawg<ThinNode> = pat.clone().into();
    let wide_dawg: FlatDawg<WideNode> = pat.clone().into();

    for word in pat.words() {
      assert!(thin_dawg.has(&word[..]));
      assert!(wide_dawg.has(&word[..]));
    }
  }

  #[test]
  fn end() {
    let pat: Pattern = "[y-]ap".parse().unwrap();
    let thin_dawg: FlatDawg<ThinNode> = pat.clone().into();
    let wide_dawg: FlatDawg<WideNode> = pat.clone().into();

    for word in pat.words() {
      assert!(thin_dawg.has(&word[..]));
      assert!(wide_dawg.has(&word[..]));
    }
  }

  #[test]
  fn all() {
    let pat: Pattern = "[-cmr-ty-]ap".parse().unwrap();
    let thin_dawg: FlatDawg<ThinNode> = pat.clone().into();
    let wide_dawg: FlatDawg<WideNode> = pat.clone().into();

    for word in pat.words() {
      assert!(thin_dawg.has(&word[..]));
      assert!(wide_dawg.has(&word[..]));
    }
  }
}
