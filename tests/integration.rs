use bevy::image::{Image, TextureAtlasLayout};
use bevy::math::{IVec2, URect, UVec2, Vec2};
use bevy::prelude::*;
use bevy::sprite::{Anchor, Text2d};
use bevy::text::{
    ComputedTextBlock, Font, GlyphAtlasInfo, GlyphAtlasLocation, PositionedGlyph, TextLayoutInfo,
};
use bevy::ui::ComputedNode;
use saddle_animation_text_animation::{
    TextAnimationBundle, TextAnimationCommand, TextAnimationConfig, TextAnimationDebugState,
    TextAnimationPlugin, TextAnimationSystems, WaveEffect,
};

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<Image>>();
    app.init_resource::<Assets<TextureAtlasLayout>>();
    app.init_resource::<Assets<Font>>();
    app.add_plugins(TextAnimationPlugin::default());
    app
}

fn assert_text_animation_runtime_resources(app: &App) {
    assert!(app.world().contains_resource::<Assets<Image>>());
    assert!(
        app.world()
            .contains_resource::<Assets<TextureAtlasLayout>>()
    );
    assert!(app.world().contains_resource::<Assets<Font>>());
    assert!(app.world().contains_resource::<bevy::time::Time>());
    assert!(app.world().contains_resource::<bevy::time::Time<Real>>());
}

#[test]
fn plugin_initializes_for_ui_and_world_text() {
    let mut app = make_app();
    app.world_mut().spawn((
        Text::new("UI"),
        TextFont::default(),
        TextColor(Color::WHITE),
        ComputedTextBlock::default(),
        TextLayoutInfo::default(),
        ComputedNode::default(),
        TextAnimationBundle {
            config: TextAnimationConfig::typewriter(20.0).with_effect(
                saddle_animation_text_animation::TextEffect::Wave(WaveEffect::default()),
            ),
            ..TextAnimationBundle::default()
        },
    ));
    app.world_mut().spawn((
        Text2d::new("World"),
        TextFont::default(),
        TextColor(Color::WHITE),
        ComputedTextBlock::default(),
        TextLayoutInfo::default(),
        Anchor::TOP_LEFT,
        TextAnimationBundle::default(),
    ));
    assert_text_animation_runtime_resources(&app);
    app.update();

    let count = {
        let world = app.world_mut();
        let mut query = world.query::<&TextAnimationDebugState>();
        query.iter(world).count()
    };
    assert!(count >= 2);
}

#[test]
fn command_message_is_registered_and_consumed() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Text2d::new("Hello"),
            TextFont::default(),
            TextColor(Color::WHITE),
            ComputedTextBlock::default(),
            TextLayoutInfo::default(),
            Anchor::TOP_LEFT,
            TextAnimationBundle::default(),
        ))
        .id();
    assert_text_animation_runtime_resources(&app);
    app.update();

    app.world_mut()
        .resource_mut::<Messages<TextAnimationCommand>>()
        .write(TextAnimationCommand {
            entity,
            action: saddle_animation_text_animation::TextAnimationAction::Restart,
        });
    app.update();

    let debug = app
        .world()
        .entity(entity)
        .get::<TextAnimationDebugState>()
        .expect("debug state should be present");
    assert!(debug.total_units > 0);
}

#[test]
fn unicode_empty_and_long_text_blocks_do_not_panic() {
    let mut app = make_app();
    app.world_mut().spawn((
        Text::new(""),
        TextFont::default(),
        TextColor(Color::WHITE),
        ComputedTextBlock::default(),
        TextLayoutInfo::default(),
        ComputedNode::default(),
        TextAnimationBundle::default(),
    ));
    app.world_mut().spawn((
        Text2d::new("e\u{301} 👨‍👩‍👧‍👦\r\nمرحبا שלום こんにちは\nA long wrapped sentence should still be safe to rebuild repeatedly."),
        TextFont::default(),
        TextColor(Color::WHITE),
        ComputedTextBlock::default(),
        TextLayoutInfo::default(),
        Anchor::TOP_LEFT,
        TextAnimationBundle::default(),
    ));

    assert_text_animation_runtime_resources(&app);
    app.update();
}

#[test]
fn public_system_sets_can_be_configured() {
    let mut app = make_app();
    app.configure_sets(
        Update,
        (
            TextAnimationSystems::DetectChanges,
            TextAnimationSystems::Advance,
            TextAnimationSystems::EvaluateEffects,
        )
            .chain(),
    );
}

#[test]
fn layout_info_changes_rebuild_render_glyphs() {
    let mut app = make_app();
    let entity = app
        .world_mut()
        .spawn((
            Text2d::new("A"),
            TextFont::default(),
            TextColor(Color::WHITE),
            ComputedTextBlock::default(),
            TextLayoutInfo::default(),
            Anchor::TOP_LEFT,
            TextAnimationBundle::default(),
        ))
        .id();

    app.update();
    let initial = app
        .world()
        .entity(entity)
        .get::<TextAnimationDebugState>()
        .expect("debug state should be present")
        .render_glyphs;
    assert_eq!(initial, 0);

    let image = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    let mut atlas_layout = TextureAtlasLayout::new_empty(UVec2::new(32, 32));
    let glyph_index = atlas_layout.add_texture(URect::from_corners(UVec2::ZERO, UVec2::splat(16)));
    let atlas = app
        .world_mut()
        .resource_mut::<Assets<TextureAtlasLayout>>()
        .add(atlas_layout);

    app.world_mut().entity_mut(entity).insert(TextLayoutInfo {
        scale_factor: 1.0,
        glyphs: vec![PositionedGlyph {
            position: Vec2::new(8.0, 10.0),
            size: Vec2::splat(16.0),
            atlas_info: GlyphAtlasInfo {
                texture: image.id(),
                texture_atlas: atlas.id(),
                location: GlyphAtlasLocation {
                    glyph_index,
                    offset: IVec2::ZERO,
                },
            },
            span_index: 0,
            line_index: 0,
            byte_index: 0,
            byte_length: 1,
        }],
        run_geometry: Vec::new(),
        size: Vec2::new(16.0, 16.0),
    });

    app.update();
    let updated = app
        .world()
        .entity(entity)
        .get::<TextAnimationDebugState>()
        .expect("debug state should be present")
        .render_glyphs;
    assert_eq!(updated, 1);
}
