use std::ffi::c_void;

use crate::registry::Registry;
use crate::window::{Device, Window};

extern "C" fn on_reading(user: *mut c_void, reading: i64) {
    let window = unsafe { &mut *(user as *mut Window) };
    window.push(reading);
}

/// One connected device. Its readings are folded into a `Window` by the driver callback.
pub struct Session {
    id: u64,
    user: *mut c_void,
}

impl Session {
    pub fn open(registry: &mut Registry, device: Device) -> Session {
        let user = Box::into_raw(Box::new(device.open())) as *mut c_void;
        let id = registry.add(on_reading, user);
        Session { id, user }
    }

    pub fn mean(&self) -> Option<i64> {
        unsafe { (*(self.user as *mut Window)).mean() }
    }

    pub fn device(&self) -> &str {
        unsafe { &(*(self.user as *mut Window)).device }
    }

    pub fn close(self, registry: &mut Registry) {
        if let Some(user) = registry.remove(self.id) {
            drop(unsafe { Box::from_raw(user as *mut Window) });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_session_sees_readings() {
        let mut registry = Registry::default();
        let session = Session::open(&mut registry, Device { name: "m-3".into() });
        registry.dispatch(10);
        registry.dispatch(30);
        assert_eq!(session.mean(), Some(20));
        session.close(&mut registry);
        assert_eq!(registry.len(), 0);
    }
}
