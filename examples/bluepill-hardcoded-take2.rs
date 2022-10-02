//! Sends the hard encodes bits to switch on the power socket on channel *A* to
//! the GPIO pin B15 of a blue pill board.

#![deny(unsafe_code)]
#![no_std]
#![cfg_attr(not(doc), no_main)]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::{pac, prelude::*, delay::Delay};

#[entry]
fn main() -> ! {
    // Init buffers for debug printing
    rtt_init_print!();
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);
    // The `clocks` handle ensures that the clocks are now configured and gives
    // the `Delay::new` function access to the configured frequency. With
    // this information it can later calculate how many cycles it has to
    // wait. The function also consumes the System Timer peripheral, so that no
    // other function can access it. Otherwise the timer could be reset during a
    // delay.
    // The delay is more flexible than a timer that is configured for a ceratin frequency.
    let mut delay = Delay::new(cp.SYST, clocks);

    // Acquire the GPIOC peripheral
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    // Configure gpio B, pin 15 as a push-pull output. The `crh` register is passed to the function
    // in order to configure the port.
    let mut output_pin = gpiob.pb15.into_push_pull_output(&mut gpiob.crh);

    // bit sequence to switch on the power socket on channel *A*.
    static BUF: [u8; 10] = [
        0b10010011, 0b01001001, 0b00100110, 0b11011010, 0b01101101, 0b10100100, 0b10011011,
        0b01001101, 0b10100100, 0b11111100,
    ];

    rprintln!("Sending buffer!");

    // send the sequence twenty times
    for repetition in 1..=20 {
        
        rprintln!("round {:?}", &repetition);

        for byte in &BUF {
            for bit_pos in (0..8).rev() {
                let p: u8 = 1 << bit_pos;
                match byte & p == 0 {
                    true => {
                        output_pin.set_low().unwrap();
                    }
                    false => {
                        output_pin.set_high().unwrap();
                    }
                }
                delay.delay_us(500_u16);
            }
        }

        output_pin.set_low().unwrap();
        delay.delay_ms(7_u16);
    }

    rprintln!("done!");
    panic!("panic!");
}
