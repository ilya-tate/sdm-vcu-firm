# Gitutorial

### Dev Dependencies

- [rustup](https://rustup.rs)
- [probe-rs](https://probe.rs/docs/getting-started/installation/)
- [git](https://git-scm.com/downloads)

### First-Time Startup

**Clone:**

```bash
git clone https://github.com/ilya-tate/sdm-vcu-firm.git
cd sdm-vcu-firm
rustup target add thumbv7em-none-eabihf
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
cargo fmt                # Format build
cargo build --release    # Test build
git add <edited-files>   # Ex. git add bruh.txt src/foo.rs
# Pro Tip:
# "git add -A" to add all edited files (run "git status" first tho)
git commit -m "DESCRIPTION" # Ex. git commit -m "Fixed this bug which caused that problem"
git push -u origin HEAD
```

### Requested Changes

- Sometimes you may be asked to make a change to your pull request

**1. Change branches (only if necessary)**

```bash
git checkout <github-username>/<feature-name>
```

**2. Make requested edits**

_Type away and test_

**3. Push changes**

```bash
cargo build                    # Test build
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
cargo run            # Debug and internal loopback
cargo run --relase   # Normal, requires CAN tranceiver
```

> #### _Don't hesitate to DM me (Ilya) on Slack if you run into any issues :)_
