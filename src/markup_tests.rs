use crate::{TextEffect, config::TextRangeSelector, markup::parse_sections};

#[test]
fn inline_markup_strips_tags_and_generates_effect_ranges() {
    let parsed = parse_sections(&[String::from("Hello <wave>pilot</wave> <scale>now</scale>.")]);

    assert_eq!(parsed.sections, vec![String::from("Hello pilot now.")]);
    assert_eq!(parsed.effects.len(), 2);

    match &parsed.effects[0] {
        TextEffect::Wave(effect) => {
            assert_eq!(
                effect.range,
                TextRangeSelector::GraphemeRange { start: 6, end: 11 }
            );
        }
        other => panic!("expected wave effect, got {other:?}"),
    }

    match &parsed.effects[1] {
        TextEffect::Scale(effect) => {
            assert_eq!(
                effect.range,
                TextRangeSelector::GraphemeRange { start: 12, end: 15 }
            );
        }
        other => panic!("expected scale effect, got {other:?}"),
    }
}

#[test]
fn malformed_tags_are_left_in_the_clean_text() {
    let parsed = parse_sections(&[String::from(
        "Signal <unknown>unsafe</unknown> <wave>ok</wave>",
    )]);

    assert_eq!(
        parsed.sections,
        vec![String::from("Signal <unknown>unsafe</unknown> ok")]
    );
    assert_eq!(parsed.effects.len(), 1);
}
