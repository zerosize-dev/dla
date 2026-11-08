//! Fallback for operating systems without a sampling backend.

use super::{OsSample, SampleError};

pub(super) fn sample(_pid: u32) -> Result<OsSample, SampleError> {
    Err(SampleError::Unsupported)
}

pub(super) fn process_start_time(_pid: u32) -> Option<String> {
    None
}

pub(super) fn is_alive(_pid: u32) -> bool {
    false
}

pub(super) fn descendants(_pid: u32) -> Vec<u32> {
    Vec::new()
}
