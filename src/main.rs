// Create a virtual joystick, just while this is running.
// Generally this requires root.

use evdev::{
    uinput::VirtualDeviceBuilder, AbsInfo, AbsoluteAxisType, EventType, InputEvent, UinputAbsSetup,
};
use evdev::{AttributeSet, BusType, InputId, Key};
use std::thread::sleep;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let abs_setup = AbsInfo::new(256, 0, 512, 20, 20, 1);
    let abs_x = UinputAbsSetup::new(AbsoluteAxisType::ABS_X, abs_setup);
    let abs_y = UinputAbsSetup::new(AbsoluteAxisType::ABS_Y, abs_setup);

    let name = "Microsoft X-Box 360 pad";
    let bus_type = BusType::BUS_USB; // VIRTUAL?
    let vendor = 0x045e;
    let product = 0x028e;
    let version = 0x110;

    let keys = [
        Key::BTN_SOUTH, // a.k.a. BTN_A
        Key::BTN_EAST,  // a.k.a. BTN_B
        Key::BTN_NORTH, // a.k.a. BTN_X
        Key::BTN_WEST,  // a.k.a. BTN_Y
        Key::BTN_START,
        Key::BTN_SELECT,
    ];

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

    let type_ = EventType::ABSOLUTE;
    // Hopefully you don't have ABS_X bound to anything important.
    let mut axis_codes = [AbsoluteAxisType::ABS_X.0, AbsoluteAxisType::ABS_Y.0]
        .iter()
        .cycle();

    let mut keys = keys.iter().cycle();

    println!("Waiting for Ctrl-C...");
    loop {
        let axis_code = *axis_codes.next().unwrap();
        let key = *keys.next().unwrap();

        let down_event = InputEvent::new(type_, axis_code, 0);
        let key_press = InputEvent::new(EventType::KEY, key.code(), 1);
        device.emit(&[down_event, key_press]).unwrap();
        print!("^");
        sleep(Duration::from_millis(200));

        let up_event = InputEvent::new(type_, axis_code, 512);
        let key_release = InputEvent::new(EventType::KEY, key.code(), 0);
        device.emit(&[up_event, key_release]).unwrap();
        print!(".");
        sleep(Duration::from_millis(800));
    }
}
