# AI Development Plan

Meow WCH ISP is a general-purpose WCH ISP operation engine with UI, CLI and AI
frontends. It must not drift back into a board-specific or keyboard-specific
flashing utility.

## Product Goal

Build a macOS/Linux desktop app and automation stack for WCH devices supported
by `wchisp`:

- automatic device discovery and identification
- generic firmware flashing and verification
- memory region read/write/erase
- visual configuration register inspection and safe editing
- repeatable project files
- AI-safe JSON CLI and later MCP tools

## Architecture Rule

All frontends must use the same core pipeline:

```text
DeviceCatalog -> DeviceSession -> OperationPlan -> OperationGuard -> EventStream
```

UI, CLI and MCP must not call low-level flashing functions directly.

## Phase 0: Ground Truth

Deliverables:

- capability matrix for every family and variant in `vendor-wchisp/devices`
- list of protocol-backed operations versus YAML-only metadata
- explicit unknown/unsupported state for incomplete capabilities

Acceptance:

- UI can explain why a feature is unavailable for a connected device.
- AI output can distinguish `supported`, `unsupported`, `unknown` and `unsafe`.

## Phase 1: Core Abstractions

Core types:

- `Transport`: USB, serial and mock
- `DeviceSession`: connected device state and supported operations
- `DeviceCatalog`: device families, variants, memory regions and config schema
- `MemoryRegion`: code flash, data flash, EEPROM and config regions
- `OperationPlan`: pre-execution plan for every destructive or write operation
- `OperationGuard`: chip match, file bounds, permissions, risk and confirmation
- `EventStream`: progress and result events shared by UI, CLI and MCP

Acceptance:

- UI no longer reaches directly into `wchisp::Flashing`.
- CLI JSON and UI use the same operation result model.
- Mock sessions can run in CI without hardware.

## Phase 2: UI Information Architecture

The app should be task-flow driven, not page-list driven.

The full visual design contract lives in [UI_DESIGN_SPEC.md](UI_DESIGN_SPEC.md).
UI implementation must follow that file for window size, typography, colors,
spacing, animation, component behavior and Computer Use QA.

Primary navigation:

- `Dashboard`: connection, detected chip, permissions, current session health
- `Flash`: firmware selection, plan preview, flash/verify/reset
- `Memory`: device memory regions and region-specific operations
- `Config`: config registers, bitfields, raw values, diffs and guarded writes
- `Project`: open and run `meowisp.project.toml`
- `Catalog`: searchable supported-device catalog
- `Automation`: JSON CLI/MCP status, recent operation events, copyable state

Dashboard must be the first screen. It should answer:

- what device is connected?
- what transport is active?
- what operations are available?
- what is unsafe or unavailable, and why?
- what should the user do next?

## Phase 3: UI State Model

Global UI states:

- no device
- scanning
- one device identified
- multiple devices found
- unsupported or unknown device
- permission problem
- busy operation
- recoverable error
- destructive operation confirmation

Rules:

- Destructive buttons are disabled unless a valid operation plan exists.
- Disabled controls must have a reason available in tooltip or detail text.
- The UI must not imply EEPROM/DataFlash/config support unless the catalog says so.
- Long chip names, register names and error messages must not resize fixed controls.

## Phase 4: Flash MVP

Goal: first alpha release.

UI:

- select `.bin`, `.hex` or `.elf` when supported by parser
- show parsed file size and target region
- show expected versus detected chip when project metadata exists
- preview erase/program/verify/reset steps before execution
- stream progress from `EventStream`
- show final report with copyable JSON

CLI:

```bash
meowisp ai probe --json
meowisp ai flash --file firmware.bin --plan --json  # implemented: read-only plan
meowisp ai flash --file firmware.bin --plan --probe-device --json  # implemented: live capacity guard
meowisp ai flash --file firmware.bin --apply --json
meowisp ai verify --file firmware.bin --json
```

Acceptance:

- no device means flash controls are disabled
- chip mismatch blocks apply
- firmware larger than target region blocks apply
- macOS and Linux release artifacts are produced from tags

## Phase 5: Memory UI

Use `MemoryRegion`, not hard-coded EEPROM pages.

Each region displays:

- name and kind
- start address and size
- readable, writable and erasable flags
- danger level
- supported operations
- disabled reason if unavailable

Operations:

- read/export
- write/import
- erase
- verify region against file

All write/erase operations must use `OperationPlan`.

## Phase 6: Config UI Read-Only

Goal: make every known config register inspectable before write support.

Register view:

- register name
- offset/address
- current raw value
- reset/default value
- field list
- reserved mask
- writable mask
- description

Field view:

- field name
- bit range
- decoded current value
- enum options when known
- danger level
- reset requirement
- constraints and notes

UI controls are read-only in this phase.

## Phase 7: Config UI Editing

Editing creates a target config only; it never writes immediately.

Control mapping:

- one-bit fields: toggle
- enum fields: dropdown or segmented control
- numeric fields: bounded input
- reserved fields: read-only
- full register: hex view with guarded raw-edit mode

Before apply, show a diff:

- register
- field
- old value
- new value
- meaning
- risk
- reset/reconnect requirement

Safety:

- reserved bits keep current value by default
- unknown writable masks block writing
- high-risk fields require second confirmation
- write is followed by readback verification

## Phase 8: Project Files

Project files describe intent, not direct execution.

Minimum shape:

```toml
[project]
name = "example"
version = 1

[target]
chip = "CH592"
family = "CH59x"
transport = "auto"

[firmware]
path = "firmware.bin"
format = "auto"

[flash]
erase = "auto"
verify = true
reset_after = true

[config]
mode = "check"

[config.fields]
CFG_BOOT_EN = true
CFG_DEBUG_EN = true
```

Project UI flow:

```text
open project -> detect device -> compare target -> preview plan -> apply
```

## Phase 9: AI and MCP

Order:

1. stable JSON CLI
2. operation-plan JSON schema
3. MCP wrapper
4. AI apply mode

AI safety rules:

- default tools are read-only
- write operations require plan plus explicit apply
- destructive results include detected chip and final verification
- all errors include `code`, `message`, `hint` and `recoverable`

## Phase 10: UI Testing With Computer Use

Use real UI screenshots plus mock sessions.

Required scenarios:

- no device
- scanning
- identified CH55x USB device
- identified CH59x USB device
- serial-only CH32V00x
- multiple devices
- Linux permission missing
- unsupported device
- firmware too large
- config field high-risk diff
- operation progress
- operation failure

Each UI test should capture:

- screenshot
- visible state summary
- enabled/disabled destructive controls
- text overflow or overlap notes
- safety verdict

Acceptance:

- no incoherent overlap
- disabled controls are visually clear
- dangerous actions are not primary unless a valid plan exists
- config diffs are understandable without reading raw hex only

## AI Iteration Protocol

Every AI development turn should follow:

```text
Goal
Scope
Non-goals
Design update
Implementation
Verification
Safety review
Handoff notes
```

Rules:

- keep changes small and phase-aligned
- update docs when behavior or safety model changes
- add mock tests before hardware-only flows
- never bypass `OperationGuard`
- never make config write a side effect of flashing
