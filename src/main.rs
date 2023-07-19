#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;
use hal::{
    clock::ClockControl,
    ledc::{channel, timer, LowSpeed, LEDC},
    peripherals::Peripherals,
    prelude::*,
    riscv::asm::nop,
    timer::TimerGroup,
    Delay, Rtc, IO,
};

macro_rules! set_next_point {
    ($prev:literal,$next:literal,[$channel1:expr,$($channel:expr),*]) => {

        $channel1
            .start_duty_fade($prev, $next, 500)
            .expect("fade fault");
        $( $channel
            .start_duty_fade($prev, $next, 500)
            .expect("fade fault");
        )+
        while $channel1.is_duty_fade_running_hw(){
            unsafe {
            nop();
        }
    }
    };
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;

    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    println!("Hello world!");

    // Set GPIO7 as an output, and set its state high initially.
    let mut io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let (r_led, g_led, b_led) = (
        io.pins.gpio3.into_push_pull_output(),
        io.pins.gpio4.into_push_pull_output(),
        io.pins.gpio5.into_push_pull_output(),
    );

    // Set up the led controller.
    // _instance is the peripheral instance
    // clock control is an reference to the set system clocks
    // system is the mutable access to the peripheral clock
    let mut ledc = LEDC::new(
        peripherals.LEDC,
        &clocks,
        &mut system.peripheral_clock_control,
    );

    ledc.set_global_slow_clock(hal::ledc::LSGlobalClkSource::APBClk);

    //set up low speed timer
    let mut lstimer0 = ledc.get_timer::<LowSpeed>(timer::Number::Timer0);
    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty10Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: 1u32.kHz(),
        })
        .unwrap();

    let mut r_ch = ledc.get_channel(channel::Number::Channel0, r_led);
    r_ch.configure(channel::config::Config {
        timer: &lstimer0,
        duty_pct: 0,
        pin_config: channel::config::PinConfig::PushPull,
    })
    .unwrap();
    let mut g_ch = ledc.get_channel(channel::Number::Channel1, g_led);
    g_ch.configure(channel::config::Config {
        timer: &lstimer0,
        duty_pct: 0,
        pin_config: channel::config::PinConfig::PushPull,
    })
    .unwrap();
    let mut b_ch = ledc.get_channel(channel::Number::Channel2, b_led);
    b_ch.configure(channel::config::Config {
        timer: &lstimer0,
        duty_pct: 0,
        pin_config: channel::config::PinConfig::PushPull,
    })
    .unwrap();

    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    let mut delay = Delay::new(&clocks);

    let button = io.pins.gpio9.into_pull_up_input();

    loop {
        let mut count = 0;
        println!("going up");

        set_next_point!(0, 8, [r_ch, g_ch, b_ch]);
        set_next_point!(8, 32, [r_ch, g_ch, b_ch]);
        set_next_point!(32, 100, [r_ch, g_ch, b_ch]);

        println!("going down");
        set_next_point!(100, 32, [r_ch, g_ch, b_ch]);
        set_next_point!(32, 8, [r_ch, g_ch, b_ch]);
        set_next_point!(8, 0, [r_ch, g_ch, b_ch]);
    }
}
