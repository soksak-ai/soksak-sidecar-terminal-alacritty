# soksak-sidecar-terminal-alacritty

Alacritty-backed implementation of the installed terminal recovery sidecar contract. This owner
contains the engine adapter, direct manifest, native build, conformance tests and release target
matrix. Shared recovery, checkpoint and render-session behavior comes from the exact
`soksak-kit-sidecar-terminal` Git revision declared in `Cargo.toml`.

## Verification

```sh
make lock TARGET=aarch64-apple-darwin
make verify TARGET=aarch64-apple-darwin
```

`make lock` is the only owner operation that projects changed Cargo declarations into
`Cargo.lock`. Normal build and verification remain `--locked`. Every target is explicit; the
repository does not discover a sibling Kit checkout or substitute another architecture.
