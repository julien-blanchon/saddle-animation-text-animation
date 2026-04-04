use saddle_animation_text_animation_example_common as common;

use bevy::prelude::*;
use saddle_animation_text_animation::{
    RainbowEffect, ShakeEffect, TextAnimationBundle, TextAnimationConfig, TextEffect,
    TextMotionPreference, TypewriterConfig, WaveEffect,
};

fn main() {
    let mut app = App::new();
    common::configure_app(&mut app, "text_animation reduced_motion");
    common::set_reduced_motion(&mut app, true);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(mut commands: Commands) {
    let root = common::spawn_base_scene(
        &mut commands,
        "reduced_motion",
        "Global reduced motion is on. The top label opts into full motion; the bottom label keeps color while suppressing position-heavy effects.",
    );

    let shared_config = TextAnimationConfig {
        typewriter: TypewriterConfig {
            enabled: false,
            ..default()
        },
        effects: vec![
            TextEffect::Wave(WaveEffect {
                amplitude: 6.5,
                speed: 3.4,
                reduced_motion_scale: 0.0,
                ..default()
            }),
            TextEffect::Shake(ShakeEffect {
                magnitude: Vec2::new(2.4, 1.8),
                reduced_motion_scale: 0.0,
                ..default()
            }),
            TextEffect::Rainbow(RainbowEffect::default()),
        ],
        ..default()
    };

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Name::new("Full Motion Sample"),
            Text::new("Full motion override"),
            common::demo_text_font(46.0),
            TextColor(Color::WHITE),
            TextAnimationBundle {
                config: shared_config.clone(),
                motion: TextMotionPreference::Full,
                ..default()
            },
        ));
        parent.spawn((
            Name::new("Reduced Motion Sample"),
            Text::new("Reduced motion variant"),
            common::demo_text_font(46.0),
            TextColor(Color::WHITE),
            TextAnimationBundle {
                config: shared_config,
                motion: TextMotionPreference::Reduced,
                ..default()
            },
        ));
    });
}
