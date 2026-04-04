# Saddle Animation Text Animation

Animated text effects for Bevy UI and world-space text. The crate targets built-in `Text` and `Text2d`, keeps layout stable during reveal by preserving the full Bevy text layout, and layers deterministic per-glyph visuals on top.

The runtime is generic. It does not know about dialogue systems, quest logic, subtitles, or any project state machine. Consumers own when text changes, when playback is restarted or skipped, and what completion means in their game.

## Quick Start

```toml
[dependencies]
bevy = "0.18"
saddle-animation-text-animation = { git = "https://github.com/julien-blanchon/saddle-animation-text-animation" }
```

```rust,no_run
use bevy::prelude::*;
use saddle_animation_text_animation::{
    TextAnimationBundle, TextAnimationConfig, TextAnimationMarkup, TextAnimationPlugin,
    TextEffect, WaveEffect,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TextAnimationPlugin::always_on(Update))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2d));

    commands.spawn((
        Name::new("World Label"),
        Text2d::new(""),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextAnimationMarkup::single("<wave>Signal uplink</wave> stable"),
        TextAnimationBundle {
            config: TextAnimationConfig::typewriter(20.0)
                .with_effect(TextEffect::Wave(WaveEffect::default())),
            ..default()
        },
        Transform::from_xyz(0.0, 24.0, 0.0),
    ));
}
```

## Supported Text Roots

- `Text` in Bevy UI
- `Text2d` in world space

## Plugin Usage

`TextAnimationPlugin` keeps injectable schedules:

```rust
use bevy::prelude::*;
use saddle_animation_text_animation::TextAnimationPlugin;

app.add_plugins(TextAnimationPlugin::new(
    OnEnter(MyState::ShowingText),
    OnExit(MyState::ShowingText),
    Update,
));
```

For always-on usage in examples, tests, and crate-local labs:

```rust
app.add_plugins(TextAnimationPlugin::always_on(Update));
```

## Public API

| Item | Purpose |
| --- | --- |
| `TextAnimationPlugin` | Registers the runtime with injectable activate, deactivate, and update schedules |
| `TextAnimationSystems` | Public ordering hooks: `DetectChanges`, `Advance`, `EvaluateEffects`, `ApplyOutput` |
| `TextAnimationConfig` | Source animation config: typewriter settings, effect stack, and effect continuation |
| `TextAnimationController` | Playback state, elapsed time, repeat, and time-source control |
| `TextAnimationBundle` | Convenience bundle for config, controller, motion preference, and debug state |
| `TextMotionPreference` | Per-entity motion override: inherit, full motion, or reduced motion |
| `TextAnimationAccessibility` | Global reduced-motion resource |
| `TypewriterConfig`, `RevealMode`, `PunctuationDelayConfig` | Reveal settings |
| `WaveEffect`, `ShakeEffect`, `RainbowEffect`, `AlphaPulseEffect`, `ScaleEffect`, `TextEffect` | Effect config types |
| `TextAnimationMarkup` | Inline-tag source that strips markup into effect ranges at runtime |
| `TextRevealSound` | Optional per-unit sound trigger config for dialogue blips or subtitle ticks |
| `TextAnimationCommand`, `TextAnimationAction` | Playback commands sent as Bevy messages |
| `TextAnimationStarted`, `TextAnimationCompleted`, `TextAnimationLoopFinished`, `TextRevealCheckpoint`, `TextRevealAdvanced`, `TextRevealSoundRequested` | Runtime marker messages |
| `TextAnimationDebugState` | Per-entity diagnostics for reveal count, glyph count, elapsed time, and effect count |

## Effects

- Typewriter reveal with grapheme, word, line, or instant reveal units
- Punctuation-aware reveal delays
- Wave with per-glyph phase offset
- Deterministic seedable shake
- Rainbow color cycling
- Alpha pulse for non-positional emphasis
- Scale pulse for dialogue emphasis without layout rebuilds
- Ordered effect composition on the same text block
- Range targeting by grapheme, word, line, section, or the full text
- Inline markup tags: `<wave>`, `<shake>`, `<rainbow>`, `<alpha>` / `<pulse>`, and `<scale>`
- Optional per-unit sound requests for dialogue blips or subtitle ticks

Effects compose in declaration order. Positional offsets add together. Rainbow blends on top of the current color, and alpha pulse multiplies the current visibility.

## Inline Markup

Attach `TextAnimationMarkup` when authored dialogue is easier to read as tagged text than as manual grapheme ranges:

```rust
TextAnimationMarkup::single(
    "Commander: <wave>steady</wave> approach, then <shake>burn hard</shake>."
)
```

The runtime strips the tags from the visible text, converts each tag span into the matching `TextEffect`, emits `TextRevealAdvanced` with the newly revealed unit labels, and can also emit `TextRevealSoundRequested` when a `TextRevealSound` component is attached.

## Reduced Motion

Decorative positional motion is suppressible through:

- global `TextAnimationAccessibility { reduced_motion: true }`
- per-entity `TextMotionPreference`
- per-effect reduced-motion amplitude scaling on wave and shake

Reveal, color, and alpha-only emphasis can stay active while positional motion is reduced or removed.

## Design Summary

The crate reveals text at the grapheme level for user-facing progression, but it does not replace Bevy's text shaping pipeline. It preserves the fully laid-out Bevy text block, hides the source glyph colors, and renders a crate-owned glyph overlay from Bevy's computed text layout. That keeps wrapping stable during reveal and lets multiple effects operate on the same text block without respawning the original text tree every frame.

Internally the runtime resolves playback state in `DetectChanges`, `Advance`, and `EvaluateEffects`, then applies glyph positions and colors in `ApplyOutput` after Bevy has produced the latest UI or `Text2d` layout for the frame.

## Examples

| Example | Run | What it demonstrates |
| --- | --- | --- |
| `basic` | `cargo run -p saddle-animation-text-animation-example-basic` | Minimal UI plus `Text2d` usage |
| `dialogue_box` | `cargo run -p saddle-animation-text-animation-example-dialogue-box` | Dialogue UI with inline markup and per-unit voice-blip hooks |
| `typewriter` | `cargo run -p saddle-animation-text-animation-example-typewriter` | Reveal, punctuation delay, pause/resume, finish-now, and restart |
| `layered_effects` | `cargo run -p saddle-animation-text-animation-example-layered-effects` | Explicit effect ordering and range-targeted emphasis |
| `reduced_motion` | `cargo run -p saddle-animation-text-animation-example-reduced-motion` | Full-motion and reduced-motion variants side by side |
| `text2d_world_label` | `cargo run -p saddle-animation-text-animation-example-text2d-world-label` | World-space labels over simple scene markers |
| `stress` | `cargo run -p saddle-animation-text-animation-example-stress` | Many concurrent labels and steady-state churn check |
| `saddle-animation-text-animation-lab` | `cargo run -p saddle-animation-text-animation-lab` | Crate-local BRP and E2E verification app |

## Crate-Local Lab

The richer verification app lives under [`examples/lab/README.md`](/Users/julienblanchon/Git/bevy_starter/shared/animation/saddle-animation-text-animation/examples/lab/README.md).

Targeted E2E scenarios:

```bash
cargo run -p saddle-animation-text-animation-lab --features e2e -- smoke_launch
cargo run -p saddle-animation-text-animation-lab --features e2e -- typewriter_showcase
cargo run -p saddle-animation-text-animation-lab --features e2e -- layered_effects_showcase
cargo run -p saddle-animation-text-animation-lab --features e2e -- reduced_motion_showcase
cargo run -p saddle-animation-text-animation-lab --features e2e -- unicode_showcase
cargo run -p saddle-animation-text-animation-lab --features e2e -- stress_showcase
```

## Known Limitations

- Reveal progression is grapheme-aware, but final rendering still follows Bevy text shaping. Ligatures and some shaped runs may appear or disappear as a unit because Bevy's `TextLayoutInfo` exposes rendered glyphs rather than a perfect grapheme-to-glyph mapping.
- Bidirectional text, fallback fonts, and advanced shaping are delegated to Bevy. The crate preserves their layout, but it does not implement its own shaper.
- While animation is active, layout stability comes from hiding the source text and drawing an overlay. This keeps wrapping stable, but source decorations and shadows are intentionally suppressed and re-applied only when animation is removed or deactivated.
- The runtime rebuilds overlay caches when source text, animation config, or Bevy text layout changes. Extremely custom text pipelines may still need an explicit refresh if they mutate layout-driving state outside Bevy's normal `TextLayoutInfo` change path.
- Per-glyph scale, rotation, caret rendering, and palette-driven color cycling are intentionally left for a future version so the crate can keep a small stable API around Bevy's current text layout hooks.

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
- [Performance](docs/performance.md)
- [Debugging](docs/debugging.md)
