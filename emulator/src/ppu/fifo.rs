use std::collections::VecDeque;

const LENGTH: usize = 8;

#[derive(Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Fifo<T> {
    data: VecDeque<T>,
    discard: u8,
    locked: bool,
}

impl<T> Fifo<T> {
    pub(super) fn with_discard(discard: u8) -> Self {
        Self {
            discard,
            ..Default::default()
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub(super) fn is_full(&self) -> bool {
        self.data.len() == LENGTH
    }

    pub(super) fn pop(&mut self) -> Option<T> {
        if self.locked {
            return None;
        }

        let item = self.data.pop_front()?;
        if self.discard > 0 {
            self.discard -= 1;
            None
        } else {
            Some(item)
        }
    }

    pub(super) fn push(&mut self, item: T) {
        self.data.push_back(item);
    }

    pub(super) fn put_back(&mut self, item: T) {
        self.data.push_front(item);
    }

    pub(super) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut()
    }

    pub(super) fn lock(&mut self) {
        self.locked = true;
    }

    pub(super) fn unlock(&mut self) {
        self.locked = false;
    }
}

impl<T: Default> Fifo<T> {
    pub(super) fn fill(&mut self) {
        while !self.is_full() {
            self.push(Default::default());
        }
    }
}

impl<T> Default for Fifo<T> {
    fn default() -> Self {
        Self {
            data: VecDeque::with_capacity(LENGTH),
            discard: 0,
            locked: false,
        }
    }
}

impl<T> Extend<T> for Fifo<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.data.extend(iter);
    }
}
