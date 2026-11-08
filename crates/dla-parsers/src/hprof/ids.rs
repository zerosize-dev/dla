//! Object-id plumbing: a cheap hasher for `u64` keys and an open-addressing table from
//! object id to dense object index that stores only the indices.

use std::hash::{BuildHasherDefault, Hasher};

/// A multiplicative hasher for integer keys.
#[derive(Default)]
pub(crate) struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0.rotate_left(5) ^ u64::from(b)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        }
    }
    fn write_u64(&mut self, i: u64) {
        self.0 = (i ^ (i >> 29)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }
    fn write_u32(&mut self, i: u32) {
        self.write_u64(u64::from(i));
    }
    fn finish(&self) -> u64 {
        self.0 ^ (self.0 >> 32)
    }
}

/// `HashMap` with [`IdHasher`].
pub(crate) type IdMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<IdHasher>>;

/// No object.
pub(crate) const NONE: u32 = u32::MAX;

/// Object id to dense index; the ids themselves live in the caller's slice.
pub(crate) struct IdTable {
    slots: Vec<u32>,
    mask: usize,
    shift: u32,
}

#[inline]
fn mix(id: u64) -> u64 {
    (id >> 3 ^ id).wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

impl IdTable {
    pub fn build(ids: &[u64]) -> Self {
        let cap = (ids.len() * 4 / 3 + 8).next_power_of_two();
        let shift = 64 - cap.trailing_zeros();
        let mut t = Self {
            slots: vec![NONE; cap],
            mask: cap - 1,
            shift,
        };
        for (i, &id) in ids.iter().enumerate() {
            let mut h = (mix(id) >> t.shift) as usize;
            loop {
                if t.slots[h] == NONE {
                    t.slots[h] = i as u32;
                    break;
                }
                if ids[t.slots[h] as usize] == id {
                    break; // duplicate id: keep the first
                }
                h = (h + 1) & t.mask;
            }
        }
        t
    }

    /// The index of object `id`, or [`NONE`].
    #[inline]
    pub fn find(&self, ids: &[u64], id: u64) -> u32 {
        if id == 0 {
            return NONE;
        }
        let mut h = (mix(id) >> self.shift) as usize;
        loop {
            let s = self.slots[h];
            if s == NONE {
                return NONE;
            }
            if ids[s as usize] == id {
                return s;
            }
            h = (h + 1) & self.mask;
        }
    }
}
