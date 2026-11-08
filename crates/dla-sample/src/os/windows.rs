//! Windows backend: PSAPI, process handles and a Toolhelp32 process snapshot.
#![allow(unsafe_code)]

use std::mem::{size_of, zeroed};

use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_ACCESS_DENIED, FILETIME, GetLastError, HANDLE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::ProcessStatus::{
    GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
};
use windows_sys::Win32::System::Threading::{
    GetExitCodeProcess, GetProcessHandleCount, GetProcessTimes, OpenProcess,
    PROCESS_QUERY_LIMITED_INFORMATION,
};

use super::{OsSample, SampleError};

/// Exit code reported by a process that has not exited.
const STILL_ACTIVE: u32 = 259;
/// Seconds between 1601-01-01 (FILETIME epoch) and 1970-01-01.
const FILETIME_UNIX_OFFSET_SECS: u64 = 11_644_473_600;

struct Handle(HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: the handle was returned by a successful OpenProcess/CreateToolhelp32Snapshot.
        unsafe { CloseHandle(self.0) };
    }
}

enum Open {
    Process(Handle),
    Gone,
    Denied(u32),
}

fn open(pid: u32) -> Open {
    // SAFETY: plain call; a null handle signals failure.
    let h = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if h.is_null() {
        // SAFETY: reads the calling thread's last error.
        let err = unsafe { GetLastError() };
        // ERROR_ACCESS_DENIED means the process exists but cannot be opened; anything else
        // (typically ERROR_INVALID_PARAMETER) means there is no such process.
        if err == ERROR_ACCESS_DENIED {
            Open::Denied(err)
        } else {
            Open::Gone
        }
    } else {
        Open::Process(Handle(h))
    }
}

fn exited(h: &Handle) -> bool {
    let mut code = 0u32;
    // SAFETY: `code` is a valid out pointer.
    let ok = unsafe { GetExitCodeProcess(h.0, &mut code) };
    ok != 0 && code != STILL_ACTIVE
}

/// One entry of a process snapshot.
struct Entry {
    pid: u32,
    ppid: u32,
    threads: u32,
}

fn snapshot() -> Option<Vec<Entry>> {
    // SAFETY: plain call; INVALID_HANDLE_VALUE signals failure.
    let h = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if h == INVALID_HANDLE_VALUE || h.is_null() {
        return None;
    }
    let snap = Handle(h);
    // SAFETY: PROCESSENTRY32W is plain old data.
    let mut pe: PROCESSENTRY32W = unsafe { zeroed() };
    pe.dwSize = size_of::<PROCESSENTRY32W>() as u32;
    let mut out = Vec::new();
    // SAFETY: `pe` has dwSize set and the snapshot handle is valid for the calls.
    let mut more = unsafe { Process32FirstW(snap.0, &mut pe) } != 0;
    while more {
        out.push(Entry {
            pid: pe.th32ProcessID,
            ppid: pe.th32ParentProcessID,
            threads: pe.cntThreads,
        });
        // SAFETY: as above.
        more = unsafe { Process32NextW(snap.0, &mut pe) } != 0;
    }
    Some(out)
}

pub(super) fn sample(pid: u32) -> Result<OsSample, SampleError> {
    let mut out = OsSample::default();
    let handle = match open(pid) {
        Open::Gone => return Err(SampleError::NoSuchProcess(pid)),
        Open::Denied(err) => {
            out.limit(format!(
                "os.rss, os.private, os.pagefile, os.handles: OpenProcess denied (error {err})"
            ));
            None
        }
        Open::Process(h) => {
            if exited(&h) {
                return Err(SampleError::NoSuchProcess(pid));
            }
            Some(h)
        }
    };

    if let Some(h) = &handle {
        // SAFETY: PROCESS_MEMORY_COUNTERS_EX is plain old data; `cb` is set to its size and
        // the structure begins with the PROCESS_MEMORY_COUNTERS layout the API fills.
        let mut pmc: PROCESS_MEMORY_COUNTERS_EX = unsafe { zeroed() };
        pmc.cb = size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
        let ok = unsafe {
            GetProcessMemoryInfo(
                h.0,
                std::ptr::from_mut(&mut pmc).cast::<PROCESS_MEMORY_COUNTERS>(),
                pmc.cb,
            )
        };
        if ok != 0 {
            out.put("os.rss", pmc.WorkingSetSize as u64);
            out.put("os.vm_hwm", pmc.PeakWorkingSetSize as u64);
            out.put("os.private", pmc.PrivateUsage as u64);
            out.put("os.pagefile", pmc.PagefileUsage as u64);
        } else {
            out.limit("os.rss, os.private, os.pagefile: GetProcessMemoryInfo failed");
        }
        let mut count = 0u32;
        // SAFETY: `count` is a valid out pointer.
        if unsafe { GetProcessHandleCount(h.0, &mut count) } != 0 {
            out.put("os.handles", u64::from(count));
        } else {
            out.limit("os.handles: GetProcessHandleCount failed");
        }
    }

    match snapshot() {
        Some(entries) => match entries.iter().find(|e| e.pid == pid) {
            Some(e) => out.put("os.threads", u64::from(e.threads)),
            None => return Err(SampleError::NoSuchProcess(pid)),
        },
        None => out.limit("os.threads: Toolhelp32 snapshot failed"),
    }
    Ok(out)
}

pub(super) fn process_start_time(pid: u32) -> Option<String> {
    let Open::Process(h) = open(pid) else {
        return None;
    };
    // SAFETY: FILETIME is plain old data.
    let (mut created, mut ended, mut kernel, mut user): (FILETIME, FILETIME, FILETIME, FILETIME) =
        unsafe { (zeroed(), zeroed(), zeroed(), zeroed()) };
    // SAFETY: all four out pointers are valid.
    let ok = unsafe { GetProcessTimes(h.0, &mut created, &mut ended, &mut kernel, &mut user) };
    if ok == 0 {
        return None;
    }
    let ticks = (u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime);
    let secs = (ticks / 10_000_000).checked_sub(FILETIME_UNIX_OFFSET_SECS)?;
    Some(super::time::rfc3339_utc(secs))
}

pub(super) fn is_alive(pid: u32) -> bool {
    match open(pid) {
        Open::Process(h) => !exited(&h),
        // Exists but cannot be opened: fall back to the snapshot.
        Open::Denied(_) => snapshot().is_some_and(|s| s.iter().any(|e| e.pid == pid)),
        Open::Gone => false,
    }
}

pub(super) fn descendants(pid: u32) -> Vec<u32> {
    let Some(entries) = snapshot() else {
        return Vec::new();
    };
    let mut found: Vec<u32> = Vec::new();
    let mut queue = vec![pid];
    while let Some(parent) = queue.pop() {
        for e in entries.iter().filter(|e| e.ppid == parent) {
            // Parent ids are not cleared when a parent exits and may be reused: skip cycles.
            if e.pid != pid && e.pid != 0 && !found.contains(&e.pid) {
                found.push(e.pid);
                queue.push(e.pid);
            }
        }
    }
    found.sort_unstable();
    found
}
