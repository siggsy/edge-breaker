use std::fmt::Debug;
use std::ops::{Index, IndexMut};

// ,---------------------------------------------------------------------------
// | Op: history commands
// '---------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
pub enum Op {
    C,
    H,
    L,
    E,
    R,
    S,
}

pub const NULL: Id = Id(0);

// ,---------------------------------------------------------------------------
// | Id: dealing with offsets and vertex ids
// '---------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Id(usize);

impl Id {
    pub fn from_offset(off: usize) -> Id {
        Id(off + 1)
    }

    pub fn new(id: usize) -> Id {
        Id(id)
    }

    pub fn offset(&self) -> usize {
        self.0 - 1
    }

    pub fn id(&self) -> usize {
        self.0
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            0 => write!(f, "NULL"),
            id => write!(f, "#{}", id),
        }
    }
}

impl<T> Index<Id> for Vec<T> {
    type Output = T;

    fn index(&self, index: Id) -> &Self::Output {
        self.index(index.0 - 1)
    }
}

impl<T> IndexMut<Id> for Vec<T> {
    fn index_mut(&mut self, index: Id) -> &mut Self::Output {
        self.index_mut(index.0 - 1)
    }
}
