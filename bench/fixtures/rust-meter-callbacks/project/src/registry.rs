use std::ffi::c_void;

/// Callback shape shared with the device drivers: an opaque user pointer and one reading.
pub type Callback = extern "C" fn(user: *mut c_void, reading: i64);

struct Entry {
    id: u64,
    callback: Callback,
    user: *mut c_void,
}

/// Table of callbacks that are invoked for every reading dispatched.
#[derive(Default)]
pub struct Registry {
    entries: Vec<Entry>,
    next_id: u64,
}

impl Registry {
    pub fn add(&mut self, callback: Callback, user: *mut c_void) -> u64 {
        self.next_id += 1;
        self.entries.push(Entry { id: self.next_id, callback, user });
        self.next_id
    }

    /// Removes the entry and returns the user pointer it was registered with.
    pub fn remove(&mut self, id: u64) -> Option<*mut c_void> {
        let at = self.entries.iter().position(|e| e.id == id)?;
        Some(self.entries.swap_remove(at).user)
    }

    pub fn dispatch(&self, reading: i64) {
        for entry in &self.entries {
            (entry.callback)(entry.user, reading);
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern "C" fn bump(user: *mut c_void, reading: i64) {
        unsafe { *(user as *mut i64) += reading };
    }

    #[test]
    fn dispatch_reaches_every_entry() {
        let mut a = 0i64;
        let mut b = 0i64;
        let mut registry = Registry::default();
        registry.add(bump, &mut a as *mut i64 as *mut c_void);
        registry.add(bump, &mut b as *mut i64 as *mut c_void);
        registry.dispatch(5);
        assert_eq!((a, b), (5, 5));
    }

    #[test]
    fn remove_returns_the_registered_pointer() {
        let mut a = 0i64;
        let ptr = &mut a as *mut i64 as *mut c_void;
        let mut registry = Registry::default();
        let id = registry.add(bump, ptr);
        assert_eq!(registry.remove(id), Some(ptr));
        assert_eq!(registry.remove(id), None);
        assert_eq!(registry.len(), 0);
    }
}
