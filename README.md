# soksak-sidecar-terminal-alacritty

Alacritty-backed implementation of the installed terminal recovery sidecar contract. This owner
contains the engine adapter, direct manifest, native build, conformance tests and release target
matrix. Shared recovery, checkpoint and render-session behavior comes from the exact
`soksak-kit-sidecar-terminal` Git revision declared in `Cargo.toml`.

Native selection delegates to Alacritty's maintained `Selection` state and
`Term::selection_to_string`. The adapter converts the common gesture point to Alacritty
`Point`/`Side`, exposes Alacritty's inclusive row ranges to the shared painter, and returns the
common versioned snapshot. It does not reconstruct selected text from exported cells.

Wheel input consumes the Kit's already normalized cell position, step counts, modifiers and route.
This adapter encodes legacy, UTF-8 and SGR mouse reports from Alacritty's current terminal modes,
including wheel direction and repeated steps. Alternate-scroll emits application cursor keys. It
refuses a route whose engine mode changed and never writes the PTY itself.

Pointer input uses the same mode state and coordinate encoders. Normal tracking emits press and
release, button-event tracking adds held motion, and any-event tracking adds no-button motion.
Legacy release uses button code three while SGR release keeps the physical button and terminates
with `m`. Modifier bits and UTF-8 coordinates follow the same encoder as wheel reports.

## Verification

```sh
make lock TARGET=aarch64-apple-darwin
make verify TARGET=aarch64-apple-darwin
make stage TARGET=aarch64-apple-darwin STAGE=dist
make attest TARGET=aarch64-apple-darwin OUT=/absolute/alacritty-release
```

`make lock` is the only owner operation that projects changed Cargo declarations into
`Cargo.lock`. Normal build and verification remain `--locked`. Every target is explicit; the
repository does not discover a sibling Kit checkout or substitute another architecture.
