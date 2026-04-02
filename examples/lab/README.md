# saddle-animation-text-animation-lab

Crate-local lab for the `saddle-animation-text-animation` shared crate.

## Run

```bash
cargo run -p saddle-animation-text-animation-lab
```

The lab renders:

- a punctuation-aware UI typewriter block
- a long-lived decorative headline with layered effects
- a reduced-motion comparison sample
- a multilingual Unicode sample
- a world-space warning label
- a stress field of many labels active at once

## BRP

The default `dev` feature enables BRP extras.

```bash
BRP_EXTRAS_PORT=15742 cargo run -p saddle-animation-text-animation-lab
BRP_PORT=15742 uv run --active --project .codex/skills/bevy-brp/script brp world query saddle_animation_text_animation::TextAnimationDebugState
```

Useful inspection targets:

- `saddle_animation_text_animation::TextAnimationDebugState`
- `saddle_animation_text_animation::TextAnimationController`
- `saddle_animation_text_animation::TextAnimationAccessibility`
- `saddle_animation_text_animation_lab::LabDiagnostics`

## E2E

```bash
cargo run -p saddle-animation-text-animation-lab --features e2e -- smoke_launch
```

Available scenarios:

- `smoke_launch`
- `typewriter_showcase`
- `layered_effects_showcase`
- `reduced_motion_showcase`
- `unicode_showcase`
- `stress_showcase`

Scenario outputs are written to `e2e_output/<scenario>/` with named screenshots and a scenario log.
