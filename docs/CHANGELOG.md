# Change log

This file records completed changes. Current behavior is defined by the documents in this
directory and the terminal contract selected by `Cargo.toml`.

## 2026-08-28

- The sidecar now reads cursor shape and blink state from Alacritty's parsed terminal state.
- Warm rehydrate preserves DECSCUSR state, and DECTCEM visibility remains independent.
- The common renderer receives the provider's 750 ms cursor animation policy.
- Contract cursor acceptance and the full arm64 owner verification passed.
