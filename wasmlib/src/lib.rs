#[no_mangle]
pub extern "C" fn return_four() -> i32 {
    4
}

#[no_mangle]
pub extern "C" fn print_hello() {
    println!("Hello, world!");
}