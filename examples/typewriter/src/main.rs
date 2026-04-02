use saddle_animation_text_animation_example_common as common;

use bevy::prelude::*;
use saddle_animation_text_animation::{
    TextAnimationAction, TextAnimationBundle, TextAnimationCommand, TextAnimationConfig,
    TextAnimationController, TextEffect, WaveEffect,
};

#[derive(Resource)]
struct ScriptedPlayback {
    entity: Entity,
    elapsed_secs: f32,
    stage: usize,
}

fn main() {
    let mut app = App::new();
    common::configure_app(&mut app, "text_animation typewriter");
    app.add_systems(Startup, setup);
    app.add_systems(Update, drive_script);
    app.run();
}

fn setup(mut commands: Commands) {
    let root = common::spawn_base_scene(
        &mut commands,
        "typewriter",
        "This example pauses, resumes, fast-forwards, and restarts the same text automatically.",
    );

    let entity = commands
        .spawn((
            Name::new("Scripted Typewriter"),
            Text::new(
                "Punctuation pauses matter. This line pauses, resumes, completes instantly, then restarts.",
            ),
            common::demo_text_font(34.0),
            TextColor(Color::WHITE),
            TextShadow::default(),
            TextAnimationBundle {
                config: TextAnimationConfig::typewriter(14.0).with_effect(TextEffect::Wave(
                    WaveEffect {
                        amplitude: 2.5,
                        speed: 2.2,
                        ..default()
                    },
                )),
                controller: TextAnimationController {
                    speed_scale: 1.0,
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    commands.entity(root).add_child(entity);
    commands.insert_resource(ScriptedPlayback {
        entity,
        elapsed_secs: 0.0,
        stage: 0,
    });
}

fn drive_script(
    time: Res<Time>,
    mut script: ResMut<ScriptedPlayback>,
    mut commands_out: MessageWriter<TextAnimationCommand>,
) {
    script.elapsed_secs += time.delta_secs();

    match script.stage {
        0 if script.elapsed_secs >= 1.15 => {
            commands_out.write(TextAnimationCommand {
                entity: script.entity,
                action: TextAnimationAction::Pause,
            });
            script.stage += 1;
        }
        1 if script.elapsed_secs >= 2.0 => {
            commands_out.write(TextAnimationCommand {
                entity: script.entity,
                action: TextAnimationAction::Play,
            });
            script.stage += 1;
        }
        2 if script.elapsed_secs >= 3.4 => {
            commands_out.write(TextAnimationCommand {
                entity: script.entity,
                action: TextAnimationAction::FinishNow,
            });
            script.stage += 1;
        }
        3 if script.elapsed_secs >= 5.2 => {
            commands_out.write(TextAnimationCommand {
                entity: script.entity,
                action: TextAnimationAction::Restart,
            });
            script.elapsed_secs = 0.0;
            script.stage = 0;
        }
        _ => {}
    }
}
