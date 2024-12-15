#[test]
fn from_t1() {}

#[test]
fn miri_should_err() {
    let ptr = std::ptr::null_mut::<u8>();
    unsafe { *ptr = 1 };
}
