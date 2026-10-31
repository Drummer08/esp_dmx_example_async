#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;

use esp_hal::peripherals::{GPIO4, UART1};

use esp_dmx::{Channel, Dmx};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[embassy_executor::task]
async fn dmx_task(uart: UART1<'static>, tx_pin: GPIO4<'static>) {
    let mut dmx = Dmx::with_defaults(uart, tx_pin).unwrap().into_async();
    let mut delay = Delay;

    // set dimmer and rgb channel
    dmx.set_channels(Channel::new(1).unwrap(), &[255, 180, 255, 150])
        .unwrap();
    info!("DMX initialised");

    loop {
        dmx.send_frame_async(&mut delay).await.unwrap();
    }
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    info!("Embassy initialized!");

    spawner.spawn(dmx_task(peripherals.UART1, peripherals.GPIO4).unwrap());

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
