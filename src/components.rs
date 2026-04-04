use bevy::prelude::*;
use bevy::text::{TextBackgroundColor, UnderlineColor};
use bevy::{sprite::Text2dShadow, text::StrikethroughColor};

use crate::config::{TextAnimationConfig, TextAnimationPlaybackState, TextAnimationTimeSource};

#[derive(Debug, Clone, Copy, Component, Reflect, PartialEq, Eq, Default)]
#[reflect(Component)]
pub enum TextMotionPreference {
    #[default]
    Inherit,
    Full,
    Reduced,
}

#[derive(Debug, Clone, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct TextAnimationAccessibility {
    pub reduced_motion: bool,
}

#[derive(Debug, Clone, Component, Reflect, PartialEq, Default)]
#[reflect(Component)]
pub struct TextAnimationMarkup {
    pub sections: Vec<String>,
}

impl TextAnimationMarkup {
    pub fn single(text: impl Into<String>) -> Self {
        Self {
            sections: vec![text.into()],
        }
    }
}

#[derive(Debug, Clone, Component, Reflect, PartialEq, Eq)]
#[reflect(Component)]
pub struct TextRevealSound {
    pub cue_id: String,
    pub every_n_units: usize,
    pub skip_whitespace: bool,
}

impl Default for TextRevealSound {
    fn default() -> Self {
        Self {
            cue_id: "text.reveal".into(),
            every_n_units: 1,
            skip_whitespace: true,
        }
    }
}

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct TextAnimationController {
    pub state: TextAnimationPlaybackState,
    pub time_source: TextAnimationTimeSource,
    pub elapsed_secs: f32,
    pub speed_scale: f32,
    pub repeat: bool,
}

impl Default for TextAnimationController {
    fn default() -> Self {
        Self {
            state: TextAnimationPlaybackState::Playing,
            time_source: TextAnimationTimeSource::Scaled,
            elapsed_secs: 0.0,
            speed_scale: 1.0,
            repeat: false,
        }
    }
}

impl TextAnimationController {
    pub fn paused() -> Self {
        Self {
            state: TextAnimationPlaybackState::Paused,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct TextAnimationDebugState {
    pub total_graphemes: usize,
    pub visible_graphemes: usize,
    pub total_units: usize,
    pub revealed_units: usize,
    pub elapsed_secs: f32,
    pub render_glyphs: usize,
    pub effect_count: usize,
    pub active: bool,
}

impl Default for TextAnimationDebugState {
    fn default() -> Self {
        Self {
            total_graphemes: 0,
            visible_graphemes: 0,
            total_units: 0,
            revealed_units: 0,
            elapsed_secs: 0.0,
            render_glyphs: 0,
            effect_count: 0,
            active: false,
        }
    }
}

#[derive(Bundle, Default)]
pub struct TextAnimationBundle {
    pub config: TextAnimationConfig,
    pub controller: TextAnimationController,
    pub motion: TextMotionPreference,
    pub debug: TextAnimationDebugState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RootKind {
    Ui,
    World,
}

#[derive(Debug, Clone)]
pub(crate) struct CachedSectionStyle {
    pub text_color: Color,
    pub background_color: Option<TextBackgroundColor>,
    pub underline_color: Option<UnderlineColor>,
    pub strikethrough_color: Option<StrikethroughColor>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct HiddenStyleCache {
    pub sections: Vec<(Entity, CachedSectionStyle)>,
    pub ui_shadow: Option<TextShadow>,
    pub world_shadow: Option<Text2dShadow>,
}

#[derive(Debug, Component)]
pub(crate) struct TextAnimationRuntime {
    pub needs_rebuild: bool,
    pub root_kind: Option<RootKind>,
    pub render_root: Option<Entity>,
    pub glyph_entities: Vec<Entity>,
    pub markup_effects: Vec<crate::config::TextEffect>,
    pub hidden_styles: HiddenStyleCache,
    pub cache: crate::glyph_cache::TextAnimationCache,
    pub sent_started: bool,
    pub sent_completed: bool,
    pub last_visible_units: usize,
    pub pending_loops: u32,
    pub evaluated_visible_units: usize,
    pub evaluated_visible_graphemes: usize,
    pub evaluated_effect_elapsed_secs: f32,
    pub reduced_motion_active: bool,
}

impl Default for TextAnimationRuntime {
    fn default() -> Self {
        Self {
            needs_rebuild: true,
            root_kind: None,
            render_root: None,
            glyph_entities: Vec::new(),
            markup_effects: Vec::new(),
            hidden_styles: HiddenStyleCache::default(),
            cache: crate::glyph_cache::TextAnimationCache::default(),
            sent_started: false,
            sent_completed: false,
            last_visible_units: 0,
            pending_loops: 0,
            evaluated_visible_units: 0,
            evaluated_visible_graphemes: 0,
            evaluated_effect_elapsed_secs: 0.0,
            reduced_motion_active: false,
        }
    }
}

#[derive(Debug, Component)]
pub(crate) struct TextAnimationRenderRoot;

#[derive(Debug, Component)]
pub(crate) struct TextAnimationGlyph {
    pub glyph_index: usize,
}
