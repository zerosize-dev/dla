//! macOS backend: libproc (`proc_pid_rusage`, `proc_pidinfo`, `proc_listchildpids`).
#![allow(unsafe_code)]

use std::mem::{size_of, zeroed};
use std::os::raw::{c_int, c_void};
use std::ptr;

use super::{OsSample, SampleError};

fn errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
}

/// Fills `T` from `proc_pidinfo`; `Err(errno)` when the call fails or returns a short read.
fn pidinfo<T>(pid: u32, flavor: c_int) -> Result<T, i32> {
    // SAFETY: the libproc structs are plain old data; all-zero is a valid value.
    let mut info: T = unsafe { zeroed() };
    let size = size_of::<T>() as c_int;
    // SAFETY: `info` is a writable buffer of exactly `size` bytes.
    let got = unsafe {
        libc::proc_pidinfo(
            pid as c_int,
            flavor,
            0,
            ptr::from_mut(&mut info).cast::<c_void>(),
            size,
        )
    };
    if got == size {
        Ok(info)
    } else if got <= 0 {
        Err(errno())
    } else {
        Err(libc::EIO)
    }
}

fn fd_count(pid: u32) -> Result<u64, i32> {
    // SAFETY: a null buffer with size 0 asks libproc for the required size.
    let needed =
        unsafe { libc::proc_pidinfo(pid as c_int, libc::PROC_PIDLISTFDS, 0, ptr::null_mut(), 0) };
    if needed <= 0 {
        return Err(errno());
    }
    let entry = size_of::<libc::proc_fdinfo>();
    // Room for descriptors opened between the two calls.
    let mut buf = vec![0u8; needed as usize + 16 * entry];
    // SAFETY: `buf` is writable for `buf.len()` bytes.
    let got = unsafe {
        libc::proc_pidinfo(
            pid as c_int,
            libc::PROC_PIDLISTFDS,
            0,
            buf.as_mut_ptr().cast::<c_void>(),
            buf.len() as c_int,
        )
    };
    if got <= 0 {
        return Err(errno());
    }
    Ok(got as u64 / entry as u64)
}

pub(super) fn sample(pid: u32) -> Result<OsSample, SampleError> {
    let mut out = OsSample::default();

    // SAFETY: rusage_info_v4 is plain old data; all-zero is a valid value.
    let mut ri: libc::rusage_info_v4 = unsafe { zeroed() };
    // SAFETY: the buffer is a rusage_info_v4 as RUSAGE_INFO_V4 requires.
    let rc = unsafe {
        libc::proc_pid_rusage(
            pid as c_int,
            libc::RUSAGE_INFO_V4,
            ptr::from_mut(&mut ri).cast::<libc::rusage_info_t>(),
        )
    };
    let rusage_errno = (rc != 0).then(errno);
    if rusage_errno == Some(libc::ESRCH) {
        return Err(SampleError::NoSuchProcess(pid));
    }
    if rusage_errno.is_none() {
        out.put("os.rss", ri.ri_resident_size);
        out.put("os.phys_footprint", ri.ri_phys_footprint);
    }

    match pidinfo::<libc::proc_taskinfo>(pid, libc::PROC_PIDTASKINFO) {
        Ok(ti) => {
            out.put("os.threads", ti.pti_threadnum.max(0) as u64);
            if rusage_errno.is_some() {
                out.put("os.rss", ti.pti_resident_size);
            }
        }
        Err(libc::ESRCH) => return Err(SampleError::NoSuchProcess(pid)),
        Err(e) => out.limit(format!(
            "os.threads: proc_pidinfo(PROC_PIDTASKINFO) failed (errno {e})"
        )),
    }
    if let Some(e) = rusage_errno {
        out.limit(format!(
            "os.phys_footprint: proc_pid_rusage failed (errno {e})"
        ));
    }

    match fd_count(pid) {
        Ok(n) => out.put("os.fds", n),
        Err(libc::ESRCH) => return Err(SampleError::NoSuchProcess(pid)),
        Err(e) => out.limit(format!(
            "os.fds: proc_pidinfo(PROC_PIDLISTFDS) failed (errno {e})"
        )),
    }
    Ok(out)
}

pub(super) fn process_start_time(pid: u32) -> Option<String> {
    let info = pidinfo::<libc::proc_bsdinfo>(pid, libc::PROC_PIDTBSDINFO).ok()?;
    Some(super::time::rfc3339_utc(info.pbi_start_tvsec))
}

pub(super) fn is_alive(pid: u32) -> bool {
    pidinfo::<libc::proc_bsdinfo>(pid, libc::PROC_PIDTBSDINFO)
        .is_ok_and(|info| info.pbi_status != libc::SZOMB)
}

fn children(pid: u32) -> Vec<u32> {
    const CAPACITY: usize = 4096;
    let mut buf = vec![0 as c_int; CAPACITY];
    // SAFETY: `buf` is writable for CAPACITY * 4 bytes.
    let got = unsafe {
        libc::proc_listchildpids(
            pid as c_int,
            buf.as_mut_ptr().cast::<c_void>(),
            (CAPACITY * size_of::<c_int>()) as c_int,
        )
    };
    if got <= 0 {
        return Vec::new();
    }
    // The call reports a count of pids; the zero filter and capacity clamp also keep this
    // correct should a libproc version report bytes.
    buf.into_iter()
        .take((got as usize).min(CAPACITY))
        .filter(|&p| p > 0)
        .map(|p| p as u32)
        .collect()
}

pub(super) fn descendants(pid: u32) -> Vec<u32> {
    let mut found: Vec<u32> = Vec::new();
    let mut queue = vec![pid];
    while let Some(parent) = queue.pop() {
        for child in children(parent) {
            if child != pid && !found.contains(&child) {
                found.push(child);
                queue.push(child);
            }
        }
    }
    found.sort_unstable();
    found
}
