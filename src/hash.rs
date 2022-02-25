
//!

use std::hash::{BuildHasher, Hasher};

#[derive(Debug, Clone)]
pub(crate) struct U64Hash {
    hash: u64,
}

impl U64Hash {
    pub(crate) fn new() -> Self {
        U64Hash { hash: 1u64 }
    }
}

impl Hasher for U64Hash {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash = 31u64 * self.hash + *byte as u64;
        }
    }

    fn write_u64(&mut self, i: u64) {
        self.hash = i;
    }
}

impl BuildHasher for U64Hash {
    type Hasher = Self;

    fn build_hasher(&self) -> Self::Hasher {
        self.clone()
    }
}

impl Default for U64Hash {
    fn default() -> Self {
        U64Hash::new()
    }
}

#[cfg(test)]
mod hash_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_with_hash_set() {
        let mut set = HashSet::<u64, U64Hash>::with_hasher(U64Hash::default());
        set.insert(123);
        set.remove(&123);
    }
}
