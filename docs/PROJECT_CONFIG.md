# Project Configuration

Goal: make flashing repeatable for boards, products and CI jobs.

The project file should describe which chip is expected, which firmware should be
used, and which configuration bits must be applied or checked.

Default file name:

```text
meowisp.project.toml
```

Example:

```toml
[project]
name = "CH592 keyboard bootloader"

[target]
chip = "CH592"
transport = "usb"

[firmware]
path = "dist/CH592F-5KEY-full.bin"
format = "bin"
verify = true
reset_after_flash = true

[config]
mode = "check"

[config.bits]
CFG_BOOT_EN = true
CFG_DEBUG_EN = true
CFG_RESET_EN = true
```

Modes:

- `check`: read config and fail if expected bits differ.
- `apply`: show a diff, ask for confirmation in UI, and write in CLI only when
  an explicit `--yes` flag is present.
- `reset`: restore default/non-protected/debug-enabled config, then apply bits.

Planned commands:

```bash
meowisp ai project plan --file meowisp.project.toml --json
meowisp project check meowisp.project.toml
meowisp project flash meowisp.project.toml
meowisp project apply-config meowisp.project.toml --yes
```

Implemented now:

- `meowisp ai project plan --file meowisp.project.toml --json`
- target chip matching against the bundled rs-wchisp catalog
- firmware path resolution relative to the project file
- config bit validation against the target register schema
- embedded guarded flash plan when the firmware file is present

The UI should expose project config as a left-to-right flow:

1. Open project.
2. Detect connected chip.
3. Compare expected chip and current chip.
4. Show firmware and config diff.
5. Flash, verify, then optionally apply config.
