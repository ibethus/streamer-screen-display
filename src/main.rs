#![no_std]
#![no_main]
#![deny(warnings)]

use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text, TextStyleBuilder},
};
use embedded_hal::digital::v2::OutputPin;

// The macro for our start-up function
use rp_pico::{
    entry,
    hal::{fugit::RateExtU32, gpio, pac, spi, Clock},
};

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::SerialPort;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// A shorter alias for the Hardware Abstraction Layer, which provides
// higher-level drivers.
use rp_pico::hal;

use epd_waveshare::{epd2in9_v2::*, prelude::*};

const MAX_TEXT_BUFFER_SIZE: usize = 2048;

#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    // Set up the USB Communications Class Device driver
    let mut serial = SerialPort::new(&usb_bus);

    // Create a USB device with a fake VID and PID
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Bethus Inc.")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(0) // from: https://www.usb.org/defined-class-codes
        .build();

    // SPI declaration
    let _spi_sclk = pins.gpio10.into_function::<gpio::FunctionSpi>();
    let _spi_mosi = pins.gpio11.into_function::<gpio::FunctionSpi>();
    let spi = spi::Spi::<_, _, _, 8>::new(pac.SPI1, (_spi_mosi, _spi_sclk));

    let mut spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        4_000_000u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );
    // End of SPI declaration

    // Start the rest of pins needed to communicate with the screen
    let mut cs = pins.gpio9.into_push_pull_output(); // CS
    cs.set_high().unwrap();
    let busy = pins.gpio13.into_pull_up_input(); // BUSY
    let dc = pins.gpio8.into_push_pull_output(); // DC
    let rst = pins.gpio12.into_push_pull_output(); // RST

    // Start the EPD struct
    let mut epd =
        Epd2in9::new(&mut spi, cs, busy, dc, rst, &mut delay).expect("e-ink initalize error");

    // Use display graphics from embedded-graphics
    let mut display = Display2in9::default();
    display.set_rotation(DisplayRotation::Rotate90);

    // Check for usb inputs
    loop {
        let mut input_buffer = [0u8; MAX_TEXT_BUFFER_SIZE];
        // Check for new data
        if usb_dev.poll(&mut [&mut serial]) {
            match serial.read(&mut input_buffer) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(0) => {
                    // Do nothing
                }
                // If data are found, add them to buffer
                Ok(count) => {
                    serial.write(b"ok !\n").unwrap();
                    epd.wake_up(&mut spi, &mut delay).unwrap();
                    epd.clear_frame(&mut spi, &mut delay).unwrap();
                    
                    let buff_as_string = core::str::from_utf8(&input_buffer[..count]).unwrap();
                    let mut lines = buff_as_string.lines();
                    draw_text_primary(&mut display, lines.next().get_or_insert(""), 5, 5);
                    lines.enumerate().for_each(|(i, line)| {
                        draw_text_secondary(
                            &mut display,
                            line,
                            5, 
                            (25 + 20 * i).try_into().unwrap(),
                        )
                    });

                    // Setup EPD
                    epd.update_frame(&mut spi, display.buffer(), &mut delay)
                        .unwrap();
                    epd.display_frame(&mut spi, &mut delay)
                        .expect("display frame new graphics");
                    // Set the EPD to sleep
                    epd.sleep(&mut spi, &mut delay).unwrap();
                }
            }
        }
    }
}

// Draw functions
fn draw_text_primary(display: &mut Display2in9, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::iso_8859_1::FONT_10X20)
        .background_color(BinaryColor::Off)
        .text_color(BinaryColor::On)
        .build();
    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();
    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}

fn draw_text_secondary(display: &mut Display2in9, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::iso_8859_1::FONT_9X18)
        .background_color(BinaryColor::Off)
        .text_color(BinaryColor::On)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();
    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}