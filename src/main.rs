// K380 Fn keys switcher
// by faust93

extern crate hidapi;

use hidapi::HidApi;
use std::env;
use std::process;

const HID_VENDOR_ID_LOGITECH: u16 = 0x46d;
const HID_DEVICE_ID_K380: u16 = 0xb342;

const K380_SEQ_FKEYS_ON: [u8; 7] = [0x10, 0xff, 0x0b, 0x1e, 0x00, 0x00, 0x00];
const K380_SEQ_FKEYS_OFF: [u8; 7] = [0x10, 0xff, 0x0b, 0x1e, 0x01, 0x00, 0x00];

const OPT_ON: &str = "on";
const OPT_OFF: &str = "off";

#[derive(Copy, Clone)]
enum Command {
    ListDevices,
    SwitchFKeys(bool),
}

enum AppError {
    Usage,
    Failure(String),
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args.get(0).map(|s| s.as_str()).unwrap_or("k380-macos");

    match run(&args) {
        Ok(()) => {}
        Err(AppError::Usage) => {
            print_usage(program);
        }
        Err(AppError::Failure(message)) => {
            eprintln!("{}", message);
            process::exit(1);
        }
    }
}

fn run(args: &[String]) -> Result<(), AppError> {
    let command = parse_command(args)?;
    let api = HidApi::new()
        .map_err(|err| AppError::Failure(format!("Unable to initialize HID subsystem: {}", err)))?;

    match command {
        Command::ListDevices => {
            list_logitech_devices(&api);
            Ok(())
        }
        Command::SwitchFKeys(fkeys_on) => {
            set_fkeys(&api, fkeys_on)?;
            println!("F-n keys: {}", if fkeys_on { "ON" } else { "OFF" });
            Ok(())
        }
    }
}

fn parse_command(args: &[String]) -> Result<Command, AppError> {
    let flag = args.get(1).ok_or(AppError::Usage)?;

    match flag.as_str() {
        "-l" => Ok(Command::ListDevices),
        "-f" => {
            let value = args
                .get(2)
                .ok_or_else(|| AppError::Failure(String::from("-f option requires an argument")))?;
            match value.as_str() {
                OPT_ON => Ok(Command::SwitchFKeys(true)),
                OPT_OFF => Ok(Command::SwitchFKeys(false)),
                _ => Err(AppError::Failure(format!(
                    "Invalid argument for -f: {}",
                    value
                ))),
            }
        }
        other => Err(AppError::Failure(format!("Unknown option: {}", other))),
    }
}

fn set_fkeys(api: &HidApi, fkeys_on: bool) -> Result<(), AppError> {
    // Open the K380 device and send the corresponding HID report.
    let device = api
        .open(HID_VENDOR_ID_LOGITECH, HID_DEVICE_ID_K380)
        .map_err(|err| AppError::Failure(format!("Unable to open device: {}", err)))?;
    let sequence = if fkeys_on {
        &K380_SEQ_FKEYS_ON
    } else {
        &K380_SEQ_FKEYS_OFF
    };

    match device.write(sequence) {
        Ok(7) => Ok(()),
        Ok(_) => Err(AppError::Failure(String::from(
            "Unable to switch F-n keys.",
        ))),
        Err(err) => Err(AppError::Failure(format!(
            "Error writing to device: {}",
            err
        ))),
    }
}

fn list_logitech_devices(api: &HidApi) {
    println!("Listing HID devices:");
    for device in api.device_list() {
        if device.vendor_id() == HID_VENDOR_ID_LOGITECH {
            println!("Device Found");
            println!(
                "  type: {:04x} {:04x}",
                device.vendor_id(),
                device.product_id()
            );
            println!("  path: {}", device.path().to_str().unwrap_or("<non-utf8>"));
            println!("  serial_number: {:?}", device.serial_number());
            println!("  Manufacturer: {:?}", device.manufacturer_string());
            println!("  Product: {:?}", device.product_string());
            println!("  Release: {:x}", device.release_number());
            println!("  Interface: {}", device.interface_number());
            println!(
                "  Usage (page): 0x{:x} (0x{:x})",
                device.usage(),
                device.usage_page()
            );
            println!();
        }
    }
}

fn print_usage(program: &str) {
    println!("Logitech K380 Keyboard Configurator");
    println!("Usage: {} -f {{on|off}}", program);
    println!("-f <on|off>");
    println!("   To enable/disable direct access to F-keys.");
    println!("-l");
    println!("   To list enabled HID devices.");
}
