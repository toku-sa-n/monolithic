#![no_std]
#![no_main]

extern crate init as _;

#[no_mangle]
fn main() -> ! {
    loop {
        core::hint::spin_loop();
    }
}