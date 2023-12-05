use std::ffi::c_char;

extern "C" {
    fn set_raw_term();
    fn set_cooked_term();
    fn get_ch() -> c_char;
}

fn main() {
    println!("Hello, world!");
    unsafe {
        set_raw_term();
    }
    unsafe {
        println!("{}", get_ch() as u8 as char);
    }
    unsafe {
        set_cooked_term();
    }
}
