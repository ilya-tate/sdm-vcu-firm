# SDM EV: _**Vehicle Control Unit Firmware**_

## Gitutorial:

### Dev Dependencies

- [rustup](https://rustup.rs)
- [teensy_loader_cli](https://github.com/PaulStoffregen/teensy_loader_cli)
- [git](https://git-scm.com/downloads)
- [github-cli](https://cli.github.com)
  > Optional if using SSH for auth

### First-Time Startup

**Embedded Rust Setup:**

```bash
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools-preview
cargo install cargo-binutils cargo-generate
```

**Authenticate GitHub CLI:**

```bash
gh auth login
# Follow CLI prompts
```

**Clone:**

```bash
git clone https://github.com/ilya-tate/sdm-vcu-firm.git
cd sdm-vcu-firm
```

### Making a Change

- Do this every time you want to make a change

**Setup:**

```bash
cd sdm-vcu-firm
git checkout main
git pull origin main
git checkout -b <github-username>/<feature-name>   # Ex. ilya-tate/big-papa-init
```

**Change:**

_Edit, finalize, and test (please) your changes_

**Upload:**

```bash
cargo build    # Test build
git add <edited-files>                  # Ex. git add bruh.txt src/foo.rs
# Pro Tip:
# "git add -A" to add all edited files (run "git status" first tho)
git commit -m "DESCRIPTION"    # Ex. git commit -m "Fixed this bug which caused that problem"
git push -u origin HEAD
gh pr create
# Follow CLI prompts
```

### Requested Changes

- Sometimes you may be asked to make a change to your pull request

**1. Change branches (only if necessary)**

```bash
git checkout -b <github-username>/<feature-name>
```

**2. Make requested edits**

_Type away and test_

**3. Push changes**

```bash
cargo build    # Test build
git add <reedited-files>
git commit -m "DESCRIPTION"    # Just address pull request feedback
git push
```

**4. Update repo after pull request merge**

```bash
git checkout main
git pull origin main
git branch -d <github-username>/<feature-name>
```

> #### _Don't hesitate to DM me (Ilya) on Slack if you run into any issues :)_

## Run

**Building**

```bash
cargo build --release
```

**Flashing**
```bash
cargo run --release
```

## Documentation

### CAN bench test

`cargo run --release` flashes a test that sends standard CAN ID `0x123`
with bytes `01 02 03 04` once per second at 500 kbps. This test traffic is
for a bench bus; remove the `can_test` task spawn before vehicle integration.

Use a 3.3 V logic-compatible CAN transceiver (pin 1 to TXD, pin 0 to RXD),
with CANH/CANL connected to a second active CAN node at 500 kbps and 120 ohm
termination at each end. The peer must acknowledge frames, not run in silent mode.

Watch the Teensy's USB serial logs (typically `/dev/ttyACM0` on Linux):

- `CAN TX queued`: a frame was submitted, not yet confirmed sent.
- `CAN TX complete`: the controller completed transmission.
- `CAN RX id=... data=...`: a standard data frame was received from the peer.
- `CAN TX still pending`: no completed transmission; check the peer and wiring.
- `CAN2 init failure`: initialization failed; the error repeats every second.

Send standard ID `0x456`, bytes `AA BB CC DD` from the peer and check that the
same ID and bytes appear in the USB log. Check that the peer sees `0x123` with
`01 02 03 04`. Error counters should remain zero on a healthy bench bus.

The driver supports standard 11-bit Classical CAN data frames with 0–8 bytes.
It polls 14 RX mailboxes and uses one TX mailbox, with no software queue.
RX overrun is reported, and mailbox scan order does not guarantee arrival order.
This is a low-traffic bring-up test, not a lossless high-load receiver.

Mailbox access follows the [NXP FlexCAN driver](https://github.com/nxp-mcuxpresso/mcux-sdk/blob/main/drivers/flexcan/fsl_flexcan.c),
including RX unlocking and the reserved-mailbox transmit workaround.

- Rust: [Embedded Rust Book](https://docs.rust-embedded.org/book)
- Teensy: [Teensy board-support API](https://docs.rs/teensy4-bsp/latest/teensy4_bsp)
- HAL: [i.MX RT HAL API](https://docs.rs/imxrt-hal/latest/imxrt_hal)
- RTIC "OS": [RTIC v2 Book](https://rtic.rs/2/book/en)
- HEX File Gen: [cargo-binutils](https://github.com/rust-embedded/cargo-binutils)
- Project Template: [teensy4-rs-template](https://github.com/mciantyre/teensy4-rs-template)

## Teensy 4.1 References

- Development Board: [Teensy 4.1](https://www.pjrc.com/store/teensy41.html)
- Microchip: [NXP i.MX RT1060](https://nxp.com/products/i.MX-RT1060)
- Rust Toolchain: [teensy4-rs](https://github.com/mciantyre/teensy4-rs)
- CAN Lib: [FlexCAN_T4](https://github.com/tonton81/FlexCAN_T4)
- Flashing: [teensy_loader_cli](https://www.pjrc.com/teensy/loader_cli.html)

## Progress Tracking
[Taiga.io Kanban Board](https://tree.taiga.io/project/ilya-tate-sdm-vcu-firmware/kanban)
