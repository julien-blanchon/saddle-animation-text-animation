use saddle_animation_text_animation_example_common as common;

use bevy::prelude::*;
use bevy::sprite::Text2dShadow;
use saddle_animation_text_animation::{
    AlphaPulseEffect, ShakeEffect, TextAnimationAccessibility, TextAnimationBundle,
    TextAnimationConfig, TextEffect, TextMotionPreference, TypewriterConfig, WaveEffect,
};

fn main() {
    let mut app = App::new();
    common::configure_app(&mut app, "text_animation text2d_world_label");
    app.insert_resource(TextAnimationAccessibility {
        reduced_motion: true,
    });
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(mut commands: Commands) {
    let _root = common::spawn_base_scene(
        &mut commands,
        "text2d_world_label",
        "World-space labels can use the same effect stack and motion accessibility rules as UI text.",
    );

    commands.spawn((
        Name::new("Warning Beacon"),
        Sprite::from_color(Color::srgb(0.82, 0.26, 0.22), Vec2::new(84.0, 84.0)),
        Transform::from_xyz(-180.0, -110.0, 0.0),
    ));
    commands.spawn((
        Name::new("Warning Label"),
        Text2d::new("Hazard field"),
        common::demo_text_font(34.0),
        TextColor(Color::srgb(0.98, 0.9, 0.8)),
        Text2dShadow::default(),
        Transform::from_xyz(-180.0, -34.0, 2.0),
        TextAnimationBundle {
            config: TextAnimationConfig {
                typewriter: TypewriterConfig {
                    enabled: false,
                    ..default()
                },
                effects: vec![
                    TextEffect::Wave(WaveEffect {
                        amplitude: 4.0,
                        speed: 2.6,
                        ..default()
                    }),
                    TextEffect::AlphaPulse(AlphaPulseEffect {
                        min_alpha: 0.78,
                        max_alpha: 1.0,
                        ..default()
                    }),
                ],
                ..default()
            },
            motion: TextMotionPreference::Full,
            ..default()
        },
    ));

    commands.spawn((
        Name::new("Reduced Beacon"),
        Sprite::from_color(Color::srgb(0.2, 0.56, 0.88), Vec2::new(84.0, 84.0)),
        Transform::from_xyz(180.0, -110.0, 0.0),
    ));
    commands.spawn((
        Name::new("Reduced Label"),
        Text2d::new("Docking lane"),
        common::demo_text_font(34.0),
        TextColor(Color::srgb(0.86, 0.95, 1.0)),
        Text2dShadow::default(),
        Transform::from_xyz(180.0, -34.0, 2.0),
        TextAnimationBundle {
            config: TextAnimationConfig {
                typewriter: TypewriterConfig {
                    enabled: false,
                    ..default()
                },
                effects: vec![
                    TextEffect::Wave(WaveEffect {
                        amplitude: 4.0,
                        speed: 2.6,
                        reduced_motion_scale: 0.0,
                        ..default()
                    }),
                    TextEffect::Shake(ShakeEffect {
                        magnitude: Vec2::new(1.6, 1.2),
                        reduced_motion_scale: 0.0,
                        ..default()
                    }),
                    TextEffect::AlphaPulse(AlphaPulseEffect::default()),
                ],
                ..default()
            },
            motion: TextMotionPreference::Reduced,
            ..default()
        },
    ));
}
