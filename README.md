# virtual-gamepad

Emulate a gamepad using a USB keyboard. This allows a local co-op play of some games (like
Spiritfarer) that allow local co-op with a keyboard + gamepad, but don't allow local co-op with
keyboard + keyboard. Only works on Linux.

## Installation

1. Obtain the Rust toolchain i.e. from https://rustup.rs/
2. `cargo install --path .` to build and install the `virtual-gamepad` binary to ~/.cargo/bin/
3. Edit the file `70-virtual-gamepad.rules` and place it to `/etc/udev/rules.d/`
4. Copy the `virtual-gamepad@.service` file to `~/.config/systemd/user/`
5. Plug the device in and profit, the udev machinery should start the service automatically

## References

- https://github.com/iosonofabio/virtual_gamepad
- https://forums.bannister.org/ubbthreads.php?ubb=showflat&Number=118786
