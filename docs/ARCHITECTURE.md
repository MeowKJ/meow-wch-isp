# Architecture

```text
meowisp app
  Slint desktop UI
  CLI automation entry
      |
      v
meowisp-core
  device probe
  flash / verify / erase
  online firmware lookup
  permission helpers
  future config schema facade
      |
      v
vendor-wchisp
  WCH ISP protocol
  USB / serial transports
  firmware parsers
  chip database from YAML
```

The project intentionally keeps `vendor-wchisp` close to upstream so Meow-ISP can
track WCH protocol and device database improvements with minimal merge pain.

## Planned Split

- `meowisp-core`: stable Rust API for UI, CLI and AI wrappers.
- `meowisp`: desktop UI and CLI binary.
- `vendor-wchisp`: vendored baseline of `ch32-rs/wchisp`.
- `docs`: public contracts, device support and design notes.

## Release Targets

- macOS arm64
- macOS x64
- Linux x64
- Linux arm64
