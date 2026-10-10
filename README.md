# SDM EV: _**Vehicle Control Unit Firmware**_

**Status:**

- Pending setup implementation of dev board tools and template code
- A custom PCB using the STM32-G4 is currently in development

## Gitutorial:

### Dev Dependencies

- [rustup](https://rustup.rs)
- [probe-rs](https://probe.rs/docs/getting-started/installation/)
- [git](https://git-scm.com/downloads)
- [github-cli](https://cli.github.com)
  > Optional if using SSH for auth

### First-Time Startup

**Authenticate GitHub CLI:**

```bash
gh auth login
# Follow CLI prompts
```

**Clone:**

```bash
git clone https://github.com/ilya-tate/sdm-vcu-firm.git
cd sdm-vcu-firm
rustup target add thumbv7em-none-eabi
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
cargo build  # Test build
cargo fmt    # Format build
git add <edited-files> # Ex. git add bruh.txt src/foo.rs
# Pro Tip:
# "git add -A" to add all edited files (run "git status" first tho)
git commit -m "DESCRIPTION" # Ex. git commit -m "Fixed this bug which caused that problem"
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

## Run

**Building**

```bash
cargo build --release
```

**Flashing**
```bash
cargo run --release
```

> #### _Don't hesitate to DM me (Ilya) on Slack if you run into any issues :)_

## Documentation

- Rust: [Embedded Rust Book](https://docs.rust-embedded.org/book)
- Hardware Access Layer: [stm32g4xx-hal](https://github.com/stm32-rs/stm32g4xx-hal)
- Flashing: [cargo-flash](https://probe.rs/docs/tools/cargo-flash/)
- Probe (Debugging): [probe-rs](https://probe.rs/docs/getting-started/probe-setup/)

## Hardware References

- Dev Board: [NUCLEO-G474RE](https://www.st.com/en/evaluation-tools/nucleo-g474re.html)
- Microcontroller Family: [STM32G4](https://www.st.com/en/microcontrollers-microprocessors/stm32g4-series.html)
- Dev Board Microcontroller: [STM32G474RE](https://www.st.com/en/microcontrollers-microprocessors/stm32g474re.html)

## Progress Tracking

[Taiga.io Kanban Board](https://tree.taiga.io/project/ilya-tate-sdm-vcu-firmware/kanban)
