//! # Serialize and Deserialize for arrays
//!
//! Currently, the `serde` package doesn't support constant size arrays,<br>
//! of the form `[T; N]` where `T` is serializable and deserializable.<br>
//! As we need to be able to serialize arrays to be able to serialize nodes.

use serde::{
    de::{Error, SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::marker::PhantomData;

/// A custom serializer for arrays of serializable values
pub fn serialize<const N: usize, T: Serialize, S: Serializer>(
    children: &[T; N],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut state = serializer.serialize_tuple(N)?;
    for idx in children {
        state.serialize_element(idx)?;
    }
    state.end()
}

/// A custom deserializer for arrays of deserializable values
pub fn deserialize<'de, const N: usize, T: Deserialize<'de>, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<[T; N], D::Error> {
    deserializer.deserialize_tuple(N, ArrayVisitor(PhantomData))
}

struct ArrayVisitor<const N: usize, T>(PhantomData<T>);

impl<'de, const N: usize, T: Deserialize<'de>> Visitor<'de> for ArrayVisitor<N, T> {
    type Value = [T; N];
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "[T; {N}]")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // `collect`ing a separate `Vec` consumes a bit more memory
        // but the alternative is `MaybeUinit` fuckery, so this'll do.
        std::iter::from_fn(|| seq.next_element().transpose())
            .collect::<Result<Vec<_>, _>>()?
            .try_into() // use array's TryFrom impl
            .map_err(|idxs: Vec<_>| Error::invalid_length(idxs.len(), &self))
    }
}
