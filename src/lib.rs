mod components;
mod config;
mod effect;
mod glyph_cache;
mod markup;
mod messages;
mod systems;

pub use components::{
    TextAnimationAccessibility, TextAnimationBundle, TextAnimationController,
    TextAnimationDebugState, TextAnimationMarkup, TextMotionPreference, TextRevealSound,
};
pub use config::{
    AlphaPulseEffect, EffectEnvelope, PunctuationDelayConfig, RainbowEffect, RevealMode,
    ScaleEffect, ShakeEffect, TextAnimationConfig, TextAnimationPlaybackState,
    TextAnimationTimeSource, TextEffect, TextEnvelopeEasing, TextRangeSelector, TypewriterConfig,
    WaveEffect,
};
pub use messages::{
    TextAnimationAction, TextAnimationCommand, TextAnimationCompleted, TextAnimationLoopFinished,
    TextAnimationStarted, TextRevealAdvanced, TextRevealCheckpoint, TextRevealSoundRequested,
};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextAnimationSystems {
    DetectChanges,
    Advance,
    EvaluateEffects,
    ApplyOutput,
}

#[derive(Resource, Default)]
pub(crate) struct TextAnimationRuntimeState {
    pub active: bool,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct TextAnimationPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl TextAnimationPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for TextAnimationPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for TextAnimationPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.init_resource::<TextAnimationRuntimeState>()
            .init_resource::<bevy::text::TextIterScratch>()
            .init_resource::<TextAnimationAccessibility>()
            .add_message::<TextAnimationCommand>()
            .add_message::<TextAnimationStarted>()
            .add_message::<TextAnimationCompleted>()
            .add_message::<TextAnimationLoopFinished>()
            .add_message::<TextRevealCheckpoint>()
            .add_message::<TextRevealAdvanced>()
            .add_message::<TextRevealSoundRequested>()
            .register_type::<AlphaPulseEffect>()
            .register_type::<EffectEnvelope>()
            .register_type::<PunctuationDelayConfig>()
            .register_type::<RainbowEffect>()
            .register_type::<RevealMode>()
            .register_type::<ScaleEffect>()
            .register_type::<ShakeEffect>()
            .register_type::<TextAnimationAccessibility>()
            .register_type::<TextAnimationConfig>()
            .register_type::<TextAnimationController>()
            .register_type::<TextAnimationDebugState>()
            .register_type::<TextAnimationMarkup>()
            .register_type::<TextAnimationPlaybackState>()
            .register_type::<TextRevealSound>()
            .register_type::<TextAnimationTimeSource>()
            .register_type::<TextEffect>()
            .register_type::<TextEnvelopeEasing>()
            .register_type::<TextMotionPreference>()
            .register_type::<TextRangeSelector>()
            .register_type::<TypewriterConfig>()
            .register_type::<WaveEffect>()
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(self.deactivate_schedule, systems::deactivate_runtime)
            .configure_sets(
                self.update_schedule,
                (
                    TextAnimationSystems::DetectChanges,
                    TextAnimationSystems::Advance,
                    TextAnimationSystems::EvaluateEffects,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                (
                    systems::initialize_new_animations
                        .in_set(TextAnimationSystems::DetectChanges),
                    (
                        systems::apply_markup_sources,
                        systems::detect_changes,
                    )
                        .in_set(TextAnimationSystems::DetectChanges)
                        .after(systems::initialize_new_animations),
                    systems::apply_commands.in_set(TextAnimationSystems::Advance),
                    systems::advance.in_set(TextAnimationSystems::Advance),
                    systems::evaluate_effects.in_set(TextAnimationSystems::EvaluateEffects),
                ),
            )
            .add_systems(
                PostUpdate,
                systems::cleanup_removed_animations.in_set(TextAnimationSystems::ApplyOutput),
            )
            .add_systems(
                PostUpdate,
                systems::apply_ui_output
                    .in_set(TextAnimationSystems::ApplyOutput)
                    .after(bevy::ui::UiSystems::PostLayout),
            )
            .add_systems(
                PostUpdate,
                systems::apply_world_output
                    .in_set(TextAnimationSystems::ApplyOutput)
                    .after(bevy::sprite::update_text2d_layout),
            );
    }
}

#[cfg(test)]
#[path = "glyph_cache_tests.rs"]
mod glyph_cache_tests;

#[cfg(test)]
#[path = "effect_tests.rs"]
mod effect_tests;

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;

#[cfg(test)]
#[path = "markup_tests.rs"]
mod markup_tests;
