#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};

use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Event, Input, InputConfig, Io, Pull};
use esp_hal::peripherals::Peripherals;
use esp_hal::{clock::CpuClock, main, rmt::Rmt, time::Rate};
use esp_hal::{handler, ram};
use esp_hal_smartled::{SmartLedsAdapter, smart_led_buffer};
use palette::{FromColor, Hsv, Srgb, rgb};
use smart_leds::SmartLedsWrite;
use smart_leds::{RGB, RGB8};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

//const NUM_LEDS: usize = 36;
const NUM_LEDS: usize = 72;
const NUM_LEDS_ROUND: usize = 8;
const NUM_ROUNDS: usize = 9;

static BUTTON: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
static SHOULD_SWITCH: AtomicBool = AtomicBool::new(false);

#[main]
fn main() -> ! {
    // generator version: 1.0.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let mut io = Io::new(peripherals.IO_MUX);
    io.set_interrupt_handler(handler);

    let config = InputConfig::default().with_pull(Pull::Up);
    let mut button = Input::new(peripherals.GPIO5, config);

    critical_section::with(|cs| {
        button.listen(Event::FallingEdge);
        BUTTON.borrow_ref_mut(cs).replace(button)
    });

    // Configure RMT (Remote Control Transceiver) peripheral globally
    // <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/peripherals/rmt.html>
    let rmt: Rmt<'_, esp_hal::Blocking> = {
        let frequency: Rate = { Rate::from_mhz(80) };
        Rmt::new(peripherals.RMT, frequency)
    }
    .expect("Failed to initialize RMT");

    // We use one of the RMT channels to instantiate a `SmartLedsAdapter` which can
    // be used directly with all `smart_led` implementations
    let rmt_channel = rmt.channel0;
    let rmt_buffer = smart_led_buffer!(NUM_LEDS);

    let mut led = SmartLedsAdapter::new(rmt_channel, peripherals.GPIO6, rmt_buffer);

    // let mut all_leds: [RGB8; 120] = [RGB8::new(0, 0, 0); 120];
    // led.write(all_leds.iter().cloned())
    //     .expect("Could not write LED data: ");

    loop {
        rainbow(&mut led);
        //white(led)
        warm_white(&mut led);
        rotate(&mut led);
        //rocket_start(led)
    }
    // let delay = Delay::new();
    // loop {
    //     delay.delay_millis(5 * 1000);
    // }
}

fn shoud_switch() -> bool {
    if SHOULD_SWITCH.load(Ordering::Relaxed) {
        SHOULD_SWITCH.store(false, Ordering::Relaxed);
        return true;
    }
    return false;
}

//const LED_BUFFER_SIZE: usize = NUM_LEDS * 8 * 3 + 1;
const LED_BUFFER_SIZE: usize = NUM_LEDS * 8 * 3 + 1;

fn white(led: &mut SmartLedsAdapter<'_, LED_BUFFER_SIZE>) -> () {
    let mut leds: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];

    for i in 0..NUM_LEDS {
        leds[i] = RGB8::new(255, 255, 255);
    }
    led.write(leds.iter().cloned())
        .expect("Could not write LED data: ");

    let delay = Delay::new();

    loop {
        delay.delay_millis(5 * 1000);
        if shoud_switch() {
            return;
        }
    }
}

fn warm_white(led: &mut SmartLedsAdapter<'_, LED_BUFFER_SIZE>) -> () {
    let mut leds: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];

    for i in 0..NUM_LEDS {
        //leds[i] = RGB8::new(253, 100, 100);
        leds[i] = RGB8::new(255, 140, 80);
    }
    led.write(leds.iter().cloned())
        .expect("Could not write LED data: ");

    let delay = Delay::new();

    loop {
        delay.delay_millis(1000);
        if shoud_switch() {
            return;
        }
    }
}

fn rotate(led: &mut SmartLedsAdapter<'_, LED_BUFFER_SIZE>) -> () {
    let mut leds: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];

    let colors: [RGB<u8>; NUM_LEDS_ROUND] = [
        RGB8::new(128, 0, 0),
        RGB8::new(0, 128, 0),
        RGB8::new(0, 0, 128),
        RGB8::new(128, 128, 0),
        RGB8::new(128, 0, 128),
        RGB8::new(0, 128, 128),
        RGB8::new(128, 64, 0),
        RGB8::new(64, 128, 64),
    ];

    let mut loop_counter = 0;
    loop {
        for i in 0..NUM_LEDS {
            leds[i] = match i {
                x if x % NUM_LEDS_ROUND == 0 => colors[(0 + loop_counter) % NUM_LEDS_ROUND],
                x if x % NUM_LEDS_ROUND == 1 => colors[(1 + loop_counter) % NUM_LEDS_ROUND],
                x if x % NUM_LEDS_ROUND == 2 => colors[(2 + loop_counter) % NUM_LEDS_ROUND],
                x if x % NUM_LEDS_ROUND == 3 => colors[(3 + loop_counter) % NUM_LEDS_ROUND],
                x if x % NUM_LEDS_ROUND == 4 => colors[(4 + loop_counter) % NUM_LEDS_ROUND],
                x if x % NUM_LEDS_ROUND == 5 => colors[(5 + loop_counter) % NUM_LEDS_ROUND],
                x if x % NUM_LEDS_ROUND == 6 => colors[(6 + loop_counter) % NUM_LEDS_ROUND],
                x if x % NUM_LEDS_ROUND == 7 => colors[(7 + loop_counter) % NUM_LEDS_ROUND],
                _ => RGB8::new(0, 0, 0),
            }
        }
        led.write(leds.iter().cloned())
            .expect("Could not write LED data: ");

        loop_counter += 1;
        if loop_counter >= NUM_LEDS_ROUND {
            loop_counter = 0;
        }

        let delay = Delay::new();

        delay.delay_millis(750);
        if shoud_switch() {
            return;
        }
    }
}

fn rocket_start(led: &mut SmartLedsAdapter<'_, LED_BUFFER_SIZE>) -> () {
    let mut leds: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];

    let colors: [RGB<u8>; NUM_ROUNDS] = [
        RGB8::new(128, 128, 0),
        RGB8::new(255, 128, 0),
        RGB8::new(255, 0, 0),
        RGB8::new(0, 0, 0),
        RGB8::new(0, 0, 0),
        RGB8::new(0, 0, 0),
        RGB8::new(0, 0, 0),
        RGB8::new(0, 0, 0),
        RGB8::new(0, 0, 0),
    ];

    let mut loop_counter: i8 = NUM_ROUNDS as i8 - 1;
    loop {
        for i in 0..NUM_LEDS {
            leds[i] = match i {
                x if x < 10 => colors[(0 + loop_counter as usize) % NUM_ROUNDS],
                x if x < 20 => colors[(1 + loop_counter as usize) % NUM_ROUNDS],
                x if x < 30 => colors[(2 + loop_counter as usize) % NUM_ROUNDS],
                x if x < 40 => colors[(3 + loop_counter as usize) % NUM_ROUNDS],
                x if x < 50 => colors[(4 + loop_counter as usize) % NUM_ROUNDS],
                x if x < 60 => colors[(5 + loop_counter as usize) % NUM_ROUNDS],
                x if x < 70 => colors[(6 + loop_counter as usize) % NUM_ROUNDS],
                x if x < 80 => colors[(7 + loop_counter as usize) % NUM_ROUNDS],
                _ => RGB8::new(0, 0, 0),
            };
        }
        led.write(leds.iter().cloned())
            .expect("Could not write LED data: ");

        loop_counter -= 1;
        if loop_counter < 0 {
            loop_counter = NUM_ROUNDS as i8 - 1;
        }

        let delay = Delay::new();
        delay.delay_millis(500);
        if shoud_switch() {
            return;
        }
    }
}

fn rainbow(led: &mut SmartLedsAdapter<'_, LED_BUFFER_SIZE>) -> () {
    let mut h: f32 = 0.0;
    let s: f32 = 1.0;
    let v: f32 = 0.1;
    let mut leds: [RGB8; NUM_LEDS] = [RGB8::default(); NUM_LEDS];

    let delay = Delay::new();

    loop {
        h += 2.0;
        if h > 360.0 {
            h = 0.0;
        }

        for i in 0..NUM_LEDS {
            let rgb = Srgb::from_color(Hsv::new(h + (i as f32 * 5.0), s, v));
            let (r, g, b) = (
                (rgb.red * 255.0) as u8,
                (rgb.green * 255.0) as u8,
                (rgb.blue * 255.0) as u8,
            );
            leds[i] = RGB8::new(r, g, b);
        }
        led.write(leds.iter().cloned())
            .expect("Could not write LED data: ");
        delay.delay_millis(25u32);
        if shoud_switch() {
            return;
        }
    }
}

#[handler]
#[ram]
fn handler() {
    if critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .is_interrupt_set()
    }) {
        esp_println::println!("Button was the source of the interrupt");
        SHOULD_SWITCH.store(true, Ordering::Relaxed);
        let delay = Delay::new();
        delay.delay_millis(100);
    } else {
        esp_println::println!("Button was not the source of the interrupt");
    }

    critical_section::with(|cs| {
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt()
    });
}
