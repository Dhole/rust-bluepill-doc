Rust on the blue pill (stm32f1xx)

# Hardware details

![](Bluepillpinout.gif)

https://wiki.stm32duino.com/index.php?title=Blue_Pill

- **Microcontroller**:         STM32F103C8
- **Flash**:   64 KB/128 KB
- **RAM**:     20 KB
- **Clock Speed**:     72 MHz
- **User LED(s)**:     PC13 (blue; lights when PC13 is LOW)

## STM32F103C8

https://www.st.com/en/microcontrollers-microprocessors/stm32f103c8.html

> The STM32F103xx medium-density performance line family incorporates the
> high-performance ARM®Cortex®-M3 32-bit RISC core operating at a 72 MHz
> frequency, high-speed embedded memories (Flash memory up to 128 Kbytes and SRAM
> up to 20 Kbytes), and an extensive range of enhanced I/Os and peripherals
> connected to two APB buses. All devices offer two 12-bit ADCs, three general
> purpose 16-bit timers plus one PWM timer, as well as standard and advanced
> communication interfaces: up to two I2Cs and SPIs, three USARTs, an USB and a
> CAN. 

# Install dependencies

## Rust

https://rustup.rs/
```
curl https://sh.rustup.rs -sSf | sh
```

### Cortex M3

```
rustup target add thumbv7m-none-eabi
```

Note: What is Thumb? See
[here](https://en.wikipedia.org/wiki/ARM_architecture#Thumb) and
[here](https://www.embedded.com/electronics-blogs/beginner-s-corner/4024632/Introduction-to-ARM-thumb)?

```
cargo install cargo-binutils
cargo install cargo-generate
rustup component add llvm-tools-preview
```

### Linux 

https://rust-embedded.github.io/book/intro/install/linux.html

Debian / Ubuntu
```
sudo apt install \
  gdb-arm-none-eabi \
  binutils-arm-none-eabi \
  openocd \
  qemu-system-arm
```

Install stlink following these instructions:
https://github.com/texane/stlink/blob/master/doc/compiling.md

Arch
```
sudo pacman -S \
  arm-none-eabi-gdb \
  arm-none-eabi-binutils \
  qemu-arch-extra \
  openocd \
  stlink
```

# Setup

To check the idVender and idProduct, plug the st-link and run dmesg:
```
sudo dmesg -T
```

Add udev rules

```
sudo vim /etc/udev/rules.d/70-st-link.rules
```

```
# STM32F3DISCOVERY rev A/B - ST-LINK/V2
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3748", TAG+="uaccess"

# STM32F3DISCOVERY rev C+ - ST-LINK/V2-1
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374b", TAG+="uaccess"
```

```
sudo udevadm control --reload-rules
```

# Cargo project

Project Name: app
```
cargo generate --git https://github.com/rust-embedded/cortex-m-quickstart
cd app
rm build.rs memory.x

cat << EOF > memory.x
/* Linker script for the STM32F103C8T6 */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K
  RAM : ORIGIN = 0x20000000, LENGTH = 20K
}
EOF
```
Add stm32f1xx-hal crate and select the microcontroller using the corresponding
feature.  Also add the nb (non-blocking) crate:
```
vim Cargo.toml
```
```
[...]

[dependencies]
[...]
nb = "0.1.1"

[dependencies.stm32f1xx-hal]
version = "0.2.1"
features = ["stm32f103", "rt"]

[...]
```



Edit src/main.rs and copy the following:
```rust
#![no_std]
#![no_main]

extern crate panic_halt;

use nb::block;

use cortex_m_rt::entry;
use stm32f1xx_hal::{pac, prelude::*, timer::Timer};

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store
    // the frozen frequencies in `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Acquire the GPIOC peripheral
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, 1.hz(), clocks);

    // Wait for the timer to trigger an update and change the state of the LED
    loop {
        block!(timer.wait()).unwrap();
        led.set_high();
        block!(timer.wait()).unwrap();
        led.set_low();
    }
}
```

## Flashing configuration

We will use openocd (a fantastic free software microcontroller debugger, which
supports a wide range of devices) to flash the binary (and also to debug later).

We need a configuration file for openocd to specify the details of our microcontroller and the programming interface.

Edit the `openocd.cfg` to look like this:
```
# Revision C (newer revision)
# source [find interface/stlink-v2-1.cfg]

# Revision A and B (older revisions)
source [find interface/stlink-v2.cfg]

source [find target/stm32f1x.cfg]
```

# Build and flash

Build:
```
cargo build --release
arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/release/app app.bin
```

Erase and flash with openocd:
```
openocd -f openocd.cfg -c "program app.bin reset exit 0x8000000"
```

You should see the green LED blinking ;-)

You can use the `flash.sh` script for convenience.

`flash.sh`
```
#!/bin/sh

set -ex

NAME=`basename ${PWD}`

cargo build --release
arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/release/${NAME} ${NAME}.bin

# stlink version
# st-flash erase
# st-flash write ${NAME}.bin 0x8000000

# OpenOCD version
# http://openocd.org/doc/html/Flash-Programming.html
openocd -f openocd.cfg -c "program ${NAME}.bin reset exit 0x8000000"
```

# Debugging

Edit `.cargo/config` to add these lines:

```
[...]

[target.'cfg(all(target_arch = "arm", target_os = "none"))']

[...]

# uncomment ONE of these three option to make `cargo run` start a GDB session
# which option to pick depends on your system
runner = "arm-none-eabi-gdb -q -x openocd.gdb"
# runner = "gdb-multiarch -q -x openocd.gdb"
# runner = "gdb -q -x openocd.gdb"
```

We will use the `openocd.cfg` file we edited before.

Download gdb-dashboard for a friendlier gdb interface:
```
wget https://raw.githubusercontent.com/cyrus-and/gdb-dashboard/master/.gdbinit -O gdb-dashboard
```

Edit `openocd.gdb` like this:
```
source gdb-dashboard
dashboard -style syntax_highlighting "monokai"

target extended-remote :3333

# print demangled symbols
set print asm-demangle on

# set backtrace limit to not have infinite backtrace loops
set backtrace limit 32

# detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
break rust_begin_unwind

# *try* to stop at the user entry point (it might be gone due to inlining)
break main

monitor arm semihosting enable

# # send captured ITM to the file itm.fifo
# # (the microcontroller SWO pin must be connected to the programmer SWO pin)
# # 8000000 must match the core clock frequency
# monitor tpiu config internal itm.txt uart off 8000000

# # OR: make the microcontroller SWO pin output compatible with UART (8N1)
# # 8000000 must match the core clock frequency
# # 2000000 is the frequency of the SWO pin
# monitor tpiu config external uart off 8000000 2000000

# # enable ITM port 0
# monitor itm port 0 on

load

# start the process but immediately halt the processor
stepi
```

Add semihosting crate to enable debug printing

In `Cargo.toml`:
```
[dependencies]
[...]
panic-semihosting = "0.5.1"
```

Edit `src/main.rs` like this:

```
#![no_std]
#![no_main]

extern crate cortex_m_semihosting;
extern crate panic_semihosting;

use nb::block;

use cortex_m_rt::entry;
use stm32f1xx_hal::{pac, prelude::*, timer::Timer};

use cortex_m_semihosting::hprintln;

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store
    // the frozen frequencies in `clocks`
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Acquire the GPIOC peripheral
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    // Configure gpio C pin 13 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port. For pins 0-7, crl should be passed instead.
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, 1.hz(), clocks);

    // Wait for the timer to trigger an update and change the state of the LED
    loop {
        block!(timer.wait()).unwrap();
        led.set_high();
        hprintln!("LED off").unwrap();
        block!(timer.wait()).unwrap();
        led.set_low();
        hprintln!("LED on").unwrap();
    }
}
```

Now in a terminal run openocd, which will connect to the hardware debugger and
show the debug print output:
```
openocd -f openocd.cfg
```

In another terminal run a gdb (the client debugger) instance to load and start
the program.  We have redefined what `cargo run` should do in `.cargo/config`.
In particular, after building the program, it will run `arm-none-eabi-gdb -q -x
openocd.gdb`, which in turn runs all the gdb commands from `openocd.gdb` script
before starting an interactive gdb session:

```
cargo run
```

The gdb script, among other things, adds a breakpoint to the main function and
halts execution at the first instruction.  To continue the program press `c`
(this will go to the main function and break), then press `c` again.  You
should see now the debug print output in the openocd console :)

At any point press `Ctrl-C` in the gdb console to pause execution and inspect
the state of the program.  Add breakpoints to a line with `b LINE`.  For more
information on how to use gdb, run `man gdb`.  For more information about the
gdb-dashboard, see https://github.com/cyrus-and/gdb-dashboard/

# Wrap up

You can find an example of the complete setup in the [app folder](/app).

# Links

- [Rust embedded book](https://rust-embedded.github.io/book/intro/index.html)
- [Blue Pill info](https://wiki.stm32duino.com/index.php?title=Blue_Pill)
- [STM32F103C8 info and datasheets](https://www.st.com/en/microcontrollers-microprocessors/stm32f103c8.html)
- [stm32f1xx-hal crate](https://github.com/stm32-rs/stm32f1xx-hal)
- [awesome-embedded-rust](https://github.com/rust-embedded/awesome-embedded-rust)
- [SSD1306 (OLED screen) crate)](https://github.com/jamwaffles/ssd1306)
- [Fixed-point numbers crate](https://docs.rs/fixed/0.3.2/fixed/index.html) (Remember that the stm32f103 doesn't have FPU)
