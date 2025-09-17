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
    AutoSwitch(bool),
}

enum AppError {
    Usage,
    Failure(String),
}

impl AppError {
    fn into_message(self) -> String {
        match self {
            AppError::Usage => String::from("Invalid command usage."),
            AppError::Failure(message) => message,
        }
    }
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
    match parse_command(args)? {
        Command::ListDevices => {
            let api = create_hid_api()?;
            list_logitech_devices(&api);
            Ok(())
        }
        Command::SwitchFKeys(fkeys_on) => {
            let api = create_hid_api()?;
            set_fkeys(&api, fkeys_on)?;
            println!("F-n keys: {}", if fkeys_on { "ON" } else { "OFF" });
            Ok(())
        }
        Command::AutoSwitch(fkeys_on) => run_auto_mode(fkeys_on),
    }
}

fn create_hid_api() -> Result<HidApi, AppError> {
    HidApi::new()
        .map_err(|err| AppError::Failure(format!("Unable to initialize HID subsystem: {}", err)))
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
        "--auto" => {
            let value = args.get(2).ok_or_else(|| {
                AppError::Failure(String::from("--auto option requires an argument"))
            })?;
            match value.as_str() {
                OPT_ON => Ok(Command::AutoSwitch(true)),
                OPT_OFF => Ok(Command::AutoSwitch(false)),
                _ => Err(AppError::Failure(format!(
                    "Invalid argument for --auto: {}",
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

#[cfg(target_os = "macos")]
fn run_auto_mode(fkeys_on: bool) -> Result<(), AppError> {
    auto_mode::run(fkeys_on)
}

#[cfg(target_os = "macos")]
mod auto_mode {
    use super::{create_hid_api, set_fkeys, AppError, HID_DEVICE_ID_K380, HID_VENDOR_ID_LOGITECH};
    use core_foundation::base::TCFType;
    use core_foundation::dictionary::CFMutableDictionary;
    use core_foundation::number::CFNumber;
    use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
    use core_foundation::set::CFSet;
    use core_foundation::string::CFString;
    use core_foundation_sys::base::{kCFAllocatorDefault, CFRelease};
    use core_foundation_sys::set::CFSetGetValues;
    use io_kit_sys::hid::base::IOHIDDeviceRef;
    use io_kit_sys::hid::manager::{
        kIOHIDManagerOptionNone, IOHIDManagerClose, IOHIDManagerCopyDevices, IOHIDManagerCreate,
        IOHIDManagerOpen, IOHIDManagerRef, IOHIDManagerRegisterDeviceMatchingCallback,
        IOHIDManagerRegisterDeviceRemovalCallback, IOHIDManagerScheduleWithRunLoop,
        IOHIDManagerSetDeviceMatching, IOHIDManagerUnscheduleFromRunLoop,
    };
    use io_kit_sys::ret::{kIOReturnNotPermitted, kIOReturnSuccess, IOReturn};
    use std::ffi::c_void;
    use std::thread;
    use std::time::Duration;

    const BACKOFF_SECONDS: u64 = 1;
    const MAX_ATTEMPTS: usize = 3;

    pub(super) fn run(fkeys_on: bool) -> Result<(), AppError> {
        unsafe {
            let manager = IOHIDManagerCreate(kCFAllocatorDefault, kIOHIDManagerOptionNone);
            if manager.is_null() {
                return Err(AppError::Failure(String::from(
                    "Unable to create IOHIDManager instance.",
                )));
            }

            let mut matching = CFMutableDictionary::new();
            matching.set(
                CFString::from_static_string("VendorID"),
                CFNumber::from(i32::from(HID_VENDOR_ID_LOGITECH)),
            );
            matching.set(
                CFString::from_static_string("ProductID"),
                CFNumber::from(i32::from(HID_DEVICE_ID_K380)),
            );

            IOHIDManagerSetDeviceMatching(manager, matching.as_concrete_TypeRef());

            let state = Box::into_raw(Box::new(AutoState { fkeys_on }));

            IOHIDManagerRegisterDeviceMatchingCallback(
                manager,
                device_matching_callback,
                state as *mut c_void,
            );
            let open_result = IOHIDManagerOpen(manager, kIOHIDManagerOptionNone);
            if open_result != kIOReturnSuccess {
                CFRelease(manager as _);
                drop(Box::from_raw(state));
                return Err(AppError::Failure(format!(
                    "IOHIDManagerOpen failed with code 0x{:x}",
                    open_result
                )));
            }

            IOHIDManagerRegisterDeviceRemovalCallback(
                manager,
                device_removal_callback,
                state as *mut c_void,
            );

            let run_loop = CFRunLoop::get_current();
            IOHIDManagerScheduleWithRunLoop(
                manager,
                run_loop.as_concrete_TypeRef(),
                kCFRunLoopDefaultMode,
            );

            AutoState::apply_existing_devices(state, manager);

            println!(
                "Auto mode armed. Waiting for Logitech K380 connections (F-n keys -> {}).",
                if fkeys_on { "ON" } else { "OFF" }
            );
            println!("Press Ctrl+C to exit.");

            CFRunLoop::run_current();

            IOHIDManagerUnscheduleFromRunLoop(
                manager,
                run_loop.as_concrete_TypeRef(),
                kCFRunLoopDefaultMode,
            );
            IOHIDManagerClose(manager, kIOHIDManagerOptionNone);
            CFRelease(manager as _);
            drop(Box::from_raw(state));

            Ok(())
        }
    }

    struct AutoState {
        fkeys_on: bool,
    }

    impl AutoState {
        fn on_connected(&self, result: IOReturn) {
            println!(
                "Logitech K380 connected. Applying F-n keys {}...",
                if self.fkeys_on { "ON" } else { "OFF" }
            );

            if result != kIOReturnSuccess {
                if let Some(hint) = describe_io_return(result) {
                    eprintln!(
                        "Device connection reported error (0x{:x}, {}). Continuing attempts...",
                        result, hint
                    );
                } else {
                    eprintln!(
                        "Device connection reported error (0x{:x}). Continuing attempts...",
                        result
                    );
                }
            }

            self.apply_fkey_mode();
        }

        fn on_disconnected(&self) {
            println!("Logitech K380 disconnected. Standing by.");
        }

        fn apply_fkey_mode(&self) {
            for attempt in 1..=MAX_ATTEMPTS {
                match create_hid_api() {
                    Ok(api) => match set_fkeys(&api, self.fkeys_on) {
                        Ok(()) => {
                            println!("F-n keys switched successfully.");
                            return;
                        }
                        Err(err) => {
                            eprintln!(
                                "Attempt {} failed to switch F-n keys: {}",
                                attempt,
                                err.into_message()
                            );
                        }
                    },
                    Err(err) => {
                        eprintln!(
                            "Attempt {} failed to initialize HID API: {}",
                            attempt,
                            err.into_message()
                        );
                    }
                }

                if attempt < MAX_ATTEMPTS {
                    thread::sleep(Duration::from_secs(BACKOFF_SECONDS));
                }
            }

            eprintln!(
                "Unable to switch F-n keys after {} attempts. Waiting for the next device event.",
                MAX_ATTEMPTS
            );
        }

        unsafe fn apply_existing_devices(state_ptr: *mut AutoState, manager: IOHIDManagerRef) {
            let devices_ref = IOHIDManagerCopyDevices(manager);
            if devices_ref.is_null() {
                return;
            }

            let devices = CFSet::<IOHIDDeviceRef>::wrap_under_create_rule(devices_ref);
            if devices.is_empty() {
                return;
            }

            let count = devices.len();
            let mut buffer: Vec<IOHIDDeviceRef> = Vec::with_capacity(count);
            CFSetGetValues(
                devices.as_concrete_TypeRef(),
                buffer.as_mut_ptr() as *mut *const c_void,
            );
            buffer.set_len(count);

            let state = &*state_ptr;
            for _device in buffer {
                state.on_connected(kIOReturnSuccess);
            }
        }
    }

    unsafe extern "C" fn device_matching_callback(
        context: *mut c_void,
        result: IOReturn,
        _sender: *mut c_void,
        _device: IOHIDDeviceRef,
    ) {
        if context.is_null() {
            return;
        }
        let state = &*(context as *const AutoState);
        state.on_connected(result);
    }

    unsafe extern "C" fn device_removal_callback(
        context: *mut c_void,
        _result: IOReturn,
        _sender: *mut c_void,
        _device: IOHIDDeviceRef,
    ) {
        if context.is_null() {
            return;
        }
        let state = &*(context as *const AutoState);
        state.on_disconnected();
    }

    fn describe_io_return(code: IOReturn) -> Option<&'static str> {
        if code == kIOReturnNotPermitted {
            Some("not permitted (権限不足の可能性)")
        } else {
            None
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn run_auto_mode(_fkeys_on: bool) -> Result<(), AppError> {
    Err(AppError::Failure(String::from(
        "Auto mode is only supported on macOS.",
    )))
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
    println!(
        "Usage: {} [-f {{on|off}} | --auto {{on|off}} | -l]",
        program
    );
    println!("-f <on|off>");
    println!("   To enable/disable direct access to F-keys.");
    println!("--auto <on|off>");
    println!("   To run as a daemon and apply the selected F-key mode on each connection.");
    println!("-l");
    println!("   To list enabled HID devices.");
}
