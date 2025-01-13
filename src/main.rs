// K380 Fn keys switcher
// by faust93
extern crate hidapi;
use hidapi::HidApi;
use std::process;
use std::env;

const HID_VENDOR_ID_LOGITECH: u16 = 0x46d;
const HID_DEVICE_ID_K380: u16 = 0xb342;

const K380_SEQ_FKEYS_ON: [u8; 7] = [0x10, 0xff, 0x0b, 0x1e, 0x00, 0x00, 0x00];
const K380_SEQ_FKEYS_OFF: [u8; 7] = [0x10, 0xff, 0x0b, 0x1e, 0x01, 0x00, 0x00];

const OPT_ON: &str = "on";
const OPT_OFF: &str = "off";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Logitech K380 Keyboard Configurator");
        println!("Usage: {} -f {{on|off}}", args[0]);
        println!("-f <on|off>");
        println!("   To enable/disable direct access to F-keys.");
        println!("-l");
        println!("   To list enabled HID devices.");
        return;
    }

    // Initialize HID API
    let api = match HidApi::new() {
        Ok(api) => api,
        Err(err) => {
            eprintln!("Unable to initialize HID subsystem: {}", err);
            process::exit(1);
        }
    };

    let fkeys_on: bool;

    if args[1] == "-l" {
        println!("Listing HID devices:");
        for device in api.device_list() {
            if device.vendor_id() == HID_VENDOR_ID_LOGITECH {
                println!("Device Found");
                println!("  type: {:04x} {:04x}", device.vendor_id(), device.product_id());
                println!("  path: {}", device.path().to_str().unwrap());
                println!("  serial_number: {:?}", device.serial_number());
                println!("  Manufacturer: {:?}", device.manufacturer_string());
                println!("  Product: {:?}", device.product_string());
                println!("  Release: {:x}", device.release_number());
                println!("  Interface: {}", device.interface_number());
                println!("  Usage (page): 0x{:x} (0x{:x})", device.usage(), device.usage_page());
                println!();
            }
        }
        return;
    } else if args[1] == "-f" {
        if args.len() < 3 {
            println!("-f option requires an argument");
            process::exit(1);
        }
        match args[2].as_str() {
            OPT_ON => fkeys_on = true,
            OPT_OFF => fkeys_on = false,
            _ => { println!("Invalid argument for -f: {}", args[2]);
                   process::exit(1);
            }
        }
    } else {
            println!("Unknown option: {}", args[1]);
            process::exit(1);
    }

    let handle = match api.open(HID_VENDOR_ID_LOGITECH, HID_DEVICE_ID_K380) {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("Unable to open device: {}", e);
            process::exit(1);
        }
    };

    // Send the F-keys configuration
    let sequence = if fkeys_on { K380_SEQ_FKEYS_ON } else { K380_SEQ_FKEYS_OFF };
    match handle.write(&sequence) {
        Ok(7) => {
            println!("F-n keys: {}", if fkeys_on { "ON" } else { "OFF" });
        }
        Ok(_) => {
            eprintln!("Unable to switch F-n keys.");
        }
        Err(err) => {
            eprintln!("Error writing to device: {}", err);
        }
    }
}
