use saddle_animation_text_animation_example_common as common;

use bevy::prelude::*;
use bevy::text::LineBreak;
use saddle_animation_text_animation::{
    AlphaPulseEffect, RainbowEffect, ShakeEffect, TextAnimationBundle, TextAnimationConfig,
    TextEffect, TextRangeSelector, TypewriterConfig, WaveEffect,
};

fn main() {
    let mut app = App::new();
    common::configure_app(&mut app, "text_animation layered_effects");
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(mut commands: Commands) {
    let root = common::spawn_base_scene(
        &mut commands,
        "layered_effects",
        "Wave, rainbow, and targeted shake/pulse run in a declared order on the same text block.",
    );

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Name::new("Instructions"),
            Text::new("Headline: wave + rainbow. Body: shake + alpha pulse on words 1-3 (\"warning beacon\"). Effects compose in declared order."),
            common::demo_text_font(13.0),
            TextColor(Color::srgb(0.55, 0.6, 0.7)),
            TextLayout::new_with_linebreak(LineBreak::WordBoundary),
        ));
        parent.spawn((
            Name::new("Decorative Headline"),
            Text::new("SYSTEMS CHECK"),
            common::demo_text_font(60.0),
            TextColor(Color::WHITE),
            TextLayout::new_with_linebreak(LineBreak::WordBoundary),
            TextAnimationBundle {
                config: TextAnimationConfig {
                    typewriter: TypewriterConfig {
                        enabled: false,
                        ..default()
                    },
                    effects: vec![
                        TextEffect::Wave(WaveEffect {
                            amplitude: 5.0,
                            speed: 3.1,
                            ..default()
                        }),
                        TextEffect::Rainbow(RainbowEffect {
                            strength: 0.92,
                            hue_speed: 0.14,
                            ..default()
                        }),
                    ],
                    ..default()
                },
                ..default()
            },
        ));
        parent.spawn((
            Name::new("Targeted Emphasis"),
            Text::new("Critical warning beacon requires attention now"),
            common::demo_text_font(30.0),
            TextColor(Color::srgb(0.92, 0.95, 0.99)),
            TextLayout::new_with_linebreak(LineBreak::WordBoundary),
            TextAnimationBundle {
                config: TextAnimationConfig {
                    typewriter: TypewriterConfig {
                        enabled: false,
                        ..default()
                    },
                    effects: vec![
                        TextEffect::Shake(ShakeEffect {
                            range: TextRangeSelector::WordRange { start: 1, end: 3 },
                            magnitude: Vec2::new(1.8, 1.3),
                            ..default()
                        }),
                        TextEffect::AlphaPulse(AlphaPulseEffect {
                            range: TextRangeSelector::WordRange { start: 1, end: 3 },
                            min_alpha: 0.65,
                            max_alpha: 1.0,
                            speed: 2.8,
                            ..default()
                        }),
                    ],
                    ..default()
                },
                ..default()
            },
        ));
    });
}
