//! Paged storage addressed by generation-checked handles.

const PAGE: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Handle {
    index: u32,
    generation: u32,
}

struct Slot<T> {
    generation: u32,
    label: [u8; 32],
    value: Option<Box<T>>,
}

impl<T> Slot<T> {
    fn empty() -> Self {
        Slot { generation: 0, label: [0; 32], value: None }
    }
}

pub struct Arena<T> {
    pages: Vec<Box<[Slot<T>]>>,
    free: Vec<u32>,
    next: u32,
    live: usize,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena { pages: Vec::new(), free: Vec::new(), next: 0, live: 0 }
    }

    fn slot_mut(&mut self, index: u32) -> &mut Slot<T> {
        let index = index as usize;
        &mut self.pages[index / PAGE][index % PAGE]
    }

    fn claim(&mut self) -> u32 {
        if let Some(index) = self.free.pop() {
            return index;
        }
        let index = self.next;
        if index as usize == self.pages.len() * PAGE {
            self.pages.push((0..PAGE).map(|_| Slot::empty()).collect::<Vec<_>>().into_boxed_slice());
        }
        self.next += 1;
        index
    }

    /// Stores `value` under a short owner label and returns its handle.
    pub fn insert(&mut self, label: &str, value: T) -> Handle {
        let index = self.claim();
        let slot = self.slot_mut(index);
        slot.generation += 1;
        let bytes = label.as_bytes();
        let n = bytes.len().min(slot.label.len());
        slot.label = [0; 32];
        slot.label[..n].copy_from_slice(&bytes[..n]);
        slot.value = Some(Box::new(value));
        let handle = Handle { index, generation: slot.generation };
        self.live += 1;
        handle
    }

    pub fn get(&self, handle: Handle) -> Option<&T> {
        let index = handle.index as usize;
        let slot = self.pages.get(index / PAGE)?.get(index % PAGE)?;
        if slot.generation != handle.generation {
            return None;
        }
        slot.value.as_deref()
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        let slot = self.slot_mut(handle.index);
        if slot.generation != handle.generation {
            return None;
        }
        slot.value.as_deref_mut()
    }

    pub fn remove(&mut self, handle: Handle) -> Option<T> {
        let slot = self.slot_mut(handle.index);
        if slot.generation != handle.generation {
            return None;
        }
        let value = slot.value.take()?;
        slot.generation += 1;
        self.live -= 1;
        Some(*value)
    }

    pub fn len(&self) -> usize {
        self.live
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserted_values_are_found() {
        let mut arena = Arena::new();
        let a = arena.insert("a", 10u32);
        let b = arena.insert("b", 20u32);
        assert_eq!(arena.get(a), Some(&10));
        assert_eq!(arena.get(b), Some(&20));
        assert_eq!(arena.len(), 2);
    }

    #[test]
    fn a_removed_handle_stops_resolving() {
        let mut arena = Arena::new();
        let a = arena.insert("a", 10u32);
        assert_eq!(arena.remove(a), Some(10));
        assert_eq!(arena.get(a), None);
        assert_eq!(arena.remove(a), None);
        assert_eq!(arena.len(), 0);
    }

    #[test]
    fn values_survive_other_removals() {
        let mut arena = Arena::new();
        let handles: Vec<Handle> = (0..600u32).map(|n| arena.insert("n", n)).collect();
        for h in handles.iter().step_by(2) {
            arena.remove(*h);
        }
        for (n, h) in handles.iter().enumerate() {
            assert_eq!(arena.get(*h).copied(), if n % 2 == 0 { None } else { Some(n as u32) });
        }
        assert_eq!(arena.len(), 300);
    }

    #[test]
    fn get_mut_changes_the_stored_value() {
        let mut arena = Arena::new();
        let a = arena.insert("a", 1u32);
        *arena.get_mut(a).unwrap() += 41;
        assert_eq!(arena.get(a), Some(&42));
    }
}
