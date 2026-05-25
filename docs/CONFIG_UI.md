# Configuration UI Plan

Goal: every WCH configuration register and bitfield from the `wchisp` device YAML
should become a visual, validated control.

## Data Source

The source of truth is `vendor-wchisp/devices/*.yaml`.

Each family can define:

- `config_registers`
- register `offset`, `name`, `reset`
- field `name`, `bit_range`, values and descriptions
- per-variant overrides

Runtime flow:

1. Detect connected chip.
2. Resolve the family and variant through `wchisp::ChipDB`.
3. Read current config registers from the device.
4. Merge current values with schema metadata.
5. Render one panel per register.
6. Let the user change fields using typed controls.
7. Preview raw register diffs before writing.
8. Write only after explicit confirmation.

## UI Control Mapping

| bitfield shape | UI control |
| --- | --- |
| one bit, named enable/disable field | toggle |
| small enum with explanations | segmented control or dropdown |
| numeric range | stepper/input |
| reserved/unknown bit | read-only raw bit |
| full register | hex value with copy button |

## Required States

- Current value
- Reset/default value
- Pending changed value
- Dirty indicator
- Validation error
- Raw register preview
- Restore register default
- Restore all defaults

## First Implementation Slice

1. Add public config schema structs to `meowisp-core`.
2. Expose `config_schema_for_connected_chip()`.
3. Expose `read_config_for_connected_chip()`.
4. Add CLI JSON commands from [AI_CONTROL.md](AI_CONTROL.md).
5. Add a Slint config sheet opened from the device card.
6. Support read-only schema display first.
7. Add toggles and diff preview.
8. Add guarded write.

## Non-goals

- Do not hand-code per-chip forms.
- Do not hide reserved fields from the raw register view.
- Do not write config silently as part of flashing.
