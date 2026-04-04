use bevy::color::LinearRgba;
use bevy::math::Vec2;
use bevy::prelude::*;

use crate::config::{
    AlphaPulseEffect, EffectEnvelope, RainbowEffect, ScaleEffect, ShakeEffect, TextEffect,
    TextEnvelopeEasing, TextRangeSelector, WaveEffect,
};
use crate::glyph_cache::{GlyphEntry, GraphemeEntry};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GlyphVisual {
    pub offset: Vec2,
    pub color: LinearRgba,
    pub alpha: f32,
    pub scale: Vec2,
}

impl GlyphVisual {
    pub fn new(base_color: Color) -> Self {
        Self {
            offset: Vec2::ZERO,
            color: LinearRgba::from(base_color),
            alpha: 1.0,
            scale: Vec2::ONE,
        }
    }
}

pub(crate) fn apply_effects<'a>(
    visual: &mut GlyphVisual,
    effects: impl IntoIterator<Item = &'a TextEffect>,
    glyph: &GlyphEntry,
    grapheme: &GraphemeEntry,
    elapsed_secs: f32,
    reduced_motion: bool,
) {
    for effect in effects {
        match effect {
            TextEffect::Wave(config) => {
                if !matches_range(config.range, grapheme) {
                    continue;
                }
                let intensity = envelope_factor(&config.envelope, elapsed_secs);
                if intensity <= 0.0 {
                    continue;
                }
                let motion_scale = if reduced_motion {
                    config.reduced_motion_scale
                } else {
                    1.0
                };
                if motion_scale <= 0.0 {
                    continue;
                }
                visual.offset.y +=
                    wave_offset(config, glyph, elapsed_secs) * intensity * motion_scale;
            }
            TextEffect::Shake(config) => {
                if !matches_range(config.range, grapheme) {
                    continue;
                }
                let intensity = envelope_factor(&config.envelope, elapsed_secs);
                if intensity <= 0.0 {
                    continue;
                }
                let motion_scale = if reduced_motion {
                    config.reduced_motion_scale
                } else {
                    1.0
                };
                if motion_scale <= 0.0 {
                    continue;
                }
                visual.offset +=
                    shake_offset(config, glyph, elapsed_secs) * intensity * motion_scale;
            }
            TextEffect::Rainbow(config) => {
                if !matches_range(config.range, grapheme) {
                    continue;
                }
                let intensity = envelope_factor(&config.envelope, elapsed_secs) * config.strength;
                if intensity <= 0.0 {
                    continue;
                }
                let rainbow = hsv_color(config, glyph, elapsed_secs);
                visual.color = lerp_rgba(visual.color, LinearRgba::from(rainbow), intensity);
            }
            TextEffect::AlphaPulse(config) => {
                if !matches_range(config.range, grapheme) {
                    continue;
                }
                let intensity = envelope_factor(&config.envelope, elapsed_secs);
                if intensity <= 0.0 {
                    continue;
                }
                let pulse = alpha_pulse(config, glyph, elapsed_secs);
                visual.alpha *= pulse.powf(intensity.max(0.001));
            }
            TextEffect::Scale(config) => {
                if !matches_range(config.range, grapheme) {
                    continue;
                }
                let intensity = envelope_factor(&config.envelope, elapsed_secs);
                if intensity <= 0.0 {
                    continue;
                }
                let scale = scale_pulse(config, glyph, elapsed_secs);
                let blended_scale = 1.0 + (scale - 1.0) * intensity;
                visual.scale *= Vec2::splat(blended_scale.max(0.001));
            }
        }
    }
}

pub(crate) fn matches_range(range: TextRangeSelector, grapheme: &GraphemeEntry) -> bool {
    match range {
        TextRangeSelector::All => true,
        TextRangeSelector::GraphemeRange { start, end } => {
            grapheme.index >= start && grapheme.index < end
        }
        TextRangeSelector::WordRange { start, end } => grapheme
            .word_index
            .is_some_and(|word_index| word_index >= start && word_index < end),
        TextRangeSelector::LineRange { start, end } => {
            grapheme.line_index >= start && grapheme.line_index < end
        }
        TextRangeSelector::SectionRange { start, end } => {
            grapheme.section_index >= start && grapheme.section_index < end
        }
    }
}

pub(crate) fn envelope_factor(envelope: &EffectEnvelope, elapsed_secs: f32) -> f32 {
    let local = elapsed_secs - envelope.start_delay_secs;
    if local < 0.0 {
        return 0.0;
    }

    let fade_in = if envelope.fade_in_secs > 0.0 {
        ease(local / envelope.fade_in_secs, envelope.easing)
    } else {
        1.0
    };

    let fade_out = if let Some(end_after_secs) = envelope.end_after_secs {
        if local >= end_after_secs {
            if envelope.fade_out_secs > 0.0 {
                let progress = ((local - end_after_secs) / envelope.fade_out_secs).clamp(0.0, 1.0);
                1.0 - ease(progress, envelope.easing)
            } else {
                0.0
            }
        } else {
            1.0
        }
    } else {
        1.0
    };

    fade_in.min(fade_out).clamp(0.0, 1.0)
}

fn wave_offset(config: &WaveEffect, glyph: &GlyphEntry, elapsed_secs: f32) -> f32 {
    let phase = elapsed_secs * config.speed
        + config.phase_offset
        + glyph.primary_index as f32 * config.frequency;
    phase.sin() * config.amplitude
}

fn shake_offset(config: &ShakeEffect, glyph: &GlyphEntry, elapsed_secs: f32) -> Vec2 {
    let scaled = elapsed_secs * config.frequency_hz.max(0.001);
    let cell = scaled.floor() as i32;
    let frac = scaled.fract();
    let a = noise2(config.seed, glyph.primary_index as u64, cell);
    let b = noise2(config.seed, glyph.primary_index as u64, cell + 1);
    let t = frac * frac * (3.0 - 2.0 * frac);
    let blend = a.lerp(b, config.smoothness.clamp(0.0, 1.0) * t);
    Vec2::new(blend.x * config.magnitude.x, blend.y * config.magnitude.y)
}

fn hsv_color(config: &RainbowEffect, glyph: &GlyphEntry, elapsed_secs: f32) -> Color {
    let hue = (elapsed_secs * config.hue_speed + glyph.primary_index as f32 * config.hue_offset)
        .rem_euclid(1.0);
    Color::hsva(hue * 360.0, config.saturation, config.value, 1.0)
}

fn alpha_pulse(config: &AlphaPulseEffect, glyph: &GlyphEntry, elapsed_secs: f32) -> f32 {
    let phase = elapsed_secs * config.speed + glyph.primary_index as f32 * config.phase_offset;
    let t = (phase.sin() + 1.0) * 0.5;
    config.min_alpha + (config.max_alpha - config.min_alpha) * t
}

fn scale_pulse(config: &ScaleEffect, glyph: &GlyphEntry, elapsed_secs: f32) -> f32 {
    let phase = elapsed_secs * config.speed + glyph.primary_index as f32 * config.phase_offset;
    let t = (phase.sin() + 1.0) * 0.5;
    config.min_scale + (config.max_scale - config.min_scale) * t
}

fn ease(value: f32, easing: TextEnvelopeEasing) -> f32 {
    let value = value.clamp(0.0, 1.0);
    match easing {
        TextEnvelopeEasing::Linear => value,
        TextEnvelopeEasing::SmoothStep => value * value * (3.0 - 2.0 * value),
        TextEnvelopeEasing::SmootherStep => {
            value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
        }
    }
}

fn lerp_rgba(a: LinearRgba, b: LinearRgba, t: f32) -> LinearRgba {
    let a = a.to_vec4();
    let b = b.to_vec4();
    let mixed = a + (b - a) * t.clamp(0.0, 1.0);
    LinearRgba::new(mixed.x, mixed.y, mixed.z, mixed.w)
}

fn hash_u64(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51afd7ed558ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ceb9fe1a85ec53);
    value ^ (value >> 33)
}

fn noise2(seed: u64, glyph_index: u64, cell: i32) -> Vec2 {
    let x_bits = hash_u64(seed ^ glyph_index ^ cell as u64);
    let y_bits = hash_u64(seed ^ glyph_index.rotate_left(7) ^ (cell as u64).rotate_left(17));
    Vec2::new(unit_from_bits(x_bits), unit_from_bits(y_bits))
}

fn unit_from_bits(bits: u64) -> f32 {
    let scalar = (bits & 0xffff) as f32 / 65535.0;
    scalar * 2.0 - 1.0
}
