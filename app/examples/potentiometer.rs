#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_semihosting as _;

use cortex_m_rt::entry;
use stm32f1xx_hal::{adc, pac, prelude::*};

use cortex_m_semihosting::hprintln;

#[entry]
fn main() -> ! {
    // Aquire peripherals
    let p = pac::Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();

    // Configure ADC clocks
    // Default value is the slowest possible ADC clock: PCLK2 / 8. Meanwhile ADC
    // clock is configurable. So its frequency may be tweaked to meet certain
    // practical needs. User specified value is be approximated using supported
    // prescaler values 2/4/6/8.
    let clocks = rcc.cfgr.adcclk(2.mhz()).freeze(&mut flash.acr);
    hprintln!("adc freq: {}", clocks.adcclk().0).unwrap();

    // Setup ADC
    let mut adc1 = adc::Adc::adc1(p.ADC1, &mut rcc.apb2);

    // Setup GPIOB
    let mut gpiob = p.GPIOB.split(&mut rcc.apb2);

    // Configure pb0 as an analog input
    let mut ch0 = gpiob.pb0.into_analog(&mut gpiob.crl);

    loop {
        let data: u16 = adc1.read(&mut ch0).unwrap();
        hprintln!("adc1: {}", data).unwrap();
    }
}
