//! Frame codec called from the JVM through JNI. There is no jni crate: the
//! JNIEnv is an opaque pointer to a table of function pointers, and the two
//! entries used here are looked up by their slot numbers.
mod frame;

use frame::Frame;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, Ordering};

type JEnv = *mut c_void;
type JObj = *mut c_void;

/// Slot numbers in the JNINativeInterface_ table.
const SLOT_GET_ARRAY_LENGTH: usize = 171;
const SLOT_GET_BYTE_ARRAY_REGION: usize = 200;

static FRAMES: Mutex<Option<HashMap<i64, Frame>>> = Mutex::new(None);
static NEXT_HANDLE: AtomicI64 = AtomicI64::new(1);

/// JNIEnv is `const struct JNINativeInterface_ **`: read the table pointer,
/// then the function pointer stored at `idx`.
unsafe fn slot<T>(env: JEnv, idx: usize) -> T {
    let table = *(env as *const *const usize);
    std::mem::transmute_copy(&*table.add(idx))
}

fn with_frames<R>(f: impl FnOnce(&mut HashMap<i64, Frame>) -> R) -> R {
    let mut guard = FRAMES.lock().unwrap();
    f(guard.get_or_insert_with(HashMap::new))
}

#[no_mangle]
pub unsafe extern "system" fn Java_shop_codec_Native_open(env: JEnv, _cls: JObj, data: JObj) -> i64 {
    let length: unsafe extern "system" fn(JEnv, JObj) -> i32 = slot(env, SLOT_GET_ARRAY_LENGTH);
    let region: unsafe extern "system" fn(JEnv, JObj, i32, i32, *mut i8) =
        slot(env, SLOT_GET_BYTE_ARRAY_REGION);
    let n = length(env, data);
    let mut bytes = vec![0u8; n as usize];
    region(env, data, 0, n, bytes.as_mut_ptr() as *mut i8);
    let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
    with_frames(|m| m.insert(handle, Frame::new(bytes)));
    handle
}

#[no_mangle]
pub unsafe extern "system" fn Java_shop_codec_Native_checksum(_env: JEnv, _cls: JObj, handle: i64) -> i64 {
    with_frames(|m| m.get(&handle).map(|f| f.checksum as i64).unwrap_or(-1))
}

#[no_mangle]
pub unsafe extern "system" fn Java_shop_codec_Native_size(_env: JEnv, _cls: JObj, handle: i64) -> i32 {
    with_frames(|m| m.get(&handle).map(|f| f.bytes.len() as i32).unwrap_or(-1))
}

#[no_mangle]
pub unsafe extern "system" fn Java_shop_codec_Native_lines(_env: JEnv, _cls: JObj, handle: i64) -> i32 {
    with_frames(|m| {
        m.get(&handle)
            .and_then(|f| f.line_count())
            .map(|n| n as i32)
            .unwrap_or(-1)
    })
}

#[no_mangle]
pub unsafe extern "system" fn Java_shop_codec_Native_close(_env: JEnv, _cls: JObj, handle: i64) {
    with_frames(|m| m.remove(&handle));
}

#[no_mangle]
pub unsafe extern "system" fn Java_shop_codec_Native_live(_env: JEnv, _cls: JObj) -> i32 {
    with_frames(|m| m.len() as i32)
}

#[no_mangle]
pub unsafe extern "system" fn Java_shop_codec_Native_liveBytes(_env: JEnv, _cls: JObj) -> i64 {
    with_frames(|m| m.values().map(|f| f.bytes.capacity() as i64).sum())
}
