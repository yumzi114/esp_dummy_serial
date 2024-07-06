#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]



use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pipe::Pipe};
use esp32c3_hal::{
    clock::ClockControl,
    embassy,
    Rng,
    peripherals::{Peripherals, UART0},
    prelude::*,
    uart::{config::{self, AtCmdConfig}, TxRxPins, UartRx, UartTx},
    Uart, IO,
};
use portable_atomic::AtomicU32;
use core::sync::atomic::Ordering;
use embassy_time::{Duration, Timer};
use esp32c3_hal::gpio::{AnyPin, Input, PullUp};
use embedded_hal_async::digital::Wait;
use esp_println::println;
use esp_backtrace as _;
use menu::*;

static BLINK_DELAY: AtomicU32 = AtomicU32::new(1000_u32);
// Read Buffer Size
const READ_BUF_SIZE: usize = 64;
// End of Transmission Character (Carrige Return -> 13 or 0x0D in ASCII)
const AT_CMD: u8 = 0x0D;
static DATAPIPE: Pipe<CriticalSectionRawMutex, READ_BUF_SIZE> = Pipe::new();
const DATA_BYTE: &str = env!("DATA_BYTE");
#[embassy_executor::task]
async fn one_second_task() {
    let mut count = 0;
    loop {
        esp_println::println!("Spawn Task Count: {}", count);
        count += 1;
        Timer::after(Duration::from_millis(1_000)).await;
    }
}
#[embassy_executor::task]
async fn uart_writer(mut tx: UartTx<'static, UART0>) {
    // Declare write buffer to store Tx characters
    let mut wbuf: [u8; READ_BUF_SIZE] = [8u8; READ_BUF_SIZE];
    loop {
        Timer::after(Duration::from_millis(BLINK_DELAY.load(Ordering::Relaxed) as u64)).await;
        // DATAPIPE.read(&mut wbuf).await;
        // embedded_io_async::Write::write(&mut tx, &wbuf)
        //     .await
        //     .unwrap();
        // println!("{:?}",wbuf);
        // embedded_io_async::Write::write(&mut tx, &[0x0D, 0x0A])
        //     .await
        //     .unwrap();
        // embedded_io_async::Write::flush(&mut tx).await.unwrap();
        // if let Ok(_)=embedded_io_async::Write::write(&mut tx, &[0x0D, 0x0A])
        //     .await{
        //         // wbuf.trim();
        //         let s: &str = core::str::from_utf8(&wbuf).unwrap();
        //         // let v: Vec<_> = s.split('\r').collect();
        //         match s {
        //             "1"=>{
        //                 println!("1");
        //                 embedded_io_async::Write::flush(&mut tx).await.unwrap();
        //             },
        //             "2"=>{
        //                 println!("2");
        //                 embedded_io_async::Write::flush(&mut tx).await.unwrap();
        //             },
        //              _ =>{
        //                 println!("{:?}",s);
        //                 println!("{:?}",wbuf);
        //                 embedded_io_async::Write::flush(&mut tx).await.unwrap();
        //              }
        //         }
        //         // s.strip_suffix("d").unwrap();
        //         // s.strip_suffix("\\r").unwrap();
        //         // s.strip_suffix("\\0").unwrap();
                
                
        // }
        // embedded_io_async::Write::write(
        //     &mut tx,
        //     b"UART Task Spawned. Waiting for Button Press...\r\n",
        // )
        // .await
        // .unwrap();
        // Read characters from pipe into write buffer
        // DATAPIPE.read(&mut wbuf).await;
        // // Transmit/echo buffer contents over UART
        // embedded_io_async::Write::write(&mut tx, &wbuf)
        //     .await
        //     .unwrap();
        // // Transmit a new line
        // embedded_io_async::Write::write(&mut tx, &[0x0D, 0x0A])
        //     .await
        //     .unwrap();
        // // Flush transmit buffer
        // embedded_io_async::Write::flush(&mut tx).await.unwrap();
    }
}


#[embassy_executor::task]
async fn uart_reader(mut rx: UartRx<'static, UART0>) {
    // Declare read buffer to store Rx characters
    let mut rbuf: [u8; READ_BUF_SIZE] = [0u8; READ_BUF_SIZE];
    loop {
        // Read characters from UART into read buffer until EOT
        let r = embedded_io_async::Read::read(&mut rx, &mut rbuf[0..]).await;
        match r {
            Ok(len) => {
                // If read succeeds then write recieved characters to pipe
                DATAPIPE.write_all(&rbuf[..len]).await;
            }
            Err(e) => esp_println::println!("RX Error: {:?}", e),
        }
    }
}
#[main]
async fn main(spawner: Spawner) {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let mut buffer = [0u8; 64];
    // let mut r = Runner::new(ROOT_MENU, &mut buffer, Output(window), &mut context);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Initialize Embassy with needed timers
    let timer_group0 = esp32c3_hal::timer::TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0.timer0);
    // Configure UART
    // Create handle for UART config struct
    let config = config::Config::default().baudrate(115_200);
    // let pin = UartPins;
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let pins = TxRxPins::new_tx_rx(
        io.pins.gpio21.into_push_pull_output(),
        io.pins.gpio20.into_floating_input(),
    );
    let del_but = io.pins.gpio3.into_pull_up_input().degrade();
    let mut leds = [
        io.pins.gpio1.into_push_pull_output().degrade(),
        io.pins.gpio10.into_push_pull_output().degrade(),
        io.pins.gpio19.into_push_pull_output().degrade(),
        io.pins.gpio18.into_push_pull_output().degrade(),
        io.pins.gpio4.into_push_pull_output().degrade(),
        io.pins.gpio5.into_push_pull_output().degrade(),
        io.pins.gpio6.into_push_pull_output().degrade(),
        io.pins.gpio7.into_push_pull_output().degrade(),
        io.pins.gpio8.into_push_pull_output().degrade(),
        io.pins.gpio9.into_push_pull_output().degrade(),
    ];
    esp32c3_hal::interrupt::enable(
        esp32c3_hal::peripherals::Interrupt::GPIO,
        esp32c3_hal::interrupt::Priority::Priority1,
    )
    .unwrap();
    let mut dd = Uart::new_with_config(peripherals.UART0, config, Some(pins), &clocks);
    // // Instantiate UART
    // let mut uart = UartDriver::new(
    //     peripherals.uart0,
    //     peripherals.pins.gpio21,
    //     peripherals.pins.gpio20,
    //     Option::<gpio::Gpio0>::None,
    //     Option::<gpio::Gpio1>::None,
    //     &config,
    // )
    // .unwrap();
    // Initialize and configure UART0
    // let mut uart0 = Uart::new(peripherals.UART0, &clocks);
    // uart0.
    dd.set_at_cmd(AtCmdConfig::new(None, None, None, AT_CMD, None));
    dd
    .set_rx_fifo_full_threshold(READ_BUF_SIZE as u16)
    .unwrap();
    // Split UART0 to create seperate Tx and Rx handles
    let (tx, rx) = dd.split();

    // Spawn Tx and Rx tasks
    spawner.spawn(uart_reader(rx)).ok();
    spawner.spawn(uart_writer(tx)).ok();
    spawner.spawn(press_button(del_but)).unwrap();
    esp_println::print!("\x1b[20h");
    loop {
        for led in &mut leds {
            led.set_high().unwrap();
            Timer::after(Duration::from_millis(BLINK_DELAY.load(Ordering::Relaxed) as u64)).await;
            led.set_low().unwrap();
            Timer::after(Duration::from_millis(100)).await;
        }
    }
}


// const ROOT_MENU: Menu<UartDriver> = Menu {
//     label: "root",
//     items: &[&Item {
//         item_type: ItemType::Callback {
//             function: hello_name,
//             parameters: &[Parameter::Mandatory {
//                 parameter_name: "name",
//                 help: Some("Enter your name"),
//             }],
//         },
//         command: "hw",
//         help: Some("This is an embedded CLI terminal. Check the summary for the list of supported commands"),
//     }],
//     entry: None,
//     exit: None,
// };

// fn hello_name<'a>(
//     _menu: &Menu<UartDriver>,
//     item: &Item<UartDriver>,
//     args: &[&str],
//     context: &mut UartDriver,
// ) {
//     // Print to console passed "name" argument
//     writeln!(
//         context,
//         "Hello, {}!",
//         argument_finder(item, args, "name").unwrap().unwrap()
//     )
//     .unwrap();
// }


#[embassy_executor::task]
async fn press_button(mut button: AnyPin<Input<PullUp>>) {
    loop {
      // Wait for Button Press
      button.wait_for_rising_edge().await.unwrap();
      esp_println:: println!("Button Pressed!");
      // Retrieve Delay Global Variable
      let del = BLINK_DELAY.load(Ordering::Relaxed);
      // Adjust Delay Accordingly
      if del <= 10_u32 {
        BLINK_DELAY.store(1000_u32,Ordering::Relaxed);
        esp_println:: println!("Delay is now 1000ms");
      } else {
        BLINK_DELAY.store(del - 10_u32,Ordering::Relaxed);
        esp_println:: println!("Delay is now {}ms", del - 10_u32);
      } 
    }
}