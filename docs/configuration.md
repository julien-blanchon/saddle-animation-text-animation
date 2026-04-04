# Configuration

## `TextAnimationConfig`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `typewriter` | `TypewriterConfig` | enabled grapheme reveal at `30.0` units/sec | any valid `TypewriterConfig` | Controls reveal timing and punctuation delays |
| `effects` | `Vec<TextEffect>` | empty | any ordered list | Ordered effect stack |
| `continue_effects_after_reveal` | `bool` | `true` | `true` or `false` | Keeps looping decorative effects active after reveal completes |

## `TypewriterConfig`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `enabled` | `bool` | `true` | `true` or `false` | When `false`, the full text is immediately visible |
| `reveal_mode` | `RevealMode` | `Grapheme` | enum variant | Reveal grouping strategy |
| `units_per_second` | `f32` | `30.0` | `> 0.0` recommended, clamped internally to `>= 0.001` | Reveal speed |
| `punctuation_delay` | `PunctuationDelayConfig` | see below | non-negative seconds recommended | Adds extra delay after punctuation and newlines |

## `RevealMode`

| Variant | Meaning |
| --- | --- |
| `Instant` | Reveal everything immediately |
| `Grapheme` | Reveal by grapheme cluster |
| `Word` | Reveal contiguous non-whitespace words |
| `Line` | Reveal newline-delimited lines |

## `PunctuationDelayConfig`

| Field | Type | Default | Valid Range | Impact |
| --- | --- | --- | --- | --- |
| `sentence_extra_secs` | `f32` | `0.18` | `>= 0.0` | Added after `.`, `!`, `?`, and similar marks |
| `clause_extra_secs` | `f32` | `0.07` | `>= 0.0` | Added after `,`, `;`, `:`, and similar marks |
| `ellipsis_extra_secs` | `f32` | `0.25` | `>= 0.0` | Added after `...` or `…` |
| `newline_extra_secs` | `f32` | `0.10` | `>= 0.0` | Added after newline reveal |

## `TextAnimationController`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `state` | `TextAnimationPlaybackState` | `Playing` | enum variant | Play or pause |
| `time_source` | `TextAnimationTimeSource` | `Scaled` | enum variant | Uses game time or real time |
| `elapsed_secs` | `f32` | `0.0` | any finite value, `>= 0.0` recommended | Current playback position |
| `speed_scale` | `f32` | `1.0` | `>= 0.0` recommended | Multiplies the time delta each frame |
| `repeat` | `bool` | `false` | `true` or `false` | Loops reveal playback when duration is non-zero |

## Accessibility

### `TextAnimationAccessibility`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `reduced_motion` | `bool` | `false` | `true` or `false` | Global decorative-motion suppression flag |

### `TextMotionPreference`

| Variant | Meaning |
| --- | --- |
| `Inherit` | Use the global accessibility resource |
| `Full` | Force full motion for this entity |
| `Reduced` | Force reduced motion for this entity |

## `TextAnimationMarkup`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `sections` | `Vec<String>` | empty | any section list | Source text with optional inline tags that are stripped into runtime effect ranges |

## `TextRevealSound`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `cue_id` | `String` | `"text.reveal"` | any stable identifier | User-defined sound cue or event id to route into the game audio layer |
| `every_n_units` | `usize` | `1` | `>= 1` | Emits at a lower cadence when set above `1` |
| `skip_whitespace` | `bool` | `true` | `true` or `false` | Suppresses events for whitespace-only reveal units |

## Effect Range Targeting

All effects use `TextRangeSelector`.

| Variant | Meaning |
| --- | --- |
| `All` | Entire text block |
| `GraphemeRange { start, end }` | Grapheme index range |
| `WordRange { start, end }` | Word index range |
| `LineRange { start, end }` | Line index range |
| `SectionRange { start, end }` | Text section range |

## Effect Envelopes

All current effects can use `EffectEnvelope`.

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `start_delay_secs` | `f32` | `0.0` | `>= 0.0` | Delays effect activation |
| `fade_in_secs` | `f32` | `0.0` | `>= 0.0` | Ramps the effect in |
| `end_after_secs` | `Option<f32>` | `None` | `None` or `Some(>= 0.0)` | Optional local end time |
| `fade_out_secs` | `f32` | `0.0` | `>= 0.0` | Ramps the effect out after `end_after_secs` |
| `easing` | `TextEnvelopeEasing` | `SmoothStep` | enum variant | Envelope curve |

## Wave

| Field | Type | Default | Valid Range | Impact | Reduced Motion |
| --- | --- | --- | --- | --- | --- |
| `range` | `TextRangeSelector` | `All` | any selector | Scope | unaffected |
| `amplitude` | `f32` | `6.0` | any finite value, `>= 0.0` recommended | Vertical offset in logical pixels | multiplied by `reduced_motion_scale` |
| `frequency` | `f32` | `0.38` | any finite value | Per-glyph phase step | same |
| `phase_offset` | `f32` | `0.45` | any finite value | Global phase offset | same |
| `speed` | `f32` | `3.5` | any finite value | Oscillation speed | same |
| `envelope` | `EffectEnvelope` | default | any envelope | Time-local intensity curve | same |
| `reduced_motion_scale` | `f32` | `0.0` | typically `0.0..=1.0` | Motion multiplier when reduced motion is active | primary control |

## Shake

| Field | Type | Default | Valid Range | Impact | Reduced Motion |
| --- | --- | --- | --- | --- | --- |
| `range` | `TextRangeSelector` | `All` | any selector | Scope | unaffected |
| `magnitude` | `Vec2` | `2.5, 2.5` | finite vector, non-negative recommended | Maximum offset | multiplied by `reduced_motion_scale` |
| `frequency_hz` | `f32` | `10.0` | `> 0.0` recommended | Noise sampling rate | same |
| `smoothness` | `f32` | `0.75` | `0.0..=1.0` recommended | Interpolation between noise samples | same |
| `seed` | `u64` | `7` | any `u64` | Deterministic seed | same |
| `envelope` | `EffectEnvelope` | default | any envelope | Time-local intensity curve | same |
| `reduced_motion_scale` | `f32` | `0.0` | typically `0.0..=1.0` | Motion multiplier when reduced motion is active | primary control |

## Rainbow

| Field | Type | Default | Valid Range | Impact | Reduced Motion |
| --- | --- | --- | --- | --- | --- |
| `range` | `TextRangeSelector` | `All` | any selector | Scope | unaffected |
| `hue_speed` | `f32` | `0.20` | any finite value | Hue cycle speed | unchanged |
| `hue_offset` | `f32` | `0.12` | any finite value | Per-glyph hue stride | unchanged |
| `saturation` | `f32` | `0.90` | `0.0..=1.0` recommended | HSV saturation | unchanged |
| `value` | `f32` | `1.0` | `0.0..=1.0` recommended | HSV value | unchanged |
| `strength` | `f32` | `1.0` | `0.0..=1.0` recommended | Blend strength | unchanged |
| `envelope` | `EffectEnvelope` | default | any envelope | Time-local intensity curve | unchanged |

## Alpha Pulse

| Field | Type | Default | Valid Range | Impact | Reduced Motion |
| --- | --- | --- | --- | --- | --- |
| `range` | `TextRangeSelector` | `All` | any selector | Scope | unaffected |
| `min_alpha` | `f32` | `0.70` | `0.0..=1.0` recommended | Lowest pulse alpha | unchanged |
| `max_alpha` | `f32` | `1.0` | `0.0..=1.0` recommended | Highest pulse alpha | unchanged |
| `speed` | `f32` | `2.5` | any finite value | Pulse speed | unchanged |
| `phase_offset` | `f32` | `0.15` | any finite value | Per-glyph phase stride | unchanged |
| `envelope` | `EffectEnvelope` | default | any envelope | Time-local intensity curve | unchanged |

## Scale

| Field | Type | Default | Valid Range | Impact | Reduced Motion |
| --- | --- | --- | --- | --- | --- |
| `range` | `TextRangeSelector` | `All` | any selector | Scope | unaffected |
| `min_scale` | `f32` | `0.92` | `> 0.0` recommended | Lowest scale multiplier | unchanged |
| `max_scale` | `f32` | `1.12` | `> 0.0` recommended | Highest scale multiplier | unchanged |
| `speed` | `f32` | `2.0` | any finite value | Pulse speed | unchanged |
| `phase_offset` | `f32` | `0.10` | any finite value | Per-glyph phase stride | unchanged |
| `envelope` | `EffectEnvelope` | default | any envelope | Time-local intensity curve | unchanged |

## Reveal Messages

| Message | Fields | Meaning |
| --- | --- | --- |
| `TextRevealCheckpoint` | `entity`, `revealed_units`, `total_units` | Reveal count changed this frame |
| `TextRevealAdvanced` | `entity`, `start_unit`, `end_unit`, `labels` | Newly revealed unit slice, useful for dialogue blips or subtitle hooks |
| `TextRevealSoundRequested` | `entity`, `cue_id`, `unit_index`, `label` | Optional sound hook for the just-revealed unit cadence |
