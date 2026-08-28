# Terminal presentation

The terminal engine owns parsed cursor state. This sidecar reads
`alacritty_terminal::Term::cursor_style()` after the engine consumes output. It does not parse CSI
or OSC a second time in an adapter.

The public cursor state has two independent parts:

- `TerminalCursorStyle` carries the engine-selected block, underline, or bar shape and its blink
  mode. DECSCUSR changes this state.
- `TerminalModes.show_cursor` carries DECTCEM visibility. Hiding the cursor does not erase its
  selected shape or blink mode.

Alacritty's `Beam` maps to the contract's `Bar`. The engine's hollow and hidden presentation
variants are represented as the block shape because hollow rendering is a focus presentation and
DECTCEM visibility is reported separately. The current sidecar config does not select either
variant as terminal state.

Blink scheduling is renderer policy, not parsed terminal state. This provider declares a 750 ms
animation interval. The common terminal Kit schedules frames only while the engine reports a
visible blinking cursor; steady and hidden cursors wait for events without a timer.

Alacritty's `Term::colors()` is the sole source for OSC 4/10/11/12 overrides. The adapter maps its
optional indexed, foreground, background and cursor RGB values to `TerminalThemeOverrides`; it
does not parse OSC. An absent value is a reset and lets the common renderer use the current host
base theme. The common Kit applies the overrides, repaints every affected row and publishes the
applied base, override and effective theme state.

The contract-owned DECSCUSR and DEC private mode cases run in this repository through
`tests/conformance.rs`. `make verify TARGET=aarch64-apple-darwin` verifies this provider's mapping
and artifact. It does not inspect another provider implementation.
