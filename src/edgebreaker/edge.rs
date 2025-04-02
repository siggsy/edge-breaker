use std::hash::Hash;

// ,---------------------------------------------------------------------------
// | Definition
// `---------------------------------------------------------------------------

#[derive(Eq)]
pub struct Edge {
    a: usize,
    b: usize,
}

// ,---------------------------------------------------------------------------
// | Implementations
// `---------------------------------------------------------------------------

impl Edge {
    pub fn new(a: usize, b: usize) -> Edge {
        Edge { a, b }
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a && self.b == other.b || self.a == other.b && self.b == other.a
    }
}

impl Hash for Edge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if self.a < self.b {
            self.a.hash(state);
            self.b.hash(state);
        } else {
            self.b.hash(state);
            self.a.hash(state);
        }
    }
}
