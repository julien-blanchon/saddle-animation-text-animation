use saddle_animation_text_animation_example_common as common;

use bevy::prelude::*;
use saddle_animation_text_animation::{
    AlphaPulseEffect, TextAnimationBundle, TextAnimationConfig, TextEffect, TypewriterConfig,
    WaveEffect,
};

fn main() {
    let mut app = App::new();
    common::configure_app(&mut app, "text_animation stress");
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(mut commands: Commands) {
    let root = common::spawn_base_scene(
        &mut commands,
        "stress",
        "Many labels stay active at once without rebuilding their source text every frame.",
    );

    commands.entity(root).with_children(|parent| {
        for index in 0..8 {
            parent.spawn((
                Name::new(format!("Stress HUD {index:02}")),
                Text::new(format!("UI lane {:02}", index + 1)),
                common::demo_text_font(18.0),
                TextColor(Color::srgb(0.82, 0.87, 0.94)),
                TextAnimationBundle {
                    config: TextAnimationConfig {
                        typewriter: TypewriterConfig {
                            enabled: false,
                            ..default()
                        },
                        effects: vec![TextEffect::AlphaPulse(AlphaPulseEffect {
                            min_alpha: 0.78,
                            max_alpha: 1.0,
                            speed: 1.2 + index as f32 * 0.15,
                            ..default()
                        })],
                        ..default()
                    },
                    ..default()
                },
            ));
        }
    });

    for row in 0..8 {
        for column in 0..10 {
            let index = row * 10 + column;
            let x = -430.0 + column as f32 * 96.0;
            let y = 220.0 - row as f32 * 58.0;
            commands.spawn((
                Name::new(format!("Stress Label {index:02}")),
                Text2d::new(format!("+{:02}", index + 1)),
                common::demo_text_font(20.0),
                TextColor(Color::srgb(
                    0.65 + row as f32 * 0.03,
                    0.78,
                    0.92 - column as f32 * 0.02,
                )),
                Transform::from_xyz(x, y - 40.0, 1.0),
                TextAnimationBundle {
                    config: TextAnimationConfig {
                        typewriter: TypewriterConfig {
                            enabled: false,
                            ..default()
                        },
                        effects: vec![TextEffect::Wave(WaveEffect {
                            amplitude: 2.0,
                            speed: 1.6 + index as f32 * 0.02,
                            ..default()
                        })],
                        ..default()
                    },
                    ..default()
                },
            ));
        }
    }
}
