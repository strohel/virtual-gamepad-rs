use anyhow::{Context, Result};
use clap::{arg, command, Parser};
use evdev::uinput::VirtualDevice;
use evdev::{
    uinput::VirtualDeviceBuilder, AbsInfo, AbsoluteAxisType, EventType, InputEvent, UinputAbsSetup,
};
use evdev::{AttributeSet, BusType, Device, InputEventKind, InputId, Key};
use log::{debug, trace};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

/// Convert an evdev keyboard to a virtual gamepad.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to input keyboard evdev device.
    #[arg(
        short,
        long,
        default_value = PathBuf::from("/dev/input/by-id/usb-Lenovo_ThinkPad_Compact_USB_Keyboard_with_TrackPoint-event-kbd").into_os_string(),
    )]
    keyboard: PathBuf,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    // TODO: nicer error on bad permissions
    let mut keyboard = Device::open(&args.keyboard)
        .with_context(|| format!("opening keyboard device {}", args.keyboard.display()))?;
    keyboard
        .grab()
        .context("grabbing the keyboard for exclusive access")?;

    let mut gamepad = Gamepad::new()?;

    loop {
        for keyboard_event in keyboard
            .fetch_events()
            .context("fetching keyboard events")?
        {
            let InputEventKind::Key(keyboard_key) = keyboard_event.kind() else {
                trace!("Ignoring non-Key event {:?}", keyboard_event);
                continue;
            };

            gamepad.handle_keyboard_key(keyboard_key, keyboard_event.value())?;
        }
    }
}

struct Gamepad {
    device: VirtualDevice,
    keys: [Key; 6],
    axis_mapping: BTreeMap<(Key, i32), (AbsoluteAxisType, i32)>,
}

impl Gamepad {
    fn new() -> Result<Gamepad> {
        let keys = [
            Key::BTN_SOUTH, // a.k.a. BTN_A
            Key::BTN_EAST,  // a.k.a. BTN_B
            Key::BTN_NORTH, // a.k.a. BTN_X
            Key::BTN_WEST,  // a.k.a. BTN_Y
            Key::BTN_START,
            Key::BTN_SELECT,
        ];

        let axis_mapping = [
            ((Key::KEY_LEFT, 1), (AbsoluteAxisType::ABS_X, 0)),
            ((Key::KEY_LEFT, 0), (AbsoluteAxisType::ABS_X, 256)),
            ((Key::KEY_RIGHT, 1), (AbsoluteAxisType::ABS_X, 512)),
            ((Key::KEY_RIGHT, 0), (AbsoluteAxisType::ABS_X, 256)),
        ]
        .into();

        let abs_setup = AbsInfo::new(256, 0, 512, 20, 20, 1);
        let abs_x = UinputAbsSetup::new(AbsoluteAxisType::ABS_X, abs_setup);
        let abs_y = UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, abs_setup);

        let name = "Microsoft X-Box 360 pad";
        let bus_type = BusType::BUS_USB; // VIRTUAL?
        let vendor = 0x045e;
        let product = 0x028e;
        let version = 0x110;

        let mut key_set = AttributeSet::<Key>::new();
        for key in keys {
            key_set.insert(key);
        }

        let mut device = VirtualDeviceBuilder::new()?
            .name(name)
            .input_id(InputId::new(bus_type, vendor, product, version))
            .with_absolute_axis(&abs_x)?
            .with_absolute_axis(&abs_y)?
            .with_keys(&key_set)?
            .build()?;

        for path in device.enumerate_dev_nodes_blocking()? {
            let path = path?;
            println!("Available as {}", path.display());
        }

        Ok(Gamepad {
            device,
            keys,
            axis_mapping,
        })
    }

    fn handle_keyboard_key(&mut self, key: Key, value: i32) -> Result<()> {
        debug!("Got {key:?} {value}");

        if let Some(&(axis, axis_value)) = self.axis_mapping.get(&(key, value)) {
            let event = InputEvent::new(EventType::ABSOLUTE, axis.0, axis_value);
            debug!("Emitting {event:?}");
            self.device.emit(&[event])?;
        }

        //     let down_event = InputEvent::new(EventType::ABSOLUTE, axis_code, 0);
        //     let key_press = InputEvent::new(EventType::KEY, key.code(), 1);
        //     gamepad.emit(&[down_event, key_press]).unwrap();

        //     let up_event = InputEvent::new(EventType::ABSOLUTE, axis_code, 512);
        //     let key_release = InputEvent::new(EventType::KEY, key.code(), 0);
        //     gamepad.emit(&[up_event, key_release]).unwrap();

        Ok(())
    }
}
