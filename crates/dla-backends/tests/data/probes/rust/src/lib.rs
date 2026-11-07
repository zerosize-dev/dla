pub fn a() -> &'static mut Vec<u8> {
    Box::leak(Box::new(Vec::new()))
}
pub fn b() {
    let v = Box::new(5u32);
    let _p = Box::into_raw(v);
}
