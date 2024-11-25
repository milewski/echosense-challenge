# EchoSense Device

This is the source code of the device built for the [AssemblyAI Challenge](https://dev.to/challenges/assemblyai).

The firmware is written in Rust using the Espressif IDF Framework.

<details>
  
  <summary>Images</summary>
  
![20241125-114049](https://github.com/user-attachments/assets/794586fb-dfc9-46ba-ac26-b5b1d4553c4e)
![20241125-114040](https://github.com/user-attachments/assets/bf78f08d-4453-4b1b-b98c-a88da17ecc0b)
![20241125-114025](https://github.com/user-attachments/assets/2fabd4d0-a22c-4839-ae0f-3fa4e2ba5ffa)


 
</details>

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
Here is the pinout from the esp32s3-zero board for reference:

![image](https://github.com/user-attachments/assets/d85cb448-d394-4962-9222-27fc855eb353)

### Building and Flashing

To compile the code and flash the firmware to the ESP32 device, use the following command:

```shell
cargo run --release
```

This will build the firmware in release mode and upload it directly to the connected board.
