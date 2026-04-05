#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use bevy::prelude::*;
use bevy::sprite::Text2dShadow;
#[cfg(feature = "dev")]
use bevy_brp_extras::BrpExtrasPlugin;
use saddle_animation_text_animation::{
    AlphaPulseEffect, RainbowEffect, ShakeEffect, TextAnimationAccessibility, TextAnimationAction,
    TextAnimationBundle, TextAnimationCommand, TextAnimationCompleted, TextAnimationConfig,
    TextAnimationController, TextAnimationDebugState, TextAnimationLoopFinished,
    TextAnimationMarkup, TextAnimationPlugin, TextAnimationStarted, TextEffect,
    TextMotionPreference, TextRevealCheckpoint, TextRevealSound, TextRevealSoundRequested,
    TypewriterConfig, WaveEffect,
};
use saddle_animation_text_animation_example_common::install_demo_pane;

const DEFAULT_BRPP_PORT: u16 = 15_742;

#[derive(Component)]
struct DiagnosticsText;

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct LabDiagnostics {
    pub started_count: usize,
    pub completed_count: usize,
    pub loop_count: usize,
    pub checkpoint_count: usize,
    pub sound_request_count: usize,
    pub dialogue_visible_units: usize,
    pub dialogue_total_units: usize,
    pub dialogue_effect_count: usize,
    pub unicode_visible_units: usize,
    pub unicode_total_units: usize,
    pub world_visible_graphemes: usize,
    pub headline_effect_count: usize,
    pub stress_label_count: usize,
    pub reduced_motion: bool,
    pub last_completed_name: Option<String>,
    pub last_checkpoint_name: Option<String>,
    pub last_sound_cue: Option<String>,
}

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.035, 0.04, 0.055)));
    app.init_resource::<LabDiagnostics>();
    app.register_type::<LabDiagnostics>();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "text_animation crate-local lab".into(),
            resolution: (1520, 960).into(),
            ..default()
        }),
        ..default()
    }));
    install_demo_pane(&mut app);
    #[cfg(feature = "dev")]
    app.add_plugins(BrpExtrasPlugin::with_port(lab_brp_port()));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::TextAnimationLabE2EPlugin);
    app.add_plugins(TextAnimationPlugin::default());
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            record_started_messages,
            record_completed_messages,
            record_loop_messages,
            record_checkpoint_messages,
            record_sound_messages,
            refresh_diagnostics,
            update_overlay,
        ),
    );
    app.run();
}

#[cfg(feature = "dev")]
fn lab_brp_port() -> u16 {
    std::env::var("BRP_EXTRAS_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_BRPP_PORT)
}

fn setup(mut commands: Commands) {
    commands.spawn((Name::new("Lab Camera"), Camera2d));
    commands.spawn((
        Name::new("Backdrop"),
        Sprite::from_color(Color::srgb(0.07, 0.08, 0.11), Vec2::new(2600.0, 1800.0)),
        Transform::from_xyz(0.0, 0.0, -30.0),
    ));
    commands.spawn((
        Name::new("Upper Band"),
        Sprite::from_color(Color::srgb(0.09, 0.11, 0.15), Vec2::new(2600.0, 340.0)),
        Transform::from_xyz(0.0, 250.0, -20.0),
    ));
    commands.spawn((
        Name::new("Lower Band"),
        Sprite::from_color(Color::srgb(0.06, 0.07, 0.1), Vec2::new(2600.0, 420.0)),
        Transform::from_xyz(0.0, -240.0, -20.0),
    ));

    let ui_root = commands
        .spawn((
            Name::new("Lab Root"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(28.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(18.0),
                overflow: Overflow::scroll_y(),
                ..default()
            },
        ))
        .id();

    commands.entity(ui_root).with_children(|parent| {
        parent.spawn((
            Name::new("Lab Header"),
            Text::new("text motion lab"),
            demo_font(24.0),
            TextColor(Color::WHITE),
        ));
        parent.spawn((
            Name::new("Lab Subtitle"),
            Text::new(
                "Typewriter reveal, layered decorative effects, reduced-motion handling, Unicode samples, Text2d labels, and a stress field.",
            ),
            demo_font(14.0),
            TextColor(Color::srgb(0.76, 0.82, 0.9)),
        ));
        parent.spawn((
            Name::new("Decorative Headline"),
            Text::new("ORBITAL TRAFFIC CONTROL"),
            demo_font(60.0),
            TextColor(Color::WHITE),
            TextAnimationBundle {
                config: TextAnimationConfig {
                    typewriter: TypewriterConfig {
                        enabled: false,
                        ..default()
                    },
                    effects: vec![
                        TextEffect::Wave(WaveEffect {
                            amplitude: 5.5,
                            speed: 3.2,
                            ..default()
                        }),
                        TextEffect::Rainbow(RainbowEffect {
                            hue_speed: 0.14,
                            strength: 0.95,
                            ..default()
                        }),
                    ],
                    ..default()
                },
                ..default()
            },
        ));
        parent.spawn((
            Name::new("Dialogue Typewriter"),
            Text::new(""),
            demo_font(26.0),
            TextColor(Color::WHITE),
            TextShadow::default(),
            TextAnimationMarkup::single(
                "<wave>Docking lane seven</wave> is still occupied. Hold position, wait for the <shake>clearance ping</shake>, then proceed at <scale>low speed</scale>.",
            ),
            TextRevealSound {
                cue_id: "lab.dialogue.blip".into(),
                ..default()
            },
            TextAnimationBundle {
                config: TextAnimationConfig::typewriter(12.0).with_effect(TextEffect::Wave(
                    WaveEffect {
                        amplitude: 2.5,
                        speed: 2.1,
                        ..default()
                    },
                )),
                controller: TextAnimationController {
                    state: saddle_animation_text_animation::TextAnimationPlaybackState::Playing,
                    ..default()
                },
                ..default()
            },
        ));
        parent.spawn((
            Name::new("Global Motion Sample"),
            Text::new("Global motion sample"),
            demo_font(36.0),
            TextColor(Color::srgb(0.92, 0.95, 0.99)),
            TextAnimationBundle {
                config: shared_motion_config(),
                motion: TextMotionPreference::Inherit,
                ..default()
            },
        ));
        parent.spawn((
            Name::new("Reduced Motion Sample"),
            Text::new("Reduced motion sample"),
            demo_font(36.0),
            TextColor(Color::srgb(0.92, 0.95, 0.99)),
            TextAnimationBundle {
                config: shared_motion_config(),
                motion: TextMotionPreference::Reduced,
                ..default()
            },
        ));
        parent.spawn((
            Name::new("Unicode Sample"),
            Text::new("Cafe\u{301} • 👨‍👩‍👧‍👦 • مرحبا • שלום • こんにちは"),
            demo_font(28.0),
            TextColor(Color::srgb(0.95, 0.92, 0.84)),
            TextAnimationBundle {
                config: TextAnimationConfig::typewriter(10.0).with_effect(TextEffect::Rainbow(
                    RainbowEffect {
                        strength: 0.8,
                        ..default()
                    },
                )),
                ..default()
            },
        ));
        parent
            .spawn((
                Name::new("Diagnostics Panel"),
                Node {
                    margin: UiRect::top(Val::Px(12.0)),
                    padding: UiRect::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.03, 0.04, 0.07, 0.82)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Name::new("Diagnostics Text"),
                    DiagnosticsText,
                    Text::new("booting diagnostics…"),
                    demo_font(14.0),
                    TextColor(Color::srgb(0.84, 0.89, 0.96)),
                ));
            });
    });

    commands.spawn((
        Name::new("World Beacon"),
        Sprite::from_color(Color::srgb(0.88, 0.24, 0.2), Vec2::new(92.0, 92.0)),
        Transform::from_xyz(0.0, -180.0, 0.0),
    ));
    commands.spawn((
        Name::new("World Warning"),
        Text2d::new("Vector wash detected"),
        demo_font(34.0),
        TextColor(Color::srgb(0.99, 0.89, 0.78)),
        Text2dShadow::default(),
        Transform::from_xyz(0.0, -110.0, 2.0),
        TextAnimationBundle {
            config: TextAnimationConfig {
                typewriter: TypewriterConfig {
                    enabled: false,
                    ..default()
                },
                effects: vec![
                    TextEffect::Wave(WaveEffect {
                        amplitude: 4.0,
                        speed: 2.3,
                        ..default()
                    }),
                    TextEffect::AlphaPulse(AlphaPulseEffect {
                        min_alpha: 0.8,
                        max_alpha: 1.0,
                        speed: 1.8,
                        ..default()
                    }),
                ],
                ..default()
            },
            ..default()
        },
    ));

    for row in 0..8 {
        for column in 0..10 {
            let index = row * 10 + column;
            let x = -520.0 + column as f32 * 116.0;
            let y = -300.0 - row as f32 * 28.0;
            commands.spawn((
                Name::new(format!("Stress Label {index:02}")),
                Text2d::new(format!("sig{:02}", index + 1)),
                demo_font(14.0),
                TextColor(Color::srgb(
                    0.66 + row as f32 * 0.02,
                    0.78,
                    0.9 - column as f32 * 0.015,
                )),
                Transform::from_xyz(x, y, 1.0),
                TextAnimationBundle {
                    config: TextAnimationConfig {
                        typewriter: TypewriterConfig {
                            enabled: false,
                            ..default()
                        },
                        effects: vec![TextEffect::Wave(WaveEffect {
                            amplitude: 1.8,
                            speed: 1.4 + index as f32 * 0.02,
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

fn demo_font(size: f32) -> TextFont {
    TextFont {
        font_size: size,
        ..default()
    }
}

fn shared_motion_config() -> TextAnimationConfig {
    TextAnimationConfig {
        typewriter: TypewriterConfig {
            enabled: false,
            ..default()
        },
        effects: vec![
            TextEffect::Wave(WaveEffect {
                amplitude: 5.0,
                speed: 3.0,
                reduced_motion_scale: 0.0,
                ..default()
            }),
            TextEffect::Shake(ShakeEffect {
                magnitude: Vec2::new(2.0, 1.5),
                reduced_motion_scale: 0.0,
                ..default()
            }),
            TextEffect::Rainbow(RainbowEffect::default()),
        ],
        ..default()
    }
}

fn record_started_messages(
    mut events: MessageReader<TextAnimationStarted>,
    mut diagnostics: ResMut<LabDiagnostics>,
) {
    diagnostics.started_count += events.read().count();
}

fn record_completed_messages(
    mut events: MessageReader<TextAnimationCompleted>,
    names: Query<&Name>,
    mut diagnostics: ResMut<LabDiagnostics>,
) {
    for event in events.read() {
        diagnostics.completed_count += 1;
        diagnostics.last_completed_name = names.get(event.entity).ok().map(|name| name.to_string());
    }
}

fn record_loop_messages(
    mut events: MessageReader<TextAnimationLoopFinished>,
    mut diagnostics: ResMut<LabDiagnostics>,
) {
    for event in events.read() {
        diagnostics.loop_count += event.completed_loops as usize;
    }
}

fn record_checkpoint_messages(
    mut events: MessageReader<TextRevealCheckpoint>,
    names: Query<&Name>,
    mut diagnostics: ResMut<LabDiagnostics>,
) {
    for event in events.read() {
        diagnostics.checkpoint_count += 1;
        diagnostics.last_checkpoint_name =
            names.get(event.entity).ok().map(|name| name.to_string());
    }
}

fn record_sound_messages(
    mut events: MessageReader<TextRevealSoundRequested>,
    mut diagnostics: ResMut<LabDiagnostics>,
) {
    for event in events.read() {
        diagnostics.sound_request_count += 1;
        diagnostics.last_sound_cue = Some(event.cue_id.clone());
    }
}

fn refresh_diagnostics(
    accessibility: Res<TextAnimationAccessibility>,
    mut diagnostics: ResMut<LabDiagnostics>,
    query: Query<(&Name, &TextAnimationDebugState, &TextAnimationConfig)>,
) {
    diagnostics.reduced_motion = accessibility.reduced_motion;
    diagnostics.stress_label_count = 0;

    for (name, debug, config) in &query {
        let name = name.as_str();
        if name == "Dialogue Typewriter" {
            diagnostics.dialogue_visible_units = debug.revealed_units;
            diagnostics.dialogue_total_units = debug.total_units;
            diagnostics.dialogue_effect_count = debug.effect_count;
        } else if name == "Unicode Sample" {
            diagnostics.unicode_visible_units = debug.revealed_units;
            diagnostics.unicode_total_units = debug.total_units;
        } else if name == "World Warning" {
            diagnostics.world_visible_graphemes = debug.visible_graphemes;
        } else if name == "Decorative Headline" {
            diagnostics.headline_effect_count = config.effects.len();
        } else if name.starts_with("Stress Label ") {
            diagnostics.stress_label_count += 1;
        }
    }
}

fn update_overlay(
    diagnostics: Res<LabDiagnostics>,
    mut query: Query<&mut Text, With<DiagnosticsText>>,
) {
    if !diagnostics.is_changed() {
        return;
    }

    let Ok(mut text) = query.single_mut() else {
        return;
    };

    **text = format!(
        "dialogue {}/{} fx={} | unicode {}/{} | completed {} | checkpoints {} | sound_requests {} {:?} | reduced_motion {} | stress_labels {} | last_completed {:?}",
        diagnostics.dialogue_visible_units,
        diagnostics.dialogue_total_units,
        diagnostics.dialogue_effect_count,
        diagnostics.unicode_visible_units,
        diagnostics.unicode_total_units,
        diagnostics.completed_count,
        diagnostics.checkpoint_count,
        diagnostics.sound_request_count,
        diagnostics.last_sound_cue,
        diagnostics.reduced_motion,
        diagnostics.stress_label_count,
        diagnostics.last_completed_name,
    );
}

fn entity_by_name(world: &mut World, name: &str) -> Option<Entity> {
    let mut query = world.query::<(Entity, &Name)>();
    query
        .iter(world)
        .find(|(_, entity_name)| entity_name.as_str() == name)
        .map(|(entity, _)| entity)
}

pub fn send_command_to_named(world: &mut World, name: &str, action: TextAnimationAction) {
    let Some(entity) = entity_by_name(world, name) else {
        return;
    };
    world
        .resource_mut::<Messages<TextAnimationCommand>>()
        .write(TextAnimationCommand { entity, action });
}

pub fn set_reduced_motion(world: &mut World, reduced: bool) {
    world
        .resource_mut::<TextAnimationAccessibility>()
        .reduced_motion = reduced;
}
