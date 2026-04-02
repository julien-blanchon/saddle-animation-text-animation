use saddle_animation_text_animation_example_common as common;

use bevy::prelude::*;
use bevy::sprite::Text2dShadow;
use saddle_animation_text_animation::{
    AlphaPulseEffect, TextAnimationBundle, TextAnimationConfig, TextEffect, TypewriterConfig,
    WaveEffect,
};

fn main() {
    let mut app = App::new();
    common::configure_app(&mut app, "text_animation basic");
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(mut commands: Commands) {
    let root = common::spawn_base_scene(
        &mut commands,
        "basic",
        "UI text and Text2d labels can share the same animation runtime.",
    );

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Name::new("Animated Headline"),
            Text::new("Shared text motion toolkit"),
            common::demo_text_font(50.0),
            TextColor(Color::WHITE),
            TextShadow::default(),
            TextAnimationBundle {
                config: TextAnimationConfig::typewriter(18.0)
                    .with_effect(TextEffect::Wave(WaveEffect {
                        amplitude: 4.0,
                        speed: 3.0,
                        ..default()
                    })),
                ..default()
            },
        ));
        parent.spawn((
            Name::new("Animated Body"),
            Text::new(
                "This label reveals progressively while a world-space warning marker below loops decorative motion.",
            ),
            common::demo_text_font(24.0),
            TextColor(Color::srgb(0.88, 0.91, 0.97)),
            TextAnimationBundle {
                config: TextAnimationConfig::typewriter(26.0),
                ..default()
            },
        ));
    });

    commands.spawn((
        Name::new("World Beacon"),
        Sprite::from_color(Color::srgb(0.78, 0.22, 0.16), Vec2::new(64.0, 64.0)),
        Transform::from_xyz(0.0, -120.0, 0.0),
    ));
    commands.spawn((
        Name::new("World Warning Label"),
        Text2d::new("Warning marker online"),
        common::demo_text_font(28.0),
        TextColor(Color::srgb(0.98, 0.88, 0.74)),
        Text2dShadow::default(),
        Transform::from_xyz(0.0, -58.0, 2.0),
        TextAnimationBundle {
            config: TextAnimationConfig {
                typewriter: TypewriterConfig {
                    enabled: false,
                    ..default()
                },
                effects: vec![
                    TextEffect::Wave(WaveEffect {
                        amplitude: 3.0,
                        speed: 2.4,
                        ..default()
                    }),
                    TextEffect::AlphaPulse(AlphaPulseEffect {
                        min_alpha: 0.75,
                        max_alpha: 1.0,
                        speed: 1.9,
                        ..default()
                    }),
                ],
                ..default()
            },
            ..default()
        },
    ));
}
