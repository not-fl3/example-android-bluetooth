# Quad bluetooth

Code from "How to write Java with miniquad" tutorial.
Showcases miniquad<->java interop.
More of a Java native plugin example than a real usable crate!

![image](https://user-images.githubusercontent.com/910977/177202055-9c983b20-2217-47f3-aa89-fe42bf6a39f5.png)

## How to use

- Add a "quad-bt" crate as a dependency to Cargo.toml:

```toml
[dependencies]
quad-bt = { path = "../quad-bt" } # functionality of example-android-bluetooth would probably be not sufficient, so local copy for local hacks is preferable!
```

- Add a required bluetooth metadata to Cargo.toml:

```
[[package.metadata.android.permission]]
name = "android.permission.BLUETOOTH"
max_sdk_version = 30

[[package.metadata.android.permission]]
name = "android.permission.BLUETOOTH_ADMIN"

[[package.metadata.android.permission]]
name = "android.permission.ACCESS_FINE_LOCATION"

[[package.metadata.android.permission]]
name = "android.permission.BLUETOOTH_SCAN"

[[package.metadata.android.permission]]
name = "android.permission.BLUETOOTH_CONNECT"
```

- Init a BluetoothAdapter in the rust code:

```rust
    let mut adapter = bt::Adapter::new().unwrap();

    ...
```

for the full example check "examples/discover.rs"
