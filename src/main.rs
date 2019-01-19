#![no_std]
#![no_main]

#![feature(alloc)]
#![feature(lang_items)]

// Plug in the allocator crate
extern crate alloc;
use core::alloc::Layout;
extern crate cortex_m_rt as rt; // v0.5.x

use alloc_cortex_m::CortexMHeap;
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();
// pick a panicking behavior
extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics
// extern crate panic_abort; // requires nightly
// extern crate panic_itm; // logs messages over ITM; requires ITM support
// extern crate panic_semihosting; // logs messages to the host stderr; requires a debugger
extern crate nrf52832_hal;

use cortex_m_rt::entry;
use nrf52832_hal::prelude::GpioExt;
use nrf52832_hal::prelude::SpimExt;
use nrf52832_hal::*;
use cortex_m::asm::delay;
use alloc::vec::Vec;
#[entry]
fn main() -> ! {

    let p = nrf52832_hal::nrf52832_pac::Peripherals::take().unwrap();
    let port0 = p.P0.split();

    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, 2048 as usize) }
    loop{
    }
}
// required: define how Out Of Memory (OOM) conditions should be handled
// *if* no other crate has already defined `oom`
#[lang = "oom"]
#[no_mangle]

pub fn rust_oom(_layout: Layout) -> ! {
   // trap here for the debuger to find
   loop {
   }
}
