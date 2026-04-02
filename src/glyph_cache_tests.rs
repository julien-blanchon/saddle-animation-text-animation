use crate::config::{RevealMode, TextAnimationConfig, TypewriterConfig};
use crate::glyph_cache::{SectionSnapshot, build_cache};
use bevy::image::TextureAtlasLayout;
use bevy::prelude::*;
use bevy::text::{ComputedTextBlock, TextFont, TextLayoutInfo};

fn section(text: &str) -> SectionSnapshot {
    SectionSnapshot {
        entity: Entity::from_bits(1),
        text: text.to_string(),
        font: TextFont::default(),
        base_color: Color::WHITE,
    }
}

#[test]
fn grapheme_segmentation_preserves_combining_marks() {
    let config = TextAnimationConfig::default();
    let texture_atlases = Assets::<TextureAtlasLayout>::default();
    let cache = build_cache(
        &config,
        vec![section("e\u{301}👨‍👩‍👧‍👦")],
        &ComputedTextBlock::default(),
        &TextLayoutInfo::default(),
        &texture_atlases,
    );

    let texts: Vec<_> = cache
        .graphemes
        .iter()
        .map(|grapheme| grapheme.text.as_str())
        .collect();
    assert_eq!(texts, vec!["e\u{301}", "👨‍👩‍👧‍👦"]);
}

#[test]
fn word_mode_groups_contiguous_non_whitespace_text() {
    let config = TextAnimationConfig {
        typewriter: TypewriterConfig {
            reveal_mode: RevealMode::Word,
            ..TypewriterConfig::default()
        },
        ..TextAnimationConfig::default()
    };
    let texture_atlases = Assets::<TextureAtlasLayout>::default();
    let cache = build_cache(
        &config,
        vec![section("Hello, world!")],
        &ComputedTextBlock::default(),
        &TextLayoutInfo::default(),
        &texture_atlases,
    );

    assert_eq!(cache.units.len(), 2);
    assert_eq!(cache.units[0].label, "Hello,");
    assert_eq!(cache.units[1].label, "world!");
}

#[test]
fn line_mode_uses_explicit_newlines() {
    let config = TextAnimationConfig {
        typewriter: TypewriterConfig {
            reveal_mode: RevealMode::Line,
            ..TypewriterConfig::default()
        },
        ..TextAnimationConfig::default()
    };
    let texture_atlases = Assets::<TextureAtlasLayout>::default();
    let cache = build_cache(
        &config,
        vec![section("alpha\nbeta\r\ngamma")],
        &ComputedTextBlock::default(),
        &TextLayoutInfo::default(),
        &texture_atlases,
    );

    assert_eq!(cache.units.len(), 3);
}

#[test]
fn reveal_timing_respects_units_per_second() {
    let config = TextAnimationConfig {
        typewriter: TypewriterConfig {
            units_per_second: 4.0,
            punctuation_delay: Default::default(),
            ..TypewriterConfig::default()
        },
        ..TextAnimationConfig::default()
    };
    let texture_atlases = Assets::<TextureAtlasLayout>::default();
    let cache = build_cache(
        &config,
        vec![section("abc")],
        &ComputedTextBlock::default(),
        &TextLayoutInfo::default(),
        &texture_atlases,
    );

    let reveal_times: Vec<_> = cache
        .units
        .iter()
        .map(|unit| unit.reveal_time_secs)
        .collect();
    assert_eq!(reveal_times, vec![0.0, 0.25, 0.5]);
}

#[test]
fn punctuation_delays_extend_reveal_timing() {
    let config = TextAnimationConfig {
        typewriter: TypewriterConfig {
            units_per_second: 2.0,
            punctuation_delay: crate::config::PunctuationDelayConfig {
                sentence_extra_secs: 0.3,
                clause_extra_secs: 0.0,
                ellipsis_extra_secs: 0.0,
                newline_extra_secs: 0.4,
            },
            ..TypewriterConfig::default()
        },
        ..TextAnimationConfig::default()
    };
    let texture_atlases = Assets::<TextureAtlasLayout>::default();
    let cache = build_cache(
        &config,
        vec![section("A!\nB")],
        &ComputedTextBlock::default(),
        &TextLayoutInfo::default(),
        &texture_atlases,
    );

    let reveal_times: Vec<_> = cache
        .units
        .iter()
        .map(|unit| unit.reveal_time_secs)
        .collect();
    let expected = [0.0, 0.5, 1.3, 2.2];
    for (actual, expected) in reveal_times.iter().zip(expected) {
        assert!((actual - expected).abs() < 0.0001);
    }
}
