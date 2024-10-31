//! [![Package](https://github.com/Mr-Helpful/odawg/actions/workflows/main.yml/badge.svg)](https://github.com/Mr-Helpful/odawg/actions/workflows/main.yml)
//!
//! # ODAWG data structure
//!
//! A common issue when playing scrabble is finding playable words in
//! a given row / column. I've attempted to solve this using the method below.
//!
//! ## Tries: eliminating common prefixes
//!
//! The first approach I came across online was the trie data structure[^1], where a list of words is stored in a trie like data structure.
//! This is acheived by identifying common prefixes of the word list. Here's a example (blatantly ripped from the DAWG article[^2]):
//!
//! __Word list:__
//!
//! - `cat`
//! - `cats`
//! - `fab`
//! - `fact`
//! - `facts`
//! - `face`
//! - `facet`
//! - `facets`
//!
//! __Trie Structure__:
//!
//! ```_
//! ─┬─ c ─── a ─── t.─── s.
//!  │
//!  └─ f ─── a ─┬─ c ─┬─ t. ─── s.
//!              │     │
//!              └─ b. └─ e.─── t.─── s.
//! ```
//!
//! A trie removes the storage requirement for the word prefixes and makes lookup faster as, instead of searching through the entire word list we can check 1 letter at a time, reducing the time to search from O(number of words) (or O(log(number of words)) for binary search) to O(length of search word).
//!
//! A trie can then be kept for each horizontal cell in a row, representing all the words moving left. When the letter `c` is placed 3 letters after from the cell, the trie can be trimmed down to only words with a `c` at depth 3, leaving `fact`, `facts`, `face`, `facet`, `facets` as so:
//!
//! __Trimmed Trie__:
//!
//! ```_
//! ══╗
//! ─┬╫ c ─── a ─── t.─── s.
//!  │╚══════════════════════════════════╗
//!  └─ f ─── a ─┬─ c ─┬─ t. ─── s.      ║
//! ═════════════╪════╗│                 ║
//!              └─ b.║└─ e.─── t.─── s. ║
//!                   ╚══════════════════╝
//! ```
//!
//! Once all restrictions have been applied and the resulting trie trimmed down, the words within the trie can be read using a depth first traversal, printing a word whenever an end of word marker (in these examples `.`) is encountered.
//!
//! [^1]: [Wikipedia article for tries](https://en.wikipedia.org/wiki/Trie)
//! [^2]: [Article on DAWG basics](https://jbp.dev/blog/dawg-basics.html)
//!
//! ## DAWGs: Eliminating common suffixes
//!
//! A DAWG stands for a Directed Acyclic Word Graph and further addresses the storage requirements for storing a trie by removing the need to store common suffixes of words as well as prefixes, so the storage space for the `ts` in both `cats` and `facets` can be shared, leading to similar space savings to those seen when using a trie over a standard word list.
//!
//! __DAWG Structure__:
//!
//! ```_
//! ─┬─ c ─── a ────┬─────┬── t.─── s.
//!  │              │     │
//!  └─ f ─── a ─┬─ c ─── e.
//!              │
//!              └─ b.
//! ```
#![warn(
  missing_docs,
  clippy::correctness,
  clippy::suspicious,
  clippy::complexity,
  clippy::perf,
  clippy::style
)]
mod pattern;
mod utils;
pub use utils::{from_word, into_word, serde_array, EndSort};
mod dawg;
pub use dawg::{AllDawg, FlatDawg, ReadDawg, WriteDawg};
mod node;
use node::{ReadNode, WriteNode};
pub use node::{ThinNode, WideNode, THIN_CHARS};
