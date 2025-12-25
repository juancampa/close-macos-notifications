# Close macOS Notifications

A simple Rust program to quickly close all active macOS notifications using Accessibility APIs.

You can use something like [hammerspoon](https://www.hammerspoon.org/) to bind it to a keyboard shortcut.

For example (binds to `cmd + escape`):
```lua
function closeNotifications()
  hs.execute('<path to close-notifications binary>')
end
hs.hotkey.bind({"cmd"}, "escape", closeNotifications)
```

### Run via Cargo
To run the tool directly:
```bash
cargo run --release
```

Or with debug logging:
```bash
RUST_LOG=debug cargo run --release
```

The compiled binary can be found at `target/release/close-notifications`.

### Test notifications

You can generate dummy notifications and run the closer automatically with:

```bash
./test.sh
```

### Accessibility Permissions

This program requires **Accessibility** permissions.
When you run it for the first time, macOS will prompt you to grant access in *System Settings > Privacy & Security > Accessibility*.
