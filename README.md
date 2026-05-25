# Meow-ISP

Meow-ISP is a cross-platform WCH ISP flashing tool for desktop users and AI agents.
It is based on the Rust `wchisp` implementation from `ch32-rs/wchisp`, with a Slint
desktop UI, a scriptable CLI, automatic device detection, and a roadmap for fully
visual chip configuration registers.

Primary targets:

- macOS and Linux first. Windows packaging is intentionally out of scope.
- WCH ISP devices over USB, with serial transport exposed through the CLI.
- Friendly desktop flashing for CH55x, CH57x, CH58x, CH59x, CH32V/CH32F, CH32X and CH32L families supported by the bundled `wchisp` device database.
- AI-safe automation: machine-readable probe, flash, verify, erase and config flows.

## Current Status

- Rust workspace with `meowisp-core` and `meowisp` desktop app.
- Slint UI with automatic USB detection, firmware selection, flash, verify, code erase and data erase.
- CLI commands for doctor, probe, info, flash, verify, erase, reset, EEPROM and config info/reset.
- Bundled `vendor-wchisp` device database with 16 WCH families and 85 variants.
- Linux udev helper and portable packaging scripts.

The current desktop UI is focused on safe flashing. Full visual editing for every
chip configuration bitfield is planned and specified in [docs/CONFIG_UI.md](docs/CONFIG_UI.md).

## Build

```bash
cargo build --bin meowisp
```

Release build:

```bash
cargo build --bin meowisp --release
```

## Run

Desktop UI:

```bash
cargo run --bin meowisp
```

CLI:

```bash
cargo run --bin meowisp -- doctor
cargo run --bin meowisp -- probe
cargo run --bin meowisp -- catalog --json
cargo run --bin meowisp -- ai catalog --json
cargo run --bin meowisp -- info
cargo run --bin meowisp -- flash --file firmware.bin
cargo run --bin meowisp -- verify --file firmware.bin
cargo run --bin meowisp -- config info
```

Compatibility flags are also kept:

```bash
meowisp --doctor
meowisp --probe
```

## Packaging

```bash
python3 scripts/package_portable.py \
  --platform macos \
  --arch arm64 \
  --version dev \
  --binary target/release/meowisp \
  --out-dir dist
```

Linux packages include `assets/50-wchisp.rules`.

## Release Automation

Pushing a tag like `v0.1.0` runs CI, builds portable packages, and uploads them
to the GitHub Release:

- macOS arm64
- macOS x64
- Linux amd64
- Linux arm64

## Project Documents

- [Supported devices](docs/SUPPORTED_DEVICES.md)
- [AI control contract](docs/AI_CONTROL.md)
- [AI development plan](docs/AI_DEVELOPMENT_PLAN.md)
- [UI design specification](docs/UI_DESIGN_SPEC.md)
- [Configuration UI plan](docs/CONFIG_UI.md)
- [Project configuration format](docs/PROJECT_CONFIG.md)
- [Architecture notes](docs/ARCHITECTURE.md)

## License

This repository is licensed as GPL-2.0 because it links and vendors `wchisp`,
which is GPL-2.0. See [LICENSE](LICENSE) and [NOTICE.md](NOTICE.md).
