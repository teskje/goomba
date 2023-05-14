use std::ops::{Index, IndexMut};

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Memory(Box<[u8]>);

impl Memory {
    pub(super) fn with_size(n: usize) -> Self {
        vec![0; n].into()
    }

    pub(super) fn len(&self) -> usize {
        self.0.len()
    }

    pub(super) fn get<I>(&self, index: I) -> Option<u8>
    where
        I: Into<usize>,
    {
        self.0.get(index.into()).copied()
    }

    pub(super) fn get_mut<I>(&mut self, index: I) -> Option<&mut u8>
    where
        I: Into<usize>,
    {
        self.0.get_mut(index.into())
    }

    pub(super) fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for Memory {
    fn from(value: Vec<u8>) -> Self {
        Self(value.into_boxed_slice())
    }
}

impl<I: Into<usize>> Index<I> for Memory {
    type Output = u8;

    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index.into())
    }
}

impl<I: Into<usize>> IndexMut<I> for Memory {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index.into())
    }
}

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Banked<const N: u16>(Memory);

impl<const N: u16> Banked<N> {
    pub(super) fn banks(&self) -> usize {
        self.0.len() / usize::from(N)
    }

    pub(super) fn get(&self, bank: u8, offset: u16) -> Option<u8> {
        self.0.get(Self::idx(bank, offset))
    }

    pub(super) fn get_mut(&mut self, bank: u8, offset: u16) -> Option<&mut u8> {
        self.0.get_mut(Self::idx(bank, offset))
    }

    pub(super) fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    fn idx(bank: u8, offset: u16) -> usize {
        usize::from(bank) * usize::from(N) + usize::from(offset)
    }
}

impl<const N: u16> TryFrom<Memory> for Banked<N> {
    type Error = ();

    fn try_from(value: Memory) -> Result<Self, Self::Error> {
        if value.len() % usize::from(N) == 0 {
            Ok(Self(value))
        } else {
            Err(())
        }
    }
}

impl<const N: u16> Index<(u8, u16)> for Banked<N> {
    type Output = u8;

    fn index(&self, (bank, offset): (u8, u16)) -> &Self::Output {
        self.0.index(Self::idx(bank, offset))
    }
}

impl<const N: u16> IndexMut<(u8, u16)> for Banked<N> {
    fn index_mut(&mut self, (bank, offset): (u8, u16)) -> &mut Self::Output {
        self.0.index_mut(Self::idx(bank, offset))
    }
}
