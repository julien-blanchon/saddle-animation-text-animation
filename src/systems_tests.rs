use bevy::image::{Image, TextureAtlasLayout};
use bevy::math::{IVec2, URect, UVec2, Vec2};
use bevy::prelude::*;
use bevy::sprite::{Anchor, Text2d};
use bevy::text::{
    ComputedTextBlock, Font, GlyphAtlasInfo, GlyphAtlasLocation, PositionedGlyph, TextLayoutInfo,
};

use crate::components::TextAnimationRuntime;
use crate::systems::image_handle_for;
use crate::{
    TextAnimationAction, TextAnimationBundle, TextAnimationCommand, TextAnimationCompleted,
    TextAnimationDebugState, TextAnimationPlugin,
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

fn spawn_world_text(app: &mut App, content: &str) -> Entity {
    app.world_mut()
        .spawn((
            Name::new("Animated Text"),
            Text2d::new(content),
            TextFont::default(),
            TextColor(Color::WHITE),
            ComputedTextBlock::default(),
            TextLayoutInfo::default(),
            Anchor::TOP_LEFT,
            TextAnimationBundle::default(),
        ))
        .id()
}

fn drain_messages<T: Message>(app: &mut App) -> Vec<T> {
    app.world_mut()
        .resource_mut::<Messages<T>>()
        .drain()
        .collect()
}

#[test]
fn finish_now_emits_completion_once_until_restart() {
    let mut app = make_app();
    let entity = spawn_world_text(&mut app, "Signal");
    app.update();
    let _ = drain_messages::<TextAnimationCompleted>(&mut app);

    app.world_mut()
        .resource_mut::<Messages<TextAnimationCommand>>()
        .write(TextAnimationCommand {
            entity,
            action: TextAnimationAction::FinishNow,
        });
    app.update();
    let completions = drain_messages::<TextAnimationCompleted>(&mut app);
    assert_eq!(completions.len(), 1);
    assert_eq!(completions[0].entity, entity);

    app.update();
    assert!(drain_messages::<TextAnimationCompleted>(&mut app).is_empty());

    app.world_mut()
        .resource_mut::<Messages<TextAnimationCommand>>()
        .write(TextAnimationCommand {
            entity,
            action: TextAnimationAction::Restart,
        });
    app.update();
    assert!(drain_messages::<TextAnimationCompleted>(&mut app).is_empty());

    app.world_mut()
        .resource_mut::<Messages<TextAnimationCommand>>()
        .write(TextAnimationCommand {
            entity,
            action: TextAnimationAction::FinishNow,
        });
    app.update();
    assert_eq!(drain_messages::<TextAnimationCompleted>(&mut app).len(), 1);
}

#[test]
fn text_replacement_mid_animation_rebuilds_debug_state() {
    let mut app = make_app();
    let entity = spawn_world_text(&mut app, "Hi");
    app.update();

    let initial_units = app
        .world()
        .entity(entity)
        .get::<TextAnimationDebugState>()
        .expect("debug state should be present")
        .total_units;
    assert_eq!(initial_units, 2);

    {
        let mut entity_ref = app.world_mut().entity_mut(entity);
        let mut controller = entity_ref
            .get_mut::<crate::TextAnimationController>()
            .expect("controller should be present");
        controller.elapsed_secs = 0.25;
    }
    app.world_mut()
        .entity_mut(entity)
        .insert(Text2d::new("Hello there"));
    app.update();

    let debug = app
        .world()
        .entity(entity)
        .get::<TextAnimationDebugState>()
        .expect("debug state should be present");
    assert!(debug.total_units > initial_units);
    assert!(debug.revealed_units <= debug.total_units);
}

#[test]
fn uuid_backed_glyph_images_still_convert_to_handles() {
    let mut images = Assets::<Image>::default();
    let id = bevy::asset::AssetId::<Image>::Uuid {
        uuid: bevy::asset::AssetId::<Image>::DEFAULT_UUID,
    };
    images
        .insert(id, Image::default())
        .expect("image should insert");

    let handle = image_handle_for(&mut images, id).expect("handle should convert");
    assert!(matches!(handle, Handle::Uuid(_, _)));
    assert_eq!(handle.id(), id);
}

#[test]
fn world_glyph_overlay_settles_after_layout_arrives() {
    let mut app = make_app();
    let entity = spawn_world_text(&mut app, "A");

    app.update();

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
        scale_factor: 2.0,
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

    let (render_root, glyph_entity) = {
        let runtime = app
            .world()
            .entity(entity)
            .get::<TextAnimationRuntime>()
            .expect("runtime should be present");
        (
            runtime.render_root.expect("render root should exist"),
            *runtime
                .glyph_entities
                .first()
                .expect("one glyph entity should exist"),
        )
    };

    app.update();

    {
        let runtime = app
            .world()
            .entity(entity)
            .get::<TextAnimationRuntime>()
            .expect("runtime should be present");
        assert_eq!(runtime.render_root, Some(render_root));
        assert_eq!(runtime.glyph_entities, vec![glyph_entity]);
    }

    let glyph_ref = app.world().entity(glyph_entity);
    let transform = glyph_ref
        .get::<Transform>()
        .expect("glyph transform should be present");
    let sprite = glyph_ref
        .get::<Sprite>()
        .expect("glyph sprite should be present");
    assert_eq!(transform.translation, Vec3::new(8.0, -10.0, 0.0));
    assert_eq!(sprite.custom_size, Some(Vec2::splat(16.0)));
    assert_eq!(
        sprite.rect,
        Some(URect::from_corners(UVec2::ZERO, UVec2::splat(16)).as_rect())
    );
    assert_eq!(sprite.image.id(), image.id());

    app.update();

    let runtime = app
        .world()
        .entity(entity)
        .get::<TextAnimationRuntime>()
        .expect("runtime should be present");
    assert_eq!(runtime.render_root, Some(render_root));
    assert_eq!(runtime.glyph_entities, vec![glyph_entity]);
}
