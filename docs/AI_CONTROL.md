# AI Control Contract

Goal: let local agents safely inspect and flash WCH devices without scraping UI text.

The first stable interface should be CLI JSON. A later MCP server can wrap the same
core operations from `meowisp-core`.

## Safety Rules

- Default to read-only commands: `doctor`, `probe`, `info`, `config read`.
- Flash, erase and config write commands must require explicit parameters.
- Destructive commands must return the detected chip before acting when possible.
- Output must be structured JSON on stdout; logs and progress go to stderr or event streams.
- Every operation result must include `ok`, `operation`, `transport`, `device`, and `messages`.

## Proposed Commands

```bash
meowisp ai doctor --json
meowisp ai catalog --json
meowisp ai probe --json
meowisp ai info --json
meowisp ai flash --file firmware.bin --plan --json
meowisp ai flash --file firmware.bin --verify --reset --json
meowisp ai verify --file firmware.bin --json
meowisp ai erase --region code --json
meowisp ai erase --region data --json
meowisp ai config read --json
meowisp ai config schema --json
meowisp ai config write --set CFG_BOOT_EN=1 --set CFG_DEBUG_EN=1 --json
```

Catalog response:

```json
{
  "source": "vendor-wchisp/devices",
  "family_count": 16,
  "variant_count": 85,
  "families": [
    {
      "name": "CH59x Series",
      "device_type_hex": "0x22",
      "transports": {
        "usb": "supported",
        "serial": "supported",
        "net": "unsupported"
      },
      "variants": [
        {
          "name": "CH592",
          "chip_id_hex": "0x92",
          "memory_regions": []
        }
      ]
    }
  ]
}
```

## JSON Shapes

Probe response:

```json
{
  "ok": true,
  "operation": "probe",
  "transport": "usb",
  "device": {
    "name": "CH592",
    "chip_id": "0x92",
    "device_type": "0x22",
    "flash_size": 458752,
    "eeprom_size": 32768
  },
  "messages": []
}
```

Config schema response:

```json
{
  "ok": true,
  "operation": "config.schema",
  "device": { "name": "CH592", "device_type": "0x22" },
  "registers": [
    {
      "offset": 0,
      "name": "RESERVED",
      "reset": null,
      "fields": []
    },
    {
      "offset": 8,
      "name": "USER_CFG",
      "fields": [
        {
          "name": "CFG_BOOT_EN",
          "bit_range": [16, 16],
          "kind": "bool",
          "value": true,
          "reset_value": true
        }
      ]
    }
  ]
}
```

Flash response:

Implemented now: `meowisp ai flash --file firmware.bin --plan --json` returns a
read-only plan. Apply mode remains intentionally blocked until guarded UI
planning and device validation are wired through the same `OperationPlan`.

```json
{
  "ok": true,
  "operation": "flash",
  "transport": "usb",
  "device": { "name": "CH592", "chip_id": "0x92" },
  "firmware": {
    "path": "firmware.bin",
    "size": 131072,
    "padded_size": 131072
  },
  "steps": [
    { "name": "erase", "ok": true },
    { "name": "program", "ok": true },
    { "name": "verify", "ok": true },
    { "name": "reset", "ok": true }
  ],
  "messages": []
}
```

## MCP Wrapper Target

Expose the same operations as tools:

- `meowisp_doctor`
- `meowisp_probe`
- `meowisp_flash`
- `meowisp_verify`
- `meowisp_erase`
- `meowisp_config_schema`
- `meowisp_config_read`
- `meowisp_config_write`

The MCP layer should call the Rust binary or link `meowisp-core`; it should not
reimplement the WCH protocol.
