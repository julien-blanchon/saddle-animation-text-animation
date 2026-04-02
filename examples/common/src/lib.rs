use bevy::app::AppExit;
use bevy::prelude::*;
use saddle_animation_text_animation::TextAnimationPlugin;

#[derive(Resource)]
struct AutoExitAfter(Timer);

pub fn configure_app(app: &mut App, title: &str) {
    app.insert_resource(ClearColor(Color::srgb(0.045, 0.05, 0.07)));
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: title.into(),
            resolution: (1280, 760).into(),
            ..default()
        }),
        ..default()
    }));
    app.add_plugins(TextAnimationPlugin::default());
    install_auto_exit(app, "TEXT_ANIMATION_EXAMPLE_EXIT_SECS");
}

pub fn install_auto_exit(app: &mut App, env_var: &str) {
    let timer = std::env::var(env_var)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .map(|seconds| AutoExitAfter(Timer::from_seconds(seconds.max(0.1), TimerMode::Once)));

    if let Some(timer) = timer {
        app.insert_resource(timer);
        app.add_systems(Update, auto_exit_after);
    }
}

fn auto_exit_after(
    time: Res<Time>,
    timer: Option<ResMut<AutoExitAfter>>,
    mut exits: MessageWriter<AppExit>,
) {
    let Some(mut timer) = timer else {
        return;
    };

    if timer.0.tick(time.delta()).just_finished() {
        exits.write(AppExit::Success);
    }
}

pub fn demo_text_font(size: f32) -> TextFont {
    TextFont {
        font_size: size,
        ..default()
    }
}

pub fn spawn_base_scene(commands: &mut Commands, title: &str, subtitle: &str) -> Entity {
    commands.spawn((Name::new("Main Camera"), Camera2d));
    commands.spawn((
        Name::new("Backdrop"),
        Sprite::from_color(Color::srgb(0.08, 0.09, 0.12), Vec2::new(2200.0, 1600.0)),
        Transform::from_xyz(0.0, 0.0, -30.0),
    ));

    let root = commands
        .spawn((
            Name::new("Example Root"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(26.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
        ))
        .id();

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Name::new("Example Title"),
            Text::new(title),
            demo_text_font(22.0),
            TextColor(Color::WHITE),
        ));
        parent.spawn((
            Name::new("Example Subtitle"),
            Text::new(subtitle),
            demo_text_font(14.0),
            TextColor(Color::srgb(0.78, 0.83, 0.9)),
        ));
    });

    root
}
