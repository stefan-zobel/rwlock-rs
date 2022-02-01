
//!

use std::hash::{BuildHasher, Hasher};

#[derive(Debug, Clone)]
pub(crate) struct BytesHash {
    hash: u64,
}

impl BytesHash {
    pub(crate) fn new() -> Self {
        BytesHash { hash: 1u64 }
    }
}

impl Hasher for BytesHash {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash = 31u64 * self.hash + *byte as u64;
        }
    }
}

impl BuildHasher for BytesHash {
    type Hasher = Self;

    fn build_hasher(&self) -> Self::Hasher {
        self.clone()
    }
}

impl Default for BytesHash {
    fn default() -> Self {
        BytesHash::new()
    }
}

#[cfg(test)]
mod hash_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_with_hash_set() {
        let mut set = HashSet::<u64, BytesHash>::with_hasher(BytesHash::default());
        set.insert(123);
        set.remove(&123);
    }
}
