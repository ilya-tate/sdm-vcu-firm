# SDM EV: _**Vehicle Control Unit Firmware**_

### Status

- A custom PCB using the STM32-G4 is currently in development
- CAN communcation w/ motor controller is functional
- Voltage readable via potentiometers, then sends torque to motor controller

### Documentation

- Rust: [Embedded Rust Book](https://docs.rust-embedded.org/book)
- Framework: [Embassy Book](https://embassy.dev/book)
- HAL: [embassy-stm32](https://docs.embassy.dev/embassy-stm32)
- Runtime: [embassy-executor](https://docs.embassy.dev/embassy-executor)
- Debugger: [probe-rs](https://probe.rs/docs/getting-started/probe-setup/)

### Hardware References

- Dev Board: [NUCLEO-G474RE](https://www.st.com/en/evaluation-tools/nucleo-g474re.html)
- Microcontroller Family: [STM32G4](https://www.st.com/en/microcontrollers-microprocessors/stm32g4-series.html)
- Dev Board Microcontroller: [STM32G474RE](https://www.st.com/en/microcontrollers-microprocessors/stm32g474re.html)

### Pinout

| Pin  | Header | Peripheral | Function                     |
| ---- | ------ | ---------- | ---------------------------- |
| PA0  | A0     | ADC1       | Accelerator pedal position 1 |
| PA1  | A1     | ADC1       | Accelerator pedal position 2 |
| PB0  | A3     | ADC1       | Brake pedal position         |
| PA11 | CN10   | FDCAN1.RX  | CAN transceiver input        |
| PA12 | CN10   | FDCAN1.TX  | CAN transceiver output       |

### Motor Controller Configuration

| Parameter                  | Address | Value           |
| -------------------------- | ------- | --------------- |
| Inverter Command Mode      | 0x8f    | CAN mode        |
| Inverter Run Mode          | 0x8e    | Torque mode     |
| CAN Command Message Active | 0x92    | Enabled         |
| CAN Timeout                | 0xac    | Nonzero         |
| CAN Bit Rate               |         | 500 kbps        |
| CAN ID Offset              |         | 0x0a0 (default) |
