use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Default)]
pub enum TextAnimationPlaybackState {
    #[default]
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Default)]
pub enum TextAnimationTimeSource {
    #[default]
    Scaled,
    Unscaled,
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Default)]
pub enum RevealMode {
    Instant,
    #[default]
    Grapheme,
    Word,
    Line,
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq)]
pub struct PunctuationDelayConfig {
    pub sentence_extra_secs: f32,
    pub clause_extra_secs: f32,
    pub ellipsis_extra_secs: f32,
    pub newline_extra_secs: f32,
}

impl Default for PunctuationDelayConfig {
    fn default() -> Self {
        Self {
            sentence_extra_secs: 0.18,
            clause_extra_secs: 0.07,
            ellipsis_extra_secs: 0.25,
            newline_extra_secs: 0.10,
        }
    }
}

#[derive(Debug, Clone, Reflect, PartialEq)]
pub struct TypewriterConfig {
    pub enabled: bool,
    pub reveal_mode: RevealMode,
    pub units_per_second: f32,
    pub punctuation_delay: PunctuationDelayConfig,
}

impl Default for TypewriterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            reveal_mode: RevealMode::Grapheme,
            units_per_second: 30.0,
            punctuation_delay: PunctuationDelayConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Default)]
pub enum TextRangeSelector {
    #[default]
    All,
    GraphemeRange {
        start: usize,
        end: usize,
    },
    WordRange {
        start: usize,
        end: usize,
    },
    LineRange {
        start: usize,
        end: usize,
    },
    SectionRange {
        start: usize,
        end: usize,
    },
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Default)]
pub enum TextEnvelopeEasing {
    Linear,
    #[default]
    SmoothStep,
    SmootherStep,
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq)]
pub struct EffectEnvelope {
    pub start_delay_secs: f32,
    pub fade_in_secs: f32,
    pub end_after_secs: Option<f32>,
    pub fade_out_secs: f32,
    pub easing: TextEnvelopeEasing,
}

impl Default for EffectEnvelope {
    fn default() -> Self {
        Self {
            start_delay_secs: 0.0,
            fade_in_secs: 0.0,
            end_after_secs: None,
            fade_out_secs: 0.0,
            easing: TextEnvelopeEasing::SmoothStep,
        }
    }
}

#[derive(Debug, Clone, Reflect, PartialEq)]
pub struct WaveEffect {
    pub range: TextRangeSelector,
    pub amplitude: f32,
    pub frequency: f32,
    pub phase_offset: f32,
    pub speed: f32,
    pub envelope: EffectEnvelope,
    pub reduced_motion_scale: f32,
}

impl Default for WaveEffect {
    fn default() -> Self {
        Self {
            range: TextRangeSelector::All,
            amplitude: 6.0,
            frequency: 0.38,
            phase_offset: 0.45,
            speed: 3.5,
            envelope: EffectEnvelope::default(),
            reduced_motion_scale: 0.0,
        }
    }
}

#[derive(Debug, Clone, Reflect, PartialEq)]
pub struct ShakeEffect {
    pub range: TextRangeSelector,
    pub magnitude: Vec2,
    pub frequency_hz: f32,
    pub smoothness: f32,
    pub seed: u64,
    pub envelope: EffectEnvelope,
    pub reduced_motion_scale: f32,
}

impl Default for ShakeEffect {
    fn default() -> Self {
        Self {
            range: TextRangeSelector::All,
            magnitude: Vec2::splat(2.5),
            frequency_hz: 10.0,
            smoothness: 0.75,
            seed: 7,
            envelope: EffectEnvelope::default(),
            reduced_motion_scale: 0.0,
        }
    }
}

#[derive(Debug, Clone, Reflect, PartialEq)]
pub struct RainbowEffect {
    pub range: TextRangeSelector,
    pub hue_speed: f32,
    pub hue_offset: f32,
    pub saturation: f32,
    pub value: f32,
    pub strength: f32,
    pub envelope: EffectEnvelope,
}

impl Default for RainbowEffect {
    fn default() -> Self {
        Self {
            range: TextRangeSelector::All,
            hue_speed: 0.20,
            hue_offset: 0.12,
            saturation: 0.90,
            value: 1.0,
            strength: 1.0,
            envelope: EffectEnvelope::default(),
        }
    }
}

#[derive(Debug, Clone, Reflect, PartialEq)]
pub struct AlphaPulseEffect {
    pub range: TextRangeSelector,
    pub min_alpha: f32,
    pub max_alpha: f32,
    pub speed: f32,
    pub phase_offset: f32,
    pub envelope: EffectEnvelope,
}

impl Default for AlphaPulseEffect {
    fn default() -> Self {
        Self {
            range: TextRangeSelector::All,
            min_alpha: 0.70,
            max_alpha: 1.0,
            speed: 2.5,
            phase_offset: 0.15,
            envelope: EffectEnvelope::default(),
        }
    }
}

#[derive(Debug, Clone, Reflect, PartialEq)]
pub struct ScaleEffect {
    pub range: TextRangeSelector,
    pub min_scale: f32,
    pub max_scale: f32,
    pub speed: f32,
    pub phase_offset: f32,
    pub envelope: EffectEnvelope,
}

impl Default for ScaleEffect {
    fn default() -> Self {
        Self {
            range: TextRangeSelector::All,
            min_scale: 0.92,
            max_scale: 1.12,
            speed: 2.0,
            phase_offset: 0.1,
            envelope: EffectEnvelope::default(),
        }
    }
}

#[derive(Debug, Clone, Reflect, PartialEq)]
pub enum TextEffect {
    Wave(WaveEffect),
    Shake(ShakeEffect),
    Rainbow(RainbowEffect),
    AlphaPulse(AlphaPulseEffect),
    Scale(ScaleEffect),
}

#[derive(Debug, Clone, Component, Reflect, PartialEq)]
#[reflect(Component)]
pub struct TextAnimationConfig {
    pub typewriter: TypewriterConfig,
    pub effects: Vec<TextEffect>,
    pub continue_effects_after_reveal: bool,
}

impl Default for TextAnimationConfig {
    fn default() -> Self {
        Self {
            typewriter: TypewriterConfig::default(),
            effects: Vec::new(),
            continue_effects_after_reveal: true,
        }
    }
}

impl TextAnimationConfig {
    pub fn typewriter(units_per_second: f32) -> Self {
        Self {
            typewriter: TypewriterConfig {
                units_per_second,
                ..TypewriterConfig::default()
            },
            ..Self::default()
        }
    }

    pub fn with_effect(mut self, effect: TextEffect) -> Self {
        self.effects.push(effect);
        self
    }
}
