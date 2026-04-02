# Debugging

## Common Failure Modes

### Text never appears

Check:

- the source entity has `Text` or `Text2d`
- it also has `TextFont`
- the plugin is registered
- the source text exists in the active schedule

For `Text2d`, also check:

- a `Camera2d` exists
- the text is in front of the camera

### Reveal never advances

Check:

- `TextAnimationController.state == Playing`
- `speed_scale > 0.0`
- the chosen time source is the one you expect
- `TextAnimationDebugState.total_units > 0`

### Completion never fires

Check:

- the text actually has reveal units
- the controller elapsed time reaches the cached duration
- the consumer does not restart the controller before completion

### Decorative motion does not move

Check:

- `TextAnimationAccessibility.reduced_motion`
- per-entity `TextMotionPreference`
- `reduced_motion_scale` on wave and shake

## Runtime Inspection

The most useful components and resources are:

- `saddle_animation_text_animation::TextAnimationDebugState`
- `saddle_animation_text_animation::TextAnimationController`
- `saddle_animation_text_animation::TextAnimationConfig`
- `saddle_animation_text_animation::TextAnimationAccessibility`

The crate-local lab adds:

- `saddle_animation_text_animation_lab::LabDiagnostics`

## BRP

The crate-local lab enables BRP in its default `dev` feature set.

```bash
BRP_EXTRAS_PORT=15742 cargo run -p saddle-animation-text-animation-lab
BRP_PORT=15742 uv run --active --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
```

Useful BRP checks:

```bash
BRP_PORT=15742 uv run --active --project .codex/skills/bevy-brp/script brp world query saddle_animation_text_animation::TextAnimationDebugState
BRP_PORT=15742 uv run --active --project .codex/skills/bevy-brp/script brp world query saddle_animation_text_animation::TextAnimationAccessibility
BRP_PORT=15742 uv run --active --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/saddle_animation_text_animation_lab.png
```

## E2E

Use the crate-local lab for screenshot-driven checks:

```bash
cargo run -p saddle-animation-text-animation-lab --features e2e -- smoke_launch
cargo run -p saddle-animation-text-animation-lab --features e2e -- typewriter_showcase
```

Outputs land in `e2e_output/<scenario>/`.

The E2E scenarios assert runtime conditions before capturing screenshots. They are meant to verify both visible animation states and the corresponding debug facts.
