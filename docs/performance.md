# Performance

## Hot Paths

Steady-state work is concentrated in:

- reveal timing lookup from cached unit timings
- per-glyph effect evaluation
- overlay transform and color writes

The crate avoids rebuilding the source text or respawning the full overlay every frame.

## Dirty Rebuild Triggers

Cache rebuild happens when:

- `TextAnimationConfig` changes
- the source text changes
- the runtime has shaped graphemes but has not yet successfully mapped any render glyphs from Bevy's atlas data

Rebuild work includes:

- grapheme segmentation
- reveal-unit construction
- glyph-to-grapheme mapping from `TextLayoutInfo`
- overlay child respawn sized to the current rendered glyph count

## Steady-State Behavior

When text and layout are stable:

- no grapheme cache rebuild happens
- no source hierarchy rebuild happens
- the crate reuses cached reveal-unit timings
- glyph output updates are limited to overlay transforms, colors, and alpha

## Scaling Advice

- Prefer a smaller number of long-lived looping decorative labels over hundreds of constantly reconfigured labels.
- Use `TypewriterConfig { enabled: false, .. }` for always-visible decorative text instead of rebuilding or restarting reveal unnecessarily.
- Keep heavy shake amplitudes and very high glyph counts for short bursts rather than permanent HUD decoration.
- If many labels are purely decorative, prefer wave or alpha pulse over shake.

## Stress Example

Use the crate’s stress example to spot churn and count active labels:

```bash
cargo run -p saddle-animation-text-animation-example-stress
```

The crate-local lab also includes a stress showcase scenario:

```bash
cargo run -p saddle-animation-text-animation-lab --features e2e -- stress_showcase
```

## Current Churn Risks

The current implementation still has a few intentional tradeoffs:

- overlay child entities are rebuilt when layout changes instead of incrementally patched
- the overlay still relies on Bevy's normal `TextLayoutInfo` change path instead of trying to observe every possible external driver directly
- source style hiding is conservative and currently optimized for stable text content rather than frequent live style edits
- large-scale `Text2d` swarms will still pay per-glyph effect evaluation and per-entity transform writes
