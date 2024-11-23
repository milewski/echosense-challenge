# EchoSense Device

This is the source code of the device built for the [AssemblyAI Challenge](https://dev.to/challenges/assemblyai).

It is build in Rust using the Espressif IDF Framework.

### Getting Started

Install esp32 [toolchain](https://github.com/esp-rs/rust-build)

```shell
cargo install espup
cargo install ldproxy
cargo install espflash
espup install
```

### Pinout

Ensure that the pins has been properly mapped to the components:

![image](https://github.com/user-attachments/assets/65c3d52b-c2f5-4f70-9276-a2a013095726)

### Build

To flash the firmware into the board run the following command:

```shell
cargo run --release
```

