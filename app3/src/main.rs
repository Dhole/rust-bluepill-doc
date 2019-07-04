//! Serial interface loopback test
//!
//! You have to short the TX and RX pins to make this program work

// #![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_halt;
extern crate ssd1306;

// use cortex_m::asm;

use nb::block;

use core::fmt::Write;
use cortex_m_rt::entry;
// use embedded_graphics::fonts::Font6x8;
// use embedded_graphics::prelude::*;
use ssd1306::prelude::*;
use ssd1306::Builder;
use stm32f1xx_hal::delay::Delay;
use stm32f1xx_hal::i2c::{BlockingI2c, DutyCycle, Mode};
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::stm32;
use stm32f1xx_hal::{pac, prelude::*, serial::Serial};

// https://stackoverflow.com/a/50201632
pub mod write_to {
    use core::cmp::min;
    use core::fmt;

    pub struct WriteTo<'a> {
        buffer: &'a mut [u8],
        // on write error (i.e. not enough space in buffer) this grows beyond
        // `buffer.len()`.
        used: usize,
    }

    impl<'a> WriteTo<'a> {
        pub fn new(buffer: &'a mut [u8]) -> Self {
            WriteTo { buffer, used: 0 }
        }

        pub fn as_str(self) -> Option<&'a str> {
            if self.used <= self.buffer.len() {
                // only successful concats of str - must be a valid str.
                use core::str::from_utf8_unchecked;
                Some(unsafe { from_utf8_unchecked(&self.buffer[..self.used]) })
            } else {
                None
            }
        }
    }

    impl<'a> fmt::Write for WriteTo<'a> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            if self.used > self.buffer.len() {
                return Err(fmt::Error);
            }
            let remaining_buf = &mut self.buffer[self.used..];
            let raw_s = s.as_bytes();
            let write_num = min(raw_s.len(), remaining_buf.len());
            remaining_buf[..write_num].copy_from_slice(&raw_s[..write_num]);
            self.used += raw_s.len();
            if write_num < raw_s.len() {
                Err(fmt::Error)
            } else {
                Ok(())
            }
        }
    }

    pub fn show<'a>(buffer: &'a mut [u8], args: fmt::Arguments) -> Result<&'a str, fmt::Error> {
        let mut w = WriteTo::new(buffer);
        fmt::write(&mut w, args)?;
        w.as_str().ok_or(fmt::Error)
    }
}

#[entry]
fn main() -> ! {
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .freeze(&mut flash.acr);

    // Prepare the alternate function I/O registers
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    // Prepare the GPIOB peripheral
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    let i2c = BlockingI2c::i2c1(
        dp.I2C1,
        (scl, sda),
        &mut afio.mapr,
        Mode::Fast {
            frequency: 800_000,
            duty_cycle: DutyCycle::Ratio2to1,
        },
        clocks,
        &mut rcc.apb1,
        1000,
        10,
        1000,
        1000,
    );
    let mut disp: TerminalMode<_> = Builder::new().connect_i2c(i2c).into();
    disp.init().unwrap();
    let _ = disp.clear();

    // USART1
    // let tx1 = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    // let rx1 = gpioa.pa10;

    // USART1
    // let tx = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
    // let rx = gpiob.pb7;

    // USART2
    let tx2 = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let rx2 = gpioa.pa3;

    // USART3
    // Configure pb10 as a push_pull output, this will be the tx pin
    // let tx = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
    // Take ownership over pb11
    // let rx = gpiob.pb11;

    // Set up the usart device. Taks ownership over the USART register and tx/rx pins. The rest of
    // the registers are used to enable and configure the device.
    // let serial1 = Serial::usart1(
    //     dp.USART1,
    //     (tx1, rx1),
    //     &mut afio.mapr,
    //     9_600.bps(),
    //     clocks,
    //     &mut rcc.apb2,
    // );

    let serial2 = Serial::usart2(
        dp.USART2,
        (tx2, rx2),
        &mut afio.mapr,
        9_600.bps(),
        clocks,
        &mut rcc.apb1,
    );

    // Split the serial struct into a receiving and a transmitting part
    // let (mut tx1, mut rx1) = serial1.split();
    let (mut tx2, mut rx2) = serial2.split();

    let init_msg = "Init          \r\n";
    for c in init_msg.bytes() {
        let _ = disp.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) });
    }

    /// `while true; do date +'Is anyone there?%Y-%m-%d      %T' > /dev/ttyUSB0; sleep 1; done`
    // let msg = "Hello!\r\n";
    // let msg = format!("Hello {}!\r\n", "World");
    // let mut buf = [0u8; 64];
    // let mut delay = Delay::new(cp.SYST, clocks);
    // let _ = disp.clear();
    let mut buf = [0u8; 16 * 8];
    let mut len = 0;
    // let mut clear = false;
    loop {
        buf[len] = block!(rx2.read()).unwrap();
        len += 1;
        //if clear {
        //    // disp.init().unwrap();
        //    let _ = disp.clear();
        //    clear = false;
        //}
        // if c == b'\n' {
        //     clear = true;
        // }
        if buf[len - 1] == b'\n' {
            let _ = disp.clear();
            let _ = disp.write_str(unsafe { core::str::from_utf8_unchecked(&buf[..len]) });
            len = 0;
        }
    }
    //for i in 0 as u32..0xffffffff {
    //    let msg = write_to::show(&mut buf, format_args!("Hello {}!\r\n", i)).unwrap();
    //    for b in msg.bytes() {
    //        // block!(tx1.write(b)).ok();
    //        block!(tx2.write(b)).ok();
    //    }
    //    delay.delay_ms(1000 as u32);
    //}
    // Read the byte that was just sent. Blocks until the read is complete
    // let received = block!(rx.read()).unwrap();

    loop {}
}
