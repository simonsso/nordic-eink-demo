#![no_std]
#![no_main]

#![feature(alloc)]
#![feature(lang_items)]

// Plug in the allocator crate
extern crate alloc;
use nrf52832_hal::prelude::ClocksExt;
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
use nrf52832_hal::gpio::*;
use nrf52832_hal::gpio::Level;
use nrf52832_hal::gpio::p0::*;
use nrf52832_hal::prelude::GpioExt;
use nrf52832_hal::prelude::SpimExt;
use nrf52832_hal::Delay;
use embedded_hal::digital::{InputPin,OutputPin};
use nrf52832_hal::*;
use cortex_m::asm::delay;
use alloc::vec::Vec;

// the eink library
extern crate epd_waveshare;
use epd_waveshare::{
    epd1in54::{Buffer1in54, EPD1in54},
    graphics::{Display, DisplayRotation},
    prelude::*,
};


// Graphics
extern crate embedded_graphics;
use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::image::Image1BPP;
use embedded_graphics::prelude::*;
//use embedded_graphics::primitives::{Circle, Line};
use embedded_graphics::Drawing;

use core::option::*;

#[entry]
fn main() -> ! {

    let p = nrf52832_hal::nrf52832_pac::Peripherals::take().unwrap();
    let port0 = p.P0.split();

    let pxx = cortex_m::Peripherals::take().unwrap();

    let syst = pxx.SYST;
    let clocks = p.CLOCK.constrain().freeze();

    let mut delay = Delay::new(syst,clocks);

    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, 2048 as usize) }

    let mut led0: P0_20<gpio::Output<PushPull>>  = port0.p0_20.into_push_pull_output(Level::Low );
    let mut led1: P0_19<gpio::Output<PushPull>>  = port0.p0_19.into_push_pull_output(Level::Low );
    let mut led2: P0_18<gpio::Output<PushPull>>  = port0.p0_18.into_push_pull_output(Level::Low );
    let mut led3: P0_17<gpio::Output<PushPull>>  = port0.p0_17.into_push_pull_output(Level::Low );
 

    let spiclk:  P0_Pin<Output<PushPull>> = port0.p0_24.into_push_pull_output(Level::Low).degrade();
    let spimosi: P0_Pin<Output<PushPull>> = port0.p0_23.into_push_pull_output(Level::Low).degrade();


    let uartpins = nrf52832_hal::uarte::Pins{
            rts: Some(port0.p0_05.into_push_pull_output(Level::Low).degrade()),
            txd: port0.p0_06.into_push_pull_output(Level::Low).degrade(),
            cts: Some(port0.p0_07.into_push_pull_output(Level::Low).degrade()),
            rxd: port0.p0_08.into_push_pull_output(Level::Low).degrade(),
    };
    use nrf52832_hal::uarte::Baudrate::BAUD115200;
    use nrf52832_hal::uarte::Parity::EXCLUDED;
    let mut uart = nrf52832_hal::uarte::Uarte::new(
         p.UARTE0,
         uartpins,
         EXCLUDED,
         BAUD115200,
    );

    // uarte DMA cannot handle strings in flash.
    // use Vec to enforce data on heap
    let _ = uart.write(&(b"Hello Line1!\r\n").to_vec());
    let _ = uart.write(&(b"Hello Line3!\r\n").to_vec());
 
    let pins = nrf52832_hal::spim::Pins{sck:spiclk,miso:None,mosi:Some(spimosi)};
    let mut spi = p.SPIM0.constrain(pins);

    let btn1  = port0.p0_13.into_pullup_input();
    let btn2  = port0.p0_14.into_pullup_input();
    let btn3  = port0.p0_15.into_pullup_input();
   // let btn4  = port0.p0_16.into_pullup_input();

    // Pin     Connecton   Colour
    // P0.27   busy        purple
    // P0.26   Rst         white
    // P0.02   DC          Green
    // GND                 black
    // P0.25   CS          orange
    // P0.24   clk         yellow
    // P0.23   Din (MOSI)  blue  
    // Setup the epd    

    let cs = port0.p0_25.into_push_pull_output(Level::High );
    let rst = port0.p0_26.into_push_pull_output(Level::High );
    let busy = port0.p0_27.into_floating_input();
    let dc = port0.p0_02.into_push_pull_output(Level::High );

    let mut epd = EPD1in54::new(&mut spi, cs, busy, dc, rst, &mut delay).unwrap();

    // Setup the graphics
    let mut buffer = Buffer1in54::default();
    let mut display = Display::new(epd.width(), epd.height(), &mut buffer.buffer);

    // Draw some text
    // display.draw(
    //     Font6x8::render_str("Hello Rust vesropm!")
    //         .with_stroke(Some(Color::Black))
    //         .with_fill(Some(Color::White))
    //         .translate(Coord::new(5, 50))
    //         .into_iter()
    // );

    let rust_bytes = include_bytes!("../data/rust144x144.raw");
    let abema_bytes = include_bytes!("../data/abema151x151.raw");
    let philips_bytes = include_bytes!("../data/philps225x168.raw");

    let rust_img:Image1BPP<epd_waveshare::color::Color> = embedded_graphics::image::Image::new (rust_bytes,144,144);
    let abema_img:Image1BPP<epd_waveshare::color::Color> = embedded_graphics::image::Image::new(abema_bytes,151,151);
    let philips_img:Image1BPP<epd_waveshare::color::Color> = embedded_graphics::image::Image::new(abema_bytes,225,168);

    display.draw(rust_img.into_iter());
    // Transfer the frame data to the epd
    let _ans = epd.update_frame(&mut spi, &display.buffer());

    // Display the frame on the epd
    let _ans2 = epd.display_frame(&mut spi);


    let mut x=0;
    let mut y=0;
    loop{
        if btn1.is_low(){
            led0.set_low();
                display.draw(
                Font6x8::render_str("Hello, World!")
                    .with_stroke(Some(Color::Black))
                    .with_fill(Some(Color::White))
                    .translate(Coord::new(x, y))
                    .into_iter()
            );


            // Transfer the frame data to the epd
            let _ans = epd.update_frame(&mut spi, &display.buffer());

            // Display the frame on the epd
            let _ans2 = epd.display_frame(&mut spi);
            x += 0;
            y += 9;
        }else
        {
            led0.set_high();
        }
        if btn2.is_low(){
            led1.set_low();
            display.clear_buffer(Color::White);
            display.draw(rust_img.translate(Coord::new(28,28)).into_iter());
                        // Transfer the frame data to the epd
            let _ans = epd.update_frame(&mut spi, &display.buffer());

            // Display the frame on the epd
            let _ans2 = epd.display_frame(&mut spi);
        }else
        {
            led1.set_high();
        }
        if btn3.is_low(){
            led2.set_low();
            display.clear_buffer(Color::Black);
            display.draw(abema_img.translate(Coord::new(24,24)).into_iter());
                        // Transfer the frame data to the epd
            let _ans = epd.update_frame(&mut spi, &display.buffer());

            // Display the frame on the epd
            let _ans2 = epd.display_frame(&mut spi);
        }else
        {
            led2.set_high();
        }
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
