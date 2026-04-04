use saddle_animation_text_animation_example_common as common;

use bevy::prelude::*;
use saddle_animation_text_animation::{
    TextAnimationBundle, TextAnimationConfig, TextAnimationMarkup, TextRevealSound,
    TextRevealSoundRequested,
};

#[derive(Component)]
struct BlipCounter;

#[derive(Resource, Default)]
struct DialogueStats {
    revealed_chunks: usize,
}

fn main() {
    let mut app = App::new();
    common::configure_app(&mut app, "text_animation dialogue_box");
    app.init_resource::<DialogueStats>();
    app.add_systems(Startup, setup);
    app.add_systems(Update, (track_reveal_blips, update_counter));
    app.run();
}

fn setup(mut commands: Commands) {
    let root = common::spawn_base_scene(
        &mut commands,
        "dialogue_box",
        "Inline tags drive a conversation box without manually authored grapheme ranges.",
    );

    commands.entity(root).with_children(|parent| {
        parent
            .spawn((
                Name::new("Dialogue Panel"),
                Node {
                    width: px(780.0),
                    min_height: px(240.0),
                    margin: UiRect::top(px(18.0)),
                    padding: UiRect::all(px(20.0)),
                    column_gap: px(18.0),
                    flex_direction: FlexDirection::Row,
                    border_radius: BorderRadius::all(px(24.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.09, 0.11, 0.16)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Name::new("Portrait"),
                    Node {
                        width: px(120.0),
                        height: px(160.0),
                        border_radius: BorderRadius::all(px(18.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.24, 0.42, 0.68)),
                ));

                panel
                    .spawn((
                        Name::new("Dialogue Column"),
                        Node {
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: px(10.0),
                            ..default()
                        },
                    ))
                    .with_children(|column| {
                        column.spawn((
                            Name::new("Speaker"),
                            Text::new("Traffic Control // Commander Vale"),
                            common::demo_text_font(18.0),
                            TextColor(Color::srgb(0.74, 0.87, 1.0)),
                        ));
                        column.spawn((
                            Name::new("Dialogue Text"),
                            Text::new(""),
                            common::demo_text_font(30.0),
                            TextColor(Color::WHITE),
                            TextAnimationMarkup::single(
                                "Commander: <wave>Orbit lock achieved</wave>. Route to <shake>docking lane four</shake> and stand by for <scale>burn confirmation</scale>.",
                            ),
                            TextRevealSound {
                                cue_id: "dialogue.blip".into(),
                                ..default()
                            },
                            TextAnimationBundle {
                                config: TextAnimationConfig::typewriter(14.0),
                                ..default()
                            },
                        ));
                        column.spawn((
                            Name::new("Blip Counter"),
                            BlipCounter,
                            Text::new("voice blips: 0"),
                            common::demo_text_font(16.0),
                            TextColor(Color::srgb(0.8, 0.86, 0.94)),
                        ));
                    });
            });
    });
}

fn track_reveal_blips(
    mut stats: ResMut<DialogueStats>,
    mut reveals: MessageReader<TextRevealSoundRequested>,
) {
    for event in reveals.read() {
        if event.cue_id == "dialogue.blip" {
            stats.revealed_chunks += 1;
        }
    }
}

fn update_counter(stats: Res<DialogueStats>, mut label: Single<&mut Text, With<BlipCounter>>) {
    **label = Text::new(format!("voice blips: {}", stats.revealed_chunks));
}
