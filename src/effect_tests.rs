use bevy::math::Vec2;
use bevy::prelude::*;

use crate::config::{EffectEnvelope, RainbowEffect, ShakeEffect, TextEffect, WaveEffect};
use crate::effect::{GlyphVisual, apply_effects, envelope_factor, matches_range};
use crate::glyph_cache::{GlyphEntry, GraphemeEntry};

fn sample_glyph() -> GlyphEntry {
    GlyphEntry {
        primary_index: 3,
        grapheme_indices: vec![3],
        section_index: 0,
        texture: bevy::asset::AssetId::default(),
        rect: bevy::math::Rect::new(0.0, 0.0, 16.0, 16.0),
        center: Vec2::new(8.0, 8.0),
        size: Vec2::splat(16.0),
    }
}

fn sample_grapheme() -> GraphemeEntry {
    GraphemeEntry {
        index: 3,
        section_index: 0,
        section_entity: Entity::from_bits(1),
        text: "A".to_string(),
        byte_range: 0..1,
        is_whitespace: false,
        line_index: 0,
        word_index: Some(1),
        unit_index: 3,
    }
}

#[test]
fn envelope_reaches_zero_before_start_and_one_after_fade_in() {
    let envelope = EffectEnvelope {
        start_delay_secs: 0.5,
        fade_in_secs: 0.5,
        ..EffectEnvelope::default()
    };
    assert_eq!(envelope_factor(&envelope, 0.0), 0.0);
    assert!(envelope_factor(&envelope, 1.2) > 0.99);
}

#[test]
fn shake_is_deterministic_for_same_seed() {
    let glyph = sample_glyph();
    let grapheme = sample_grapheme();
    let effect = TextEffect::Shake(ShakeEffect::default());
    let mut first = GlyphVisual::new(Color::WHITE);
    let mut second = GlyphVisual::new(Color::WHITE);
    apply_effects(
        &mut first,
        &[effect.clone()],
        &glyph,
        &grapheme,
        0.75,
        false,
    );
    apply_effects(&mut second, &[effect], &glyph, &grapheme, 0.75, false);
    assert_eq!(first.offset, second.offset);
}

#[test]
fn layered_effects_accumulate_translation_and_color() {
    let glyph = sample_glyph();
    let grapheme = sample_grapheme();
    let mut visual = GlyphVisual::new(Color::WHITE);
    apply_effects(
        &mut visual,
        &[
            TextEffect::Wave(WaveEffect::default()),
            TextEffect::Rainbow(RainbowEffect::default()),
        ],
        &glyph,
        &grapheme,
        1.0,
        false,
    );
    assert!(visual.offset.length() > 0.0);
    assert_ne!(visual.color, bevy::color::LinearRgba::WHITE);
}

#[test]
fn wave_frequency_changes_the_resulting_offset() {
    let mut low_frequency = GlyphVisual::new(Color::WHITE);
    let mut high_frequency = GlyphVisual::new(Color::WHITE);
    let glyph = sample_glyph();
    let grapheme = sample_grapheme();

    apply_effects(
        &mut low_frequency,
        &[TextEffect::Wave(WaveEffect {
            frequency: 0.1,
            phase_offset: 0.0,
            ..WaveEffect::default()
        })],
        &glyph,
        &grapheme,
        0.35,
        false,
    );
    apply_effects(
        &mut high_frequency,
        &[TextEffect::Wave(WaveEffect {
            frequency: 1.2,
            phase_offset: 0.0,
            ..WaveEffect::default()
        })],
        &glyph,
        &grapheme,
        0.35,
        false,
    );

    assert_ne!(low_frequency.offset.y, high_frequency.offset.y);
}

#[test]
fn range_matching_respects_word_indices() {
    let grapheme = sample_grapheme();
    assert!(matches_range(
        crate::config::TextRangeSelector::WordRange { start: 1, end: 2 },
        &grapheme
    ));
    assert!(!matches_range(
        crate::config::TextRangeSelector::WordRange { start: 2, end: 3 },
        &grapheme
    ));
}

#[test]
fn reduced_motion_suppresses_positional_effects_when_scale_is_zero() {
    let glyph = sample_glyph();
    let grapheme = sample_grapheme();
    let mut visual = GlyphVisual::new(Color::WHITE);

    apply_effects(
        &mut visual,
        &[
            TextEffect::Wave(WaveEffect {
                reduced_motion_scale: 0.0,
                ..WaveEffect::default()
            }),
            TextEffect::Shake(ShakeEffect {
                reduced_motion_scale: 0.0,
                ..ShakeEffect::default()
            }),
        ],
        &glyph,
        &grapheme,
        0.8,
        true,
    );

    assert_eq!(visual.offset, Vec2::ZERO);
}
