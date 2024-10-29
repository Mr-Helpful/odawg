#![warn(
  clippy::correctness,
  clippy::suspicious,
  clippy::complexity,
  clippy::perf,
  clippy::style
)]
mod pattern;
pub mod utils;
pub use utils::{serde_array, EndSort};
mod dawg;
pub use dawg::{AllDawg, FlatDawg, ReadDawg, WriteDawg};
mod node;
use node::{ReadNode, WriteNode};
pub use node::{ThinNode, WideNode, THIN_CHARS};
