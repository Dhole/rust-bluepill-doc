//! Print "Hello world!" with "Hello rust!" underneath. Uses the `embedded_graphics` crate to draw
//! the text with a 6x8 pixel font.
//!
//! This example is for the STM32F103 "Blue Pill" board using I2C1.
//!
//! Wiring connections are as follows for a CRIUS-branded display:
//!
//! ```
//!      Display -> Blue Pill
//! (black)  GND -> GND
//! (red)    +5V -> VCC
//! (yellow) SDA -> PB9
//! (green)  SCL -> PB8
//! ```
//!
//! Run on a Blue Pill with `cargo run --example text_i2c`.#![no_main]

#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate embedded_graphics;
extern crate heapless;
extern crate panic_semihosting;
extern crate stm32f1xx_hal as hal;

use core::fmt::Write;
use cortex_m_rt::ExceptionFrame;
use cortex_m_rt::{entry, exception};
use embedded_graphics::fonts::Font12x16;
use embedded_graphics::prelude::*;
use hal::i2c::{BlockingI2c, DutyCycle, Mode};
use hal::prelude::*;
use hal::stm32;
use heapless::consts::*;
use heapless::String;
use ssd1306::prelude::*;
use ssd1306::Builder;

// About the main return type:
// https://www.reddit.com/r/rust/comments/3j22vx/what_is_the_meaning_of_as_a_return_type/
// main function never returns.

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 400_000,
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );

    let mut disp: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();

    disp.init().unwrap();
    disp.flush().unwrap();

    let button = gpiob.pb5.into_pull_down_input(&mut gpiob.crl);

    let repeat_delay = 16;
    let repeat_rate = 8;
    let mut elapsed: u32 = 0;

    let mut prev_state = false;
    let mut counter: u32 = 0;
    let mut s: String<U32> = String::new();
    write!(s, "{}", counter).unwrap();
    loop {
        let state = button.is_high();
        // value = match (prev_state, state) {
        //     (false, true) => "ON",
        //     (true, false) => "OFF",
        //     _ => value,
        // };
        // if (prev_state, state) == (false, true) {
        //     counter += 1;
        // }
        match (prev_state, state) {
            (false, true) => counter += 1,
            (true, true) => {
                elapsed += 1;
                if elapsed >= repeat_delay {
                    if (elapsed - repeat_delay) % repeat_rate == 0 {
                        counter += 1;
                    }
                }
            }
            _ => elapsed = 0,
        }
        s.clear();
        write!(s, "{}", counter).unwrap();
        let x = 128 - s.len() * 12;
        disp.clear();
        disp.draw(
            Font12x16::render_str(s.as_str())
                .with_stroke(Some(1u8.into()))
                .translate(Coord::new(x as i32, 28))
                .into_iter(),
        );
        disp.flush().unwrap();
        prev_state = state;
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
