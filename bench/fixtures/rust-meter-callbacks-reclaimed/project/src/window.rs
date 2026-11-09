/// A metering device known to the gateway.
pub struct Device {
    pub name: String,
}

impl Device {
    /// Starts a session's rolling statistics for this device.
    pub fn open(&self) -> Window {
        Window::new(self.name.clone())
    }
}

/// Rolling statistics for the readings of one device session.
pub struct Window {
    pub device: String,
    samples: Vec<i64>,
    sum: i64,
}

impl Window {
    fn new(device: String) -> Self {
        Window { device, samples: Vec::with_capacity(512), sum: 0 }
    }

    pub fn push(&mut self, reading: i64) {
        if self.samples.len() == 512 {
            let oldest = self.samples.remove(0);
            self.sum -= oldest;
        }
        self.samples.push(reading);
        self.sum += reading;
    }

    pub fn mean(&self) -> Option<i64> {
        if self.samples.is_empty() {
            None
        } else {
            Some(self.sum / self.samples.len() as i64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_of_samples() {
        let mut w = Device { name: "m-1".into() }.open();
        assert_eq!(w.mean(), None);
        w.push(10);
        w.push(20);
        assert_eq!(w.mean(), Some(15));
    }

    #[test]
    fn window_is_capped() {
        let mut w = Device { name: "m-2".into() }.open();
        for i in 0..600 {
            w.push(i);
        }
        assert_eq!(w.samples.len(), 512);
    }
}
