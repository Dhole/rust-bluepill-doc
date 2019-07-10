#![deny(unsafe_code)]
#![no_main]
#![no_std]

use panic_semihosting as _;

use cortex_m_rt::entry;
use stm32f1xx_hal::{adc, pac, prelude::*};

#[entry]
fn main() -> ! {
    // Aquire peripherals
    let dp = pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Configure ADC clocks
    // Default value is the slowest possible ADC clock: PCLK2 / 8. Meanwhile ADC
    // clock is configurable. So its frequency may be tweaked to meet certain
    // practical needs. User specified value is be approximated using supported
    // prescaler values 2/4/6/8.
    let clocks = rcc.cfgr.adcclk(2.mhz()).freeze(&mut flash.acr);

    // Setup ADC
    let mut adc1 = adc::Adc::adc1(dp.ADC1, &mut rcc.apb2);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    // Setup GPIOB
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    // TIM2
    let c1 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);

    // Configure pb0 as an analog input
    let mut ch0 = gpiob.pb0.into_analog(&mut gpiob.crl);

    let mut pwm = dp
        .TIM2
        .pwm(c1, &mut afio.mapr, 1000.hz(), clocks, &mut rcc.apb1);
    let max = pwm.get_max_duty();
    pwm.enable();
    // let mut pa0 = gpioa.pa0.into_push_pull_output(&mut gpioa.crl);
    // pa0.set_high();

    loop {
        let data: u16 = adc1.read(&mut ch0).unwrap();
        let duty: f32 = (data as f32 - 2000.0) * max as f32 / 2000.0;
        pwm.set_duty(duty as u16);
    }
}
