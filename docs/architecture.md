# Architecture

## Goals

The crate separates two concerns:

1. text progression and reveal state
2. deterministic effect evaluation for visible glyphs

That split keeps playback control generic while letting the effect stack stay reusable across UI labels, world-space prompts, menu headlines, captions, score popups, and accessibility overlays.

## Reveal Unit

Reveal progression is grapheme-based by default.

- `RevealMode::Grapheme` reveals one Unicode grapheme cluster at a time.
- `RevealMode::Word` groups contiguous non-whitespace graphemes.
- `RevealMode::Line` groups explicit newline-delimited spans.
- `RevealMode::Instant` reveals the full block immediately.

The crate uses `unicode-segmentation` for grapheme boundaries. It does not rely on byte slicing or raw `char` counts for reveal progression.

## Text Representation

The crate does not rewrite the original Bevy text hierarchy into one entity per grapheme. Instead it uses a hybrid model:

1. the original `Text` or `Text2d` block remains the layout authority
2. the crate reads `ComputedTextBlock` and `TextLayoutInfo` after Bevy updates the layout
3. the crate hides the source glyph colors and shadows
4. the crate renders a crate-owned per-glyph overlay using Bevy’s atlas-backed glyph layout

This keeps wrapping stable during typewriter reveal because the full text block stays laid out from the start.

## Pipeline

The runtime stages are:

1. `DetectChanges`
2. `Advance`
3. `EvaluateEffects`
4. `ApplyOutput`

Detailed flow:

1. Detect dirty config or source-text changes.
2. Preprocess optional `TextAnimationMarkup` spans into clean text sections plus generated `TextEffect` ranges.
3. Rebuild grapheme and reveal-unit caches only when needed, including layout-driven rebuilds when `TextLayoutInfo` changes.
4. Advance playback time according to scaled or unscaled time.
5. Resolve visible reveal units, reduced-motion state, and effect-local time in `EvaluateEffects`.
6. Apply final positions, colors, alpha, and scale to overlay glyph entities in `ApplyOutput` once Bevy has published the latest text layout for the frame.
7. Emit start, completion, loop, checkpoint, reveal-advanced, and optional sound-request messages.

`ApplyOutput` runs in `PostUpdate` after Bevy’s UI and `Text2d` layout systems.

`TextRevealAdvanced` and `TextRevealSoundRequested` deliberately stay as message surfaces only. The crate reports reveal cadence and labels, but host apps own the actual audio, portrait, subtitle, or dialogue-state reactions.

## Effect Composition

Effect composition is ordered and explicit.

- Wave and shake add positional offsets.
- Rainbow blends toward its computed color using effect strength and envelope intensity.
- Alpha pulse multiplies visibility.
- Scale multiplies per-glyph render scale.

The public effect list is declared as `Vec<TextEffect>` so consumers can choose and document the ordering they rely on.

Markup-generated effects are appended to the authored effect list at runtime. That lets dialogue authors use tagged text for local spans while still applying crate-wide baseline effects from `TextAnimationConfig`.

## Wrapping Strategy

Typewriter reveal avoids the usual line-jump problem by preserving Bevy’s full layout from the start.

- The source text remains complete.
- The crate reads the final laid-out glyph positions from `TextLayoutInfo`.
- Visibility is applied to the overlay glyphs instead of shrinking the source text string every frame.

Tradeoff:

- wrapping stays stable
- shaped glyph runs are still constrained by what Bevy exposes in `TextLayoutInfo`

## Determinism

The runtime is deterministic when:

- source text, config, and effect order are unchanged
- the same time source is used
- the same shake seed is used

Shake is seedable and uses a hashed, interpolated pseudo-noise function instead of per-frame nondeterministic randomness.

## Source Text Changes Mid-Animation

If the text or animation config changes:

- the runtime marks the block dirty
- grapheme and reveal-unit caches rebuild on the next output pass
- overlay glyph data retries until Bevy has published atlas-backed glyph layout that can be mapped
- playback continues from the controller’s current elapsed time unless the consumer explicitly restarts it

This keeps the public control model simple, but consumers should restart playback when a new text payload semantically replaces the old one.

## Current Limitations

- live mutation of hidden source style data while animation is active is intentionally conservative
- the crate follows Bevy layout changes through `TextLayoutInfo`, but custom rendering flows that bypass that component may still need an explicit refresh
- per-glyph rotation is not yet part of the public effect surface
- complex ligatures can still reveal as a shaped unit because Bevy exposes rendered glyphs, not a lossless grapheme-to-glyph correspondence
