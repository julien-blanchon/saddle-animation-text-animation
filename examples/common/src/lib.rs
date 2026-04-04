use bevy::app::AppExit;
use bevy::prelude::*;
use saddle_animation_text_animation::{
    AlphaPulseEffect, RainbowEffect, ScaleEffect, ShakeEffect, TextAnimationAccessibility,
    TextAnimationConfig, TextAnimationPlugin, TextEffect, WaveEffect,
};
use saddle_pane::prelude::*;

#[derive(Resource)]
struct AutoExitAfter(Timer);

#[derive(Resource, Default)]
struct TextAnimationDemoPaneInstalled;

#[derive(Component, Clone)]
struct TextAnimationPaneBaseline(TextAnimationConfig);

#[derive(Resource, Clone, Pane)]
#[pane(title = "Text Motion")]
struct TextAnimationDemoPane {
    #[pane(slider, min = 0.5, max = 3.0, step = 0.05)]
    reveal_speed_scale: f32,
    #[pane(slider, min = 0.5, max = 3.0, step = 0.05)]
    effect_speed_scale: f32,
    #[pane(slider, min = 0.0, max = 2.0, step = 0.05)]
    motion_scale: f32,
    reduced_motion: bool,
}

impl Default for TextAnimationDemoPane {
    fn default() -> Self {
        Self {
            reveal_speed_scale: 1.0,
            effect_speed_scale: 1.0,
            motion_scale: 1.0,
            reduced_motion: false,
        }
    }
}

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
    install_demo_pane(app);
    app.add_plugins(TextAnimationPlugin::default());
    install_auto_exit(app, "TEXT_ANIMATION_EXAMPLE_EXIT_SECS");
}

pub fn install_demo_pane(app: &mut App) {
    if app.world().contains_resource::<TextAnimationDemoPaneInstalled>() {
        return;
    }

    app.insert_resource(TextAnimationDemoPaneInstalled);
    if !app.world().contains_resource::<TextAnimationDemoPane>() {
        app.insert_resource(TextAnimationDemoPane::default());
    }
    if !app.is_plugin_added::<saddle_pane::PanePlugin>() {
        app.add_plugins(pane_plugins());
    }
    app.register_pane::<TextAnimationDemoPane>();
    app.add_systems(Update, (capture_pane_baselines, sync_demo_pane).chain());
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

pub fn set_reduced_motion(app: &mut App, enabled: bool) {
    let mut pane = app
        .world_mut()
        .get_resource::<TextAnimationDemoPane>()
        .cloned()
        .unwrap_or_default();
    pane.reduced_motion = enabled;
    app.insert_resource(pane);
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

pub fn pane_plugins() -> (
    bevy_flair::FlairPlugin,
    bevy_input_focus::InputDispatchPlugin,
    bevy_ui_widgets::UiWidgetsPlugins,
    bevy_input_focus::tab_navigation::TabNavigationPlugin,
    saddle_pane::PanePlugin,
) {
    (
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        saddle_pane::PanePlugin,
    )
}

fn capture_pane_baselines(
    mut commands: Commands,
    query: Query<(Entity, &TextAnimationConfig), (Added<TextAnimationConfig>, Without<TextAnimationPaneBaseline>)>,
) {
    for (entity, config) in &query {
        commands
            .entity(entity)
            .insert(TextAnimationPaneBaseline(config.clone()));
    }
}

fn sync_demo_pane(
    pane: Res<TextAnimationDemoPane>,
    mut accessibility: ResMut<TextAnimationAccessibility>,
    mut query: Query<(&TextAnimationPaneBaseline, &mut TextAnimationConfig)>,
) {
    if !pane.is_changed() && !pane.is_added() {
        return;
    }

    accessibility.reduced_motion = pane.reduced_motion;

    for (baseline, mut config) in &mut query {
        *config = scaled_config(&baseline.0, &pane);
    }
}

fn scaled_config(base: &TextAnimationConfig, pane: &TextAnimationDemoPane) -> TextAnimationConfig {
    let mut config = base.clone();
    config.typewriter.units_per_second *= pane.reveal_speed_scale;
    for effect in &mut config.effects {
        match effect {
            TextEffect::Wave(WaveEffect {
                amplitude, speed, ..
            }) => {
                *amplitude *= pane.motion_scale;
                *speed *= pane.effect_speed_scale;
            }
            TextEffect::Shake(ShakeEffect {
                magnitude,
                frequency_hz,
                ..
            }) => {
                *magnitude *= pane.motion_scale;
                *frequency_hz *= pane.effect_speed_scale;
            }
            TextEffect::Rainbow(RainbowEffect { hue_speed, .. }) => {
                *hue_speed *= pane.effect_speed_scale;
            }
            TextEffect::AlphaPulse(AlphaPulseEffect { speed, .. }) => {
                *speed *= pane.effect_speed_scale;
            }
            TextEffect::Scale(ScaleEffect {
                min_scale,
                max_scale,
                speed,
                ..
            }) => {
                *min_scale = 1.0 + (*min_scale - 1.0) * pane.motion_scale;
                *max_scale = 1.0 + (*max_scale - 1.0) * pane.motion_scale;
                *speed *= pane.effect_speed_scale;
            }
        }
    }
    config
}
