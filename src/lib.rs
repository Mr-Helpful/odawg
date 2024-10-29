// pub mod rcdawg;
// pub use rcdawg::*;
mod pattern;
pub mod utils;
pub use utils::{serde_array, EndSort};
mod dawg;
pub use dawg::{AllDawg, FlatDawg, ReadDawg, WriteDawg};
mod node;
use node::{ReadNode, WriteNode};
pub use node::{ThinNode, WideNode, THIN_CHARS};
