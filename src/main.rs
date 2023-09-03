#![no_std]
#![no_main]

extern crate alloc;

mod dos_tests;
mod interrupts;

use crate::dos_tests::{
    allocator_test::allocator_test, cooperative_multitasking_test::cooperative_multitasking_test,
    file::file_read_test,
};
use interrupts::display_string;
use rust_dos::*;
//use crate::dos_tests::allocator_test::allocator_test;
//use crate::dos_tests::file::file_read_test;

entry!(main);

fn main() {
    //allocator_test();
    //file_read_test();
    //cooperative_multitasking_test();

    display_string("Hello, world!$");
}
