# Change log

This file records completed changes. Current behavior is defined by the documents in this
directory and the terminal contract selected by `Cargo.toml`.

## 2026-08-31

- Release 0.0.44 uses the exact SDK 0.0.20 release closure for both local and public owner proofs.
- Release 0.0.43 assigns the rewritten source commit a new immutable release identity; 0.0.42
  remains bound to its original source commit and bytes.

## 2026-08-30

- Release 0.0.42 publishes the expanded mouse-tracking mode surface.
- DEC modes 9 and 1001 remain false because Alacritty does not retain either mode; no other
  tracking state is used as an alias.
- Wheel and pointer admission now follows the public `TerminalModes` rules, with owner tests for
  the unsupported legacy modes and the distinct supported click mode.

## 2026-08-28

- The sidecar now reads cursor shape and blink state from Alacritty's parsed terminal state.
- Warm rehydrate preserves DECSCUSR state, and DECTCEM visibility remains independent.
- The common renderer receives the provider's 750 ms cursor animation policy.
- Contract cursor acceptance and the full arm64 owner verification passed.
