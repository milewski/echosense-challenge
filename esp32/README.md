# EchoSense Device

This is the source code of the device built for the [AssemblyAI Challenge](https://dev.to/challenges/assemblyai).

The firmware is written in Rust using the Espressif IDF Framework.

### Getting Started

To get started, install the necessary [toolchain](https://github.com/esp-rs/rust-build) for the ESP32.

```shell
cargo install espup
cargo install ldproxy
cargo install espflash
espup install
```

### Pinout

Double-check the pins has been correctly connected to each component.
Here is the pinout from the esp32s3 mini board for reference:

![image](https://github.com/user-attachments/assets/65c3d52b-c2f5-4f70-9276-a2a013095726)

### Building and Flashing

To compile the code and flash the firmware to the ESP32 device, use the following command:

```shell
cargo run --release
```

This will build the firmware in release mode and upload it directly to the connected board.
