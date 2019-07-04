//! Testing PWM output

//#![deny(unsafe_code)]
//#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_halt;

use cast::{u16, u32};
// use cortex_m::asm;
use cortex_m_rt::entry;
use stm32f1xx_hal::delay::Delay;
use stm32f1xx_hal::rcc::Clocks;
use stm32f1xx_hal::stm32;
use stm32f1xx_hal::time::Hertz;
use stm32f1xx_hal::{pac, pac::TIM2, prelude::*};

const C0: f32 = 16.35;
const CS0: f32 = 17.32;
const D0: f32 = 18.35;
const DS0: f32 = 19.45;
const E0: f32 = 20.60;
const F0: f32 = 21.83;
const FS0: f32 = 23.12;
const G0: f32 = 24.50;
const GS0: f32 = 25.96;
const A0: f32 = 27.50;
const AS0: f32 = 29.14;
const B0: f32 = 30.87;

#[derive(PartialEq)]
enum Note {
    C,
    CS,
    D,
    DS,
    E,
    F,
    FS,
    G,
    GS,
    A,
    AS,
    B,
    S,
}

impl Note {
    fn freq0(&self) -> f32 {
        match self {
            Note::C => C0,
            Note::CS => CS0,
            Note::D => D0,
            Note::DS => DS0,
            Note::E => E0,
            Note::F => F0,
            Note::FS => FS0,
            Note::G => G0,
            Note::GS => GS0,
            Note::A => A0,
            Note::AS => AS0,
            Note::B => B0,
            Note::S => C0,
        }
    }
}

struct Tone {
    note: Note,
    octave: u32,
    duration: u16,
    freq: Hertz,
}

impl Tone {
    fn new(note: Note, octave: u32, duration: u16) -> Self {
        let freq = ((note.freq0() * ((2 as u32).pow(octave) as f32)) as u32).hz();
        Tone {
            note,
            octave,
            duration,
            freq,
        }
    }
}

fn set_pwn_freq(clocks: Clocks, freq: Hertz) {
    unsafe {
        let clk = clocks.pclk1_tim().0;
        let freq = freq.0;
        let ticks = clk / freq;
        let psc = u16(ticks / (1 << 16)).unwrap();
        (*TIM2::ptr()).psc.write(|w| unsafe { w.psc().bits(psc) });
        let arr = u16(ticks / u32(psc + 1)).unwrap();
        (*TIM2::ptr()).arr.write(|w| w.arr().bits(arr));

        (*TIM2::ptr()).cr1.write(|w| unsafe {
            w.cms()
                .bits(0b00)
                .dir()
                .clear_bit()
                .opm()
                .clear_bit()
                .cen()
                .set_bit()
        });
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(36.mhz())
        .freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    // let mut gpiob = p.GPIOB.split(&mut rcc.apb2);

    // TIM2
    let c1 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);

    let tones_intro = [
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::E, 5, 1),
        Tone::new(Note::FS, 5, 2),
        Tone::new(Note::B, 5, 2),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::E, 5, 1),
        Tone::new(Note::FS, 5, 1),
        Tone::new(Note::B, 5, 1),
        Tone::new(Note::CS, 6, 1),
        Tone::new(Note::DS, 6, 1),
        Tone::new(Note::CS, 6, 1),
        Tone::new(Note::AS, 5, 1),
        Tone::new(Note::B, 5, 2),
        Tone::new(Note::FS, 5, 2),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::E, 5, 1),
        Tone::new(Note::FS, 5, 2),
        Tone::new(Note::B, 5, 2),
        Tone::new(Note::CS, 6, 1),
        Tone::new(Note::AS, 5, 1),
        Tone::new(Note::B, 5, 1),
        Tone::new(Note::CS, 6, 1),
        Tone::new(Note::E, 6, 1),
        Tone::new(Note::DS, 6, 1),
        Tone::new(Note::E, 6, 1),
        Tone::new(Note::B, 5, 1),
    ];

    let tones_melody = [
        Tone::new(Note::FS, 5, 2),
        Tone::new(Note::GS, 5, 2),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::S, 0, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::D, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::S, 0, 1),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::CS, 5, 2),
        Tone::new(Note::D, 5, 2),
        Tone::new(Note::D, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::FS, 5, 1),
        Tone::new(Note::GS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::FS, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::DS, 5, 2),
        Tone::new(Note::FS, 5, 2),
        Tone::new(Note::GS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::FS, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::D, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::D, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::D, 5, 2),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::FS, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::CS, 5, 2),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::CS, 5, 2),
        Tone::new(Note::FS, 5, 2),
        Tone::new(Note::GS, 5, 2),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::S, 0, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::D, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::S, 0, 1),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::CS, 5, 2),
        Tone::new(Note::D, 5, 2),
        Tone::new(Note::D, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::FS, 5, 1),
        Tone::new(Note::GS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::FS, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::DS, 5, 2),
        Tone::new(Note::FS, 5, 2),
        Tone::new(Note::GS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::FS, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::D, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::D, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::D, 5, 2),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::FS, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::CS, 5, 2),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::CS, 5, 2),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::GS, 4, 1),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::GS, 4, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::E, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::E, 5, 1),
        Tone::new(Note::FS, 5, 1),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::GS, 4, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::E, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::DS, 4, 1),
        Tone::new(Note::E, 4, 1),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::GS, 4, 1),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::GS, 4, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::GS, 4, 1),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::AS, 4, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::GS, 4, 1),
        Tone::new(Note::E, 4, 1),
        Tone::new(Note::E, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::E, 5, 1),
        Tone::new(Note::FS, 5, 1),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::AS, 4, 2),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::GS, 4, 1),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::GS, 4, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::E, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::E, 5, 1),
        Tone::new(Note::FS, 5, 1),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::GS, 4, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::E, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::DS, 4, 1),
        Tone::new(Note::E, 4, 1),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::GS, 4, 1),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::GS, 4, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::CS, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::GS, 4, 1),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::AS, 4, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::FS, 4, 1),
        Tone::new(Note::GS, 4, 1),
        Tone::new(Note::B, 4, 1),
        Tone::new(Note::E, 5, 1),
        Tone::new(Note::DS, 5, 1),
        Tone::new(Note::E, 5, 1),
        Tone::new(Note::FS, 5, 1),
        Tone::new(Note::B, 4, 2),
        Tone::new(Note::CS, 5, 2),
    ];

    let mut pwm = dp
        .TIM2
        .pwm(c1, &mut afio.mapr, 440.hz(), clocks, &mut rcc.apb1);
    let max = pwm.get_max_duty();
    pwm.set_duty(max / 2);

    let mut delay = Delay::new(cp.SYST, clocks);
    let dur = 100;
    let pause = 10;
    loop {
        for tone in tones_intro.iter() {
            if tone.note != Note::S {
                set_pwn_freq(clocks, tone.freq);
                pwm.enable();
            }
            delay.delay_ms(tone.duration * dur);
            pwm.disable();
            delay.delay_ms(pause as u32);
        }

        for _ in 0..3 {
            for tone in tones_melody.iter() {
                if tone.note != Note::S {
                    set_pwn_freq(clocks, tone.freq);
                    pwm.enable();
                }
                delay.delay_ms(tone.duration * dur);
                pwm.disable();
                delay.delay_ms(pause as u32);
            }
        }
    }

    loop {}
}
