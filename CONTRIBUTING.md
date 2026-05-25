# Contributing

Thanks for helping Meow-ISP improve WCH flashing on desktop systems.

Before opening a PR:

```bash
cargo fmt --all --check
cargo check --workspace
```

For UI or flashing changes, include:

- Platform tested: macOS, Linux, or Windows.
- Transport tested: USB or serial.
- Chip tested, if hardware was available.
- Whether the change touched config registers or destructive operations.

Destructive operations must stay explicit. Do not make config write, erase, or
flash happen as a side effect of probing.
