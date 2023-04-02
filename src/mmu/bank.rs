use std::array::TryFromSliceError;
use std::ops::{Index, IndexMut};

use anyhow::{Result, bail};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeTuple;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub(super) struct Bank<const N: usize>([u8; N]);

impl<const N: usize> Bank<N> {
    pub(super) fn split_from(s: &[u8]) -> Result<Vec<Self>> {
        let mut banks = Vec::new();
        for chunk in s.chunks(N) {
            match chunk.try_into() {
                Ok(b) => banks.push(b),
                Err(_) => bail!("slice length doesn't divide bank size"),
            }
        }
        Ok(banks)
    }

    pub(super) fn get(&self, index: u16) -> Option<u8> {
        self.0.get(usize::from(index)).copied()
    }

    pub(super) fn get_mut(&mut self, index: u16) -> Option<&mut u8> {
        self.0.get_mut(usize::from(index))
    }
}

impl<const N: usize> Default for Bank<N> {
    fn default() -> Self {
        Self([0; N])
    }
}

impl<const N: usize> Index<u16> for Bank<N> {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        self.0.index(usize::from(index))
    }
}

impl<const N: usize> IndexMut<u16> for Bank<N> {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        self.0.index_mut(usize::from(index))
    }
}

impl<const N: usize> TryFrom<Vec<u8>> for Bank<N> {
    type Error = Vec<u8>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        value.try_into().map(Bank)
    }
}

impl<const N: usize> TryFrom<&[u8]> for Bank<N> {
    type Error = TryFromSliceError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        value.try_into().map(Bank)
    }
}

impl<const N: usize> Serialize for Bank<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_tuple(N)?;
        for byte in self.0 {
            seq.serialize_element(&byte)?;
        }
        seq.end()
    }
}

impl<'de, const N: usize> Deserialize<'de> for Bank<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(N, BankVisitor)
    }
}

struct BankVisitor<const N: usize>;

impl<'de, const N: usize> Visitor<'de> for BankVisitor<N> {
    type Value = Bank<N>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an array of length {N}")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        use serde::de::Error;

        let mut array = [0; N];
        for byte in array.iter_mut() {
            match seq.next_element()? {
                Some(v) => *byte = v,
                None => return Err(Error::invalid_length(N, &self)),
            }
        }

        Ok(Bank(array))
    }
}
