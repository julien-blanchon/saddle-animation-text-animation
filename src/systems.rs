use core::marker::PhantomData;

use bevy::asset::{AssetId, Assets};
use bevy::ecs::system::SystemParam;
use bevy::image::Image;
use bevy::image::TextureAtlasLayout;
use bevy::prelude::*;
use bevy::sprite::{Anchor, Sprite, Text2d};
use bevy::text::{
    ComputedTextBlock, TextBackgroundColor, TextColor, TextLayoutInfo, TextReader, TextRoot,
    TextWriter, UnderlineColor,
};
use bevy::ui::UiGlobalTransform;
use bevy::{
    prelude::TextUiReader,
    sprite::{Text2dReader, Text2dShadow, Text2dWriter},
    text::StrikethroughColor,
    ui::widget::TextUiWriter,
};

use crate::TextAnimationRuntimeState;
use crate::components::{
    CachedSectionStyle, HiddenStyleCache, RootKind, TextAnimationAccessibility,
    TextAnimationController, TextAnimationDebugState, TextAnimationGlyph, TextAnimationMarkup,
    TextAnimationRenderRoot, TextAnimationRuntime, TextMotionPreference, TextRevealSound,
};
use crate::config::{TextAnimationConfig, TextAnimationPlaybackState, TextAnimationTimeSource};
use crate::effect::{GlyphVisual, apply_effects};
use crate::glyph_cache::{
    RevealUnit, SectionSnapshot, TextAnimationCache, build_cache, recalc_units,
};
use crate::markup::parse_sections;
use crate::messages::{
    TextAnimationCommand, TextAnimationCompleted, TextAnimationLoopFinished, TextAnimationStarted,
    TextRevealAdvanced, TextRevealCheckpoint, TextRevealSoundRequested,
};

#[derive(SystemParam)]
pub(crate) struct AnimationMessages<'w, 's> {
    started: MessageWriter<'w, TextAnimationStarted>,
    completed: MessageWriter<'w, TextAnimationCompleted>,
    looped: MessageWriter<'w, TextAnimationLoopFinished>,
    checkpoints: MessageWriter<'w, TextRevealCheckpoint>,
    reveal_advanced: MessageWriter<'w, TextRevealAdvanced>,
    reveal_sound: MessageWriter<'w, TextRevealSoundRequested>,
    marker: PhantomData<&'s ()>,
}

pub(crate) fn activate_runtime(mut state: ResMut<TextAnimationRuntimeState>) {
    state.active = true;
}

pub(crate) fn deactivate_runtime(
    mut state: ResMut<TextAnimationRuntimeState>,
    mut commands: Commands,
    mut runtimes: Query<(
        Entity,
        &mut TextAnimationRuntime,
        Option<&mut TextAnimationDebugState>,
    )>,
    mut text_colors: Query<&mut TextColor>,
    mut background_colors: Query<&mut TextBackgroundColor>,
    mut underline_colors: Query<&mut UnderlineColor>,
    mut strikethrough_colors: Query<&mut StrikethroughColor>,
    mut ui_shadows: Query<&mut TextShadow>,
    mut world_shadows: Query<&mut Text2dShadow>,
) {
    state.active = false;
    for (entity, mut runtime, debug) in &mut runtimes {
        restore_source_styles(
            entity,
            &runtime.hidden_styles,
            &mut text_colors,
            &mut background_colors,
            &mut underline_colors,
            &mut strikethrough_colors,
            &mut ui_shadows,
            &mut world_shadows,
        );
        if let Some(render_root) = runtime.render_root.take() {
            commands.entity(render_root).despawn_children();
            commands.entity(render_root).despawn();
        }
        runtime.glyph_entities.clear();
        runtime.evaluated_visible_units = 0;
        runtime.evaluated_visible_graphemes = 0;
        runtime.evaluated_effect_elapsed_secs = 0.0;
        runtime.reduced_motion_active = false;
        if let Some(mut debug) = debug {
            debug.active = false;
        }
    }
}

pub(crate) fn initialize_new_animations(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            With<TextAnimationConfig>,
            Or<(Added<TextAnimationConfig>, Added<TextAnimationController>)>,
            Without<TextAnimationRuntime>,
        ),
    >,
) {
    for entity in &query {
        commands.entity(entity).insert((
            TextAnimationRuntime::default(),
            TextAnimationDebugState::default(),
        ));
    }
}

pub(crate) fn apply_markup_sources(
    runtime_state: Res<TextAnimationRuntimeState>,
    mut ui_roots: Query<
        (
            Entity,
            Ref<TextAnimationMarkup>,
            Option<&mut TextAnimationRuntime>,
        ),
        (With<Text>, Without<Text2d>),
    >,
    mut world_roots: Query<
        (
            Entity,
            Ref<TextAnimationMarkup>,
            Option<&mut TextAnimationRuntime>,
        ),
        (With<Text2d>, Without<Text>),
    >,
    mut writers: ParamSet<(TextUiWriter, Text2dWriter)>,
) {
    if !runtime_state.active {
        return;
    }

    for (entity, markup, runtime) in &mut ui_roots {
        if !markup.is_added() && !markup.is_changed() {
            continue;
        }

        let parsed = parse_sections(&markup.sections);
        apply_markup_sections(entity, &parsed.sections, &mut writers.p0());
        if let Some(mut runtime) = runtime {
            runtime.markup_effects = parsed.effects;
            runtime.needs_rebuild = true;
            runtime.root_kind = Some(RootKind::Ui);
        }
    }

    for (entity, markup, runtime) in &mut world_roots {
        if !markup.is_added() && !markup.is_changed() {
            continue;
        }

        let parsed = parse_sections(&markup.sections);
        apply_markup_sections(entity, &parsed.sections, &mut writers.p1());
        if let Some(mut runtime) = runtime {
            runtime.markup_effects = parsed.effects;
            runtime.needs_rebuild = true;
            runtime.root_kind = Some(RootKind::World);
        }
    }
}

pub(crate) fn apply_commands(
    mut commands_in: MessageReader<TextAnimationCommand>,
    mut controllers: Query<(&mut TextAnimationController, &mut TextAnimationRuntime)>,
) {
    for command in commands_in.read() {
        let Ok((mut controller, mut runtime)) = controllers.get_mut(command.entity) else {
            continue;
        };
        match command.action {
            crate::messages::TextAnimationAction::Play => {
                controller.state = TextAnimationPlaybackState::Playing;
            }
            crate::messages::TextAnimationAction::Pause => {
                controller.state = TextAnimationPlaybackState::Paused;
            }
            crate::messages::TextAnimationAction::Restart => {
                controller.state = TextAnimationPlaybackState::Playing;
                controller.elapsed_secs = 0.0;
                runtime.sent_started = false;
                runtime.sent_completed = false;
                runtime.last_visible_units = 0;
                runtime.pending_loops = 0;
            }
            crate::messages::TextAnimationAction::FinishNow => {
                controller.state = TextAnimationPlaybackState::Playing;
                controller.elapsed_secs = runtime.cache.total_duration_secs;
            }
        }
    }
}

pub(crate) fn detect_changes(
    runtime_state: Res<TextAnimationRuntimeState>,
    mut ui_roots: Query<
        (
            Ref<TextAnimationConfig>,
            Ref<Text>,
            &mut TextAnimationRuntime,
        ),
        Without<Text2d>,
    >,
    mut world_roots: Query<
        (
            Ref<TextAnimationConfig>,
            Ref<Text2d>,
            &mut TextAnimationRuntime,
        ),
        Without<Text>,
    >,
) {
    if !runtime_state.active {
        return;
    }

    for (config, text, mut runtime) in &mut ui_roots {
        if text.is_changed() {
            // Text content changed — full rebuild needed (re-segment graphemes, remap glyphs).
            runtime.needs_rebuild = true;
            runtime.root_kind = Some(RootKind::Ui);
        } else if config.is_changed() {
            // Config-only change (speed, effects, etc.) — just recalculate reveal unit timing.
            // This is much cheaper and avoids flickering from glyph remapping/render tree rebuild.
            runtime.needs_recalc_units = true;
        }
    }

    for (config, text, mut runtime) in &mut world_roots {
        if text.is_changed() {
            runtime.needs_rebuild = true;
            runtime.root_kind = Some(RootKind::World);
        } else if config.is_changed() {
            runtime.needs_recalc_units = true;
        }
    }
}

pub(crate) fn advance(
    runtime_state: Res<TextAnimationRuntimeState>,
    scaled_time: Res<Time>,
    real_time: Res<Time<Real>>,
    mut query: Query<(&mut TextAnimationController, &mut TextAnimationRuntime)>,
) {
    if !runtime_state.active {
        return;
    }

    for (mut controller, mut runtime) in &mut query {
        if controller.state != TextAnimationPlaybackState::Playing {
            continue;
        }

        let delta_secs = match controller.time_source {
            TextAnimationTimeSource::Scaled => scaled_time.delta_secs(),
            TextAnimationTimeSource::Unscaled => real_time.delta_secs(),
        } * controller.speed_scale.max(0.0);

        let previous = controller.elapsed_secs;
        controller.elapsed_secs += delta_secs;

        let duration = runtime.cache.total_duration_secs;
        if controller.repeat && duration > 0.0 && controller.elapsed_secs > duration {
            let previous_loops = (previous / duration).floor() as u32;
            let current_loops = (controller.elapsed_secs / duration).floor() as u32;
            if current_loops > previous_loops {
                runtime.pending_loops += current_loops - previous_loops;
                runtime.sent_started = false;
                runtime.sent_completed = false;
                runtime.last_visible_units = 0;
            }
            controller.elapsed_secs = controller.elapsed_secs.rem_euclid(duration);
        }
    }
}

pub(crate) fn evaluate_effects(
    runtime_state: Res<TextAnimationRuntimeState>,
    motion_accessibility: Res<TextAnimationAccessibility>,
    mut query: Query<(
        &TextAnimationConfig,
        &TextAnimationController,
        Option<&TextMotionPreference>,
        &mut TextAnimationRuntime,
    )>,
) {
    if !runtime_state.active {
        return;
    }

    for (config, controller, motion, mut runtime) in &mut query {
        refresh_runtime_evaluation(
            config,
            controller,
            motion.copied(),
            &motion_accessibility,
            &mut runtime,
        );
    }
}

pub(crate) fn cleanup_removed_animations(
    mut commands: Commands,
    mut removed: RemovedComponents<TextAnimationConfig>,
    mut runtimes: Query<&mut TextAnimationRuntime>,
    mut text_colors: Query<&mut TextColor>,
    mut background_colors: Query<&mut TextBackgroundColor>,
    mut underline_colors: Query<&mut UnderlineColor>,
    mut strikethrough_colors: Query<&mut StrikethroughColor>,
    mut ui_shadows: Query<&mut TextShadow>,
    mut world_shadows: Query<&mut Text2dShadow>,
) {
    for entity in removed.read() {
        let Ok(mut runtime) = runtimes.get_mut(entity) else {
            continue;
        };
        restore_source_styles(
            entity,
            &runtime.hidden_styles,
            &mut text_colors,
            &mut background_colors,
            &mut underline_colors,
            &mut strikethrough_colors,
            &mut ui_shadows,
            &mut world_shadows,
        );
        if let Some(render_root) = runtime.render_root.take() {
            commands.entity(render_root).despawn_children();
            commands.entity(render_root).despawn();
        }
        commands.entity(entity).remove::<TextAnimationRuntime>();
    }
}

pub(crate) fn apply_ui_output(
    runtime_state: Res<TextAnimationRuntimeState>,
    motion_accessibility: Res<TextAnimationAccessibility>,
    mut commands: Commands,
    mut images: Option<ResMut<Assets<Image>>>,
    texture_atlases: Option<Res<Assets<TextureAtlasLayout>>>,
    mut roots: Query<
        (
            Entity,
            &TextAnimationConfig,
            &TextAnimationController,
            Option<&TextMotionPreference>,
            Option<&TextRevealSound>,
            &ComputedTextBlock,
            Ref<TextLayoutInfo>,
            &ComputedNode,
            &UiGlobalTransform,
            &mut TextAnimationRuntime,
            &mut TextAnimationDebugState,
        ),
        (With<Text>, Without<Text2d>),
    >,
    mut render_root_nodes: Query<
        &mut Node,
        (With<TextAnimationRenderRoot>, Without<TextAnimationGlyph>),
    >,
    mut glyph_query: Query<
        (&TextAnimationGlyph, &mut Node, &mut ImageNode),
        Without<TextAnimationRenderRoot>,
    >,
    mut text_access: ParamSet<(TextUiReader, Query<&mut TextColor>)>,
    mut background_colors: Query<&mut TextBackgroundColor>,
    mut underline_colors: Query<&mut UnderlineColor>,
    mut strikethrough_colors: Query<&mut StrikethroughColor>,
    mut ui_shadows: Query<&mut TextShadow>,
    mut messages: AnimationMessages,
) {
    if !runtime_state.active {
        return;
    }
    let Some(images) = images.as_mut() else {
        return;
    };
    let Some(texture_atlases) = texture_atlases else {
        return;
    };

    for (
        entity,
        config,
        controller,
        motion,
        reveal_sound,
        computed,
        layout_info,
        computed_node,
        ui_transform,
        mut runtime,
        mut debug,
    ) in &mut roots
    {
        let sections = {
            let mut reader = text_access.p0();
            collect_sections(entity, &mut reader, &runtime.hidden_styles)
        };
        let sections = {
            let mut text_colors = text_access.p1();
            snapshot_ui_sections(
                entity,
                sections,
                &mut runtime.hidden_styles,
                &mut text_colors,
                &mut background_colors,
                &mut underline_colors,
                &mut strikethrough_colors,
                &mut ui_shadows,
            )
        };

        if runtime.needs_rebuild || layout_differs(&layout_info, &runtime.cache, &texture_atlases) {
            runtime.cache = build_cache(config, sections, computed, &layout_info, &texture_atlases);
            runtime.needs_rebuild = false;
            runtime.needs_recalc_units = false;
            refresh_runtime_evaluation(
                config,
                controller,
                motion.copied(),
                &motion_accessibility,
                &mut runtime,
            );
            rebuild_ui_render_tree(
                entity,
                runtime.cache.glyphs.len(),
                &mut runtime,
                &mut commands,
            );
        } else if runtime.needs_recalc_units {
            // Config-only change: recalculate reveal timing without remapping glyphs
            // or rebuilding the render tree. This prevents flickering when adjusting
            // parameters via the pane.
            recalc_units(&mut runtime.cache, config);
            runtime.needs_recalc_units = false;
            refresh_runtime_evaluation(
                config,
                controller,
                motion.copied(),
                &motion_accessibility,
                &mut runtime,
            );
        } else {
            sync_section_colors(&sections, &mut runtime.cache);
        }

        update_ui_root_node(
            runtime.render_root,
            computed_node,
            ui_transform,
            &mut render_root_nodes,
        );

        let reduced_motion = runtime.reduced_motion_active;
        let visible_units = runtime.evaluated_visible_units;
        let visible_graphemes = runtime.evaluated_visible_graphemes;
        let reveal_units = runtime.cache.units.clone();
        update_messages(
            entity,
            visible_units,
            reveal_units.len(),
            &reveal_units,
            reveal_sound,
            &mut runtime,
            &mut messages,
        );

        for (glyph_index, glyph_entity) in runtime.glyph_entities.iter().enumerate() {
            let Ok((_, mut node, mut image_node)) = glyph_query.get_mut(*glyph_entity) else {
                continue;
            };
            let Some(glyph) = runtime.cache.glyphs.get(glyph_index) else {
                continue;
            };
            let grapheme = &runtime.cache.graphemes[glyph.primary_index];
            let section = &runtime.cache.sections[glyph.section_index];
            let mut visual = GlyphVisual::new(section.base_color);
            let glyph_visible = glyph
                .grapheme_indices
                .iter()
                .any(|index| runtime.cache.graphemes[*index].unit_index < visible_units);
            if !glyph_visible {
                visual.alpha = 0.0;
            }
            apply_effects(
                &mut visual,
                config.effects.iter().chain(runtime.markup_effects.iter()),
                glyph,
                grapheme,
                runtime.evaluated_effect_elapsed_secs,
                reduced_motion,
            );

            let scaled_size = glyph.size * visual.scale;
            node.left = Val::Px(glyph.center.x - scaled_size.x * 0.5 + visual.offset.x);
            node.top = Val::Px(glyph.center.y - scaled_size.y * 0.5 + visual.offset.y);
            node.width = Val::Px(scaled_size.x);
            node.height = Val::Px(scaled_size.y);
            image_node.rect = Some(glyph.rect);
            image_node.color = Color::LinearRgba(visual.color.with_alpha(visual.alpha));
            if let Some(handle) = image_handle_for(images, glyph.texture) {
                image_node.image = handle;
            }
        }

        debug.total_graphemes = runtime.cache.graphemes.len();
        debug.visible_graphemes = visible_graphemes;
        debug.total_units = runtime.cache.units.len();
        debug.revealed_units = visible_units;
        debug.elapsed_secs = controller.elapsed_secs;
        debug.render_glyphs = runtime.cache.glyphs.len();
        debug.effect_count = config.effects.len() + runtime.markup_effects.len();
        debug.active = true;
    }
}

pub(crate) fn apply_world_output(
    runtime_state: Res<TextAnimationRuntimeState>,
    motion_accessibility: Res<TextAnimationAccessibility>,
    mut commands: Commands,
    mut images: Option<ResMut<Assets<Image>>>,
    texture_atlases: Option<Res<Assets<TextureAtlasLayout>>>,
    mut roots: Query<
        (
            Entity,
            &TextAnimationConfig,
            &TextAnimationController,
            Option<&TextMotionPreference>,
            Option<&TextRevealSound>,
            &ComputedTextBlock,
            Ref<TextLayoutInfo>,
            &Anchor,
            &mut TextAnimationRuntime,
            &mut TextAnimationDebugState,
        ),
        (With<Text2d>, Without<Text>),
    >,
    mut glyph_query: Query<(&TextAnimationGlyph, &mut Transform, &mut Sprite)>,
    mut text_access: ParamSet<(Text2dReader, Query<&mut TextColor>)>,
    mut background_colors: Query<&mut TextBackgroundColor>,
    mut underline_colors: Query<&mut UnderlineColor>,
    mut strikethrough_colors: Query<&mut StrikethroughColor>,
    mut world_shadows: Query<&mut Text2dShadow>,
    mut messages: AnimationMessages,
) {
    if !runtime_state.active {
        return;
    }
    let Some(images) = images.as_mut() else {
        return;
    };
    let Some(texture_atlases) = texture_atlases else {
        return;
    };

    for (
        entity,
        config,
        controller,
        motion,
        reveal_sound,
        computed,
        layout_info,
        anchor,
        mut runtime,
        mut debug,
    ) in &mut roots
    {
        let sections = {
            let mut reader = text_access.p0();
            collect_sections(entity, &mut reader, &runtime.hidden_styles)
        };
        let sections = {
            let mut text_colors = text_access.p1();
            snapshot_world_sections(
                entity,
                sections,
                &mut runtime.hidden_styles,
                &mut text_colors,
                &mut background_colors,
                &mut underline_colors,
                &mut strikethrough_colors,
                &mut world_shadows,
            )
        };

        if runtime.needs_rebuild || layout_differs(&layout_info, &runtime.cache, &texture_atlases) {
            runtime.cache = build_cache(config, sections, computed, &layout_info, &texture_atlases);
            runtime.needs_rebuild = false;
            runtime.needs_recalc_units = false;
            refresh_runtime_evaluation(
                config,
                controller,
                motion.copied(),
                &motion_accessibility,
                &mut runtime,
            );
            rebuild_world_render_tree(
                entity,
                runtime.cache.glyphs.len(),
                &mut runtime,
                &mut commands,
            );
        } else if runtime.needs_recalc_units {
            recalc_units(&mut runtime.cache, config);
            runtime.needs_recalc_units = false;
            refresh_runtime_evaluation(
                config,
                controller,
                motion.copied(),
                &motion_accessibility,
                &mut runtime,
            );
        } else {
            sync_section_colors(&sections, &mut runtime.cache);
        }

        let reduced_motion = runtime.reduced_motion_active;
        let visible_units = runtime.evaluated_visible_units;
        let visible_graphemes = runtime.evaluated_visible_graphemes;
        let reveal_units = runtime.cache.units.clone();
        update_messages(
            entity,
            visible_units,
            reveal_units.len(),
            &reveal_units,
            reveal_sound,
            &mut runtime,
            &mut messages,
        );

        let size = layout_info.size;
        let top_left = (Anchor::TOP_LEFT.0 - anchor.as_vec()) * size;

        for (glyph_index, glyph_entity) in runtime.glyph_entities.iter().enumerate() {
            let Ok((_, mut transform, mut sprite)) = glyph_query.get_mut(*glyph_entity) else {
                continue;
            };
            let Some(glyph) = runtime.cache.glyphs.get(glyph_index) else {
                continue;
            };
            let grapheme = &runtime.cache.graphemes[glyph.primary_index];
            let section = &runtime.cache.sections[glyph.section_index];
            let mut visual = GlyphVisual::new(section.base_color);
            let glyph_visible = glyph
                .grapheme_indices
                .iter()
                .any(|index| runtime.cache.graphemes[*index].unit_index < visible_units);
            if !glyph_visible {
                visual.alpha = 0.0;
            }
            apply_effects(
                &mut visual,
                config.effects.iter().chain(runtime.markup_effects.iter()),
                glyph,
                grapheme,
                runtime.evaluated_effect_elapsed_secs,
                reduced_motion,
            );

            transform.translation = Vec3::new(
                top_left.x + glyph.center.x + visual.offset.x,
                -(top_left.y + glyph.center.y + visual.offset.y),
                0.0,
            );
            sprite.rect = Some(glyph.rect);
            sprite.color = Color::LinearRgba(visual.color.with_alpha(visual.alpha));
            sprite.custom_size = Some(glyph.size * visual.scale);
            if let Some(handle) = image_handle_for(images, glyph.texture) {
                sprite.image = handle;
            }
        }

        debug.total_graphemes = runtime.cache.graphemes.len();
        debug.visible_graphemes = visible_graphemes;
        debug.total_units = runtime.cache.units.len();
        debug.revealed_units = visible_units;
        debug.elapsed_secs = controller.elapsed_secs;
        debug.render_glyphs = runtime.cache.glyphs.len();
        debug.effect_count = config.effects.len() + runtime.markup_effects.len();
        debug.active = true;
    }
}

fn snapshot_ui_sections(
    root: Entity,
    sections: Vec<SectionSnapshot>,
    hidden_styles: &mut HiddenStyleCache,
    text_colors: &mut Query<&mut TextColor>,
    background_colors: &mut Query<&mut TextBackgroundColor>,
    underline_colors: &mut Query<&mut UnderlineColor>,
    strikethrough_colors: &mut Query<&mut StrikethroughColor>,
    ui_shadows: &mut Query<&mut TextShadow>,
) -> Vec<SectionSnapshot> {
    hide_sections(
        &sections,
        hidden_styles,
        text_colors,
        background_colors,
        underline_colors,
        strikethrough_colors,
    );
    if let Ok(mut shadow) = ui_shadows.get_mut(root) {
        if shadow.color.alpha() > 0.0 || hidden_styles.ui_shadow.is_none() {
            hidden_styles.ui_shadow = Some(*shadow);
        }
        if shadow.color.alpha() > 0.0 {
            shadow.color = shadow.color.with_alpha(0.0);
        }
    }
    sections
}

fn snapshot_world_sections(
    root: Entity,
    sections: Vec<SectionSnapshot>,
    hidden_styles: &mut HiddenStyleCache,
    text_colors: &mut Query<&mut TextColor>,
    background_colors: &mut Query<&mut TextBackgroundColor>,
    underline_colors: &mut Query<&mut UnderlineColor>,
    strikethrough_colors: &mut Query<&mut StrikethroughColor>,
    world_shadows: &mut Query<&mut Text2dShadow>,
) -> Vec<SectionSnapshot> {
    hide_sections(
        &sections,
        hidden_styles,
        text_colors,
        background_colors,
        underline_colors,
        strikethrough_colors,
    );
    if let Ok(mut shadow) = world_shadows.get_mut(root) {
        if shadow.color.alpha() > 0.0 || hidden_styles.world_shadow.is_none() {
            hidden_styles.world_shadow = Some(*shadow);
        }
        if shadow.color.alpha() > 0.0 {
            shadow.color = shadow.color.with_alpha(0.0);
        }
    }
    sections
}

fn apply_markup_sections<R: TextRoot>(
    root: Entity,
    sections: &[String],
    writer: &mut TextWriter<R>,
) {
    if sections.is_empty() {
        return;
    }

    for (index, clean_text) in sections.iter().enumerate() {
        let Some(mut current_text) = writer.get_text(root, index) else {
            break;
        };
        if current_text.as_str() != clean_text {
            *current_text = clean_text.clone();
        }
    }
}

fn collect_sections<R: TextRoot>(
    root: Entity,
    reader: &mut TextReader<R>,
    hidden_styles: &HiddenStyleCache,
) -> Vec<SectionSnapshot> {
    let mut snapshots = Vec::new();

    for (entity, _depth, text, font, color, _line_height) in reader.iter(root) {
        let actual_color = if color.alpha() > 0.0 {
            color
        } else {
            hidden_styles
                .sections
                .iter()
                .find(|(section_entity, _)| *section_entity == entity)
                .map(|(_, style)| style.text_color)
                .unwrap_or(color)
        };

        snapshots.push(SectionSnapshot {
            entity,
            text: text.to_string(),
            font: font.clone(),
            base_color: actual_color,
        });
    }

    snapshots
}

fn hide_sections(
    sections: &[SectionSnapshot],
    hidden_styles: &mut HiddenStyleCache,
    text_colors: &mut Query<&mut TextColor>,
    background_colors: &mut Query<&mut TextBackgroundColor>,
    underline_colors: &mut Query<&mut UnderlineColor>,
    strikethrough_colors: &mut Query<&mut StrikethroughColor>,
) {
    let previous = hidden_styles.sections.clone();
    hidden_styles.sections.clear();

    for section in sections {
        let previous_style = previous
            .iter()
            .find(|(entity, _)| *entity == section.entity)
            .map(|(_, style)| style.clone());

        let cached = CachedSectionStyle {
            text_color: section.base_color,
            background_color: background_colors
                .get(section.entity)
                .ok()
                .map(|background| match &previous_style {
                    Some(previous_style) if background.0.alpha() <= 0.0 => previous_style
                        .background_color
                        .unwrap_or(TextBackgroundColor(background.0)),
                    _ => TextBackgroundColor(background.0),
                }),
            underline_color: underline_colors.get(section.entity).ok().map(|underline| {
                match &previous_style {
                    Some(previous_style) if underline.0.alpha() <= 0.0 => previous_style
                        .underline_color
                        .unwrap_or(UnderlineColor(underline.0)),
                    _ => UnderlineColor(underline.0),
                }
            }),
            strikethrough_color: strikethrough_colors.get(section.entity).ok().map(
                |strikethrough| match &previous_style {
                    Some(previous_style) if strikethrough.0.alpha() <= 0.0 => previous_style
                        .strikethrough_color
                        .unwrap_or(StrikethroughColor(strikethrough.0)),
                    _ => StrikethroughColor(strikethrough.0),
                },
            ),
        };
        hidden_styles.sections.push((
            section.entity,
            CachedSectionStyle {
                background_color: cached.background_color.or(previous_style
                    .as_ref()
                    .and_then(|style| style.background_color)),
                underline_color: cached.underline_color.or(previous_style
                    .as_ref()
                    .and_then(|style| style.underline_color)),
                strikethrough_color: cached.strikethrough_color.or(previous_style
                    .as_ref()
                    .and_then(|style| style.strikethrough_color)),
                text_color: cached.text_color,
            },
        ));

        if let Ok(mut text_color) = text_colors.get_mut(section.entity)
            && text_color.0.alpha() > 0.0
        {
            text_color.0 = Color::LinearRgba(LinearRgba::from(section.base_color).with_alpha(0.0));
        }
        if let Ok(mut background) = background_colors.get_mut(section.entity)
            && background.0.alpha() > 0.0
        {
            background.0 = background.0.with_alpha(0.0);
        }
        if let Ok(mut underline) = underline_colors.get_mut(section.entity)
            && underline.0.alpha() > 0.0
        {
            underline.0 = underline.0.with_alpha(0.0);
        }
        if let Ok(mut strikethrough) = strikethrough_colors.get_mut(section.entity)
            && strikethrough.0.alpha() > 0.0
        {
            strikethrough.0 = strikethrough.0.with_alpha(0.0);
        }
    }
}

fn sync_section_colors(sections: &[SectionSnapshot], cache: &mut TextAnimationCache) {
    for section in sections {
        if let Some(cached) = cache
            .sections
            .iter_mut()
            .find(|cached| cached.entity == section.entity)
        {
            cached.base_color = section.base_color;
        }
    }
}

fn rebuild_ui_render_tree(
    _root_entity: Entity,
    glyph_count: usize,
    runtime: &mut TextAnimationRuntime,
    commands: &mut Commands,
) {
    let render_root = match runtime.render_root {
        Some(root) => root,
        None => {
            let root = commands
                .spawn((
                    Name::new("Text Animation UI Overlay"),
                    TextAnimationRenderRoot,
                    Node {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                ))
                .id();
            runtime.render_root = Some(root);
            root
        }
    };

    let current_count = runtime.glyph_entities.len();

    // Despawn excess entities (only when we have more than needed)
    if glyph_count < current_count {
        for entity in runtime.glyph_entities.drain(glyph_count..) {
            commands.entity(entity).despawn();
        }
    }

    // Spawn missing entities
    for glyph_index in current_count.min(glyph_count)..glyph_count {
        let glyph = commands
            .spawn((
                Name::new(format!("Animated Glyph {glyph_index}")),
                TextAnimationGlyph,
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ImageNode::default(),
            ))
            .id();
        commands.entity(render_root).add_child(glyph);
        runtime.glyph_entities.push(glyph);
    }
    runtime.root_kind = Some(RootKind::Ui);
}

fn rebuild_world_render_tree(
    root_entity: Entity,
    glyph_count: usize,
    runtime: &mut TextAnimationRuntime,
    commands: &mut Commands,
) {
    let render_root = match runtime.render_root {
        Some(root) => root,
        None => {
            let root = commands
                .spawn((
                    Name::new("Text Animation World Overlay"),
                    TextAnimationRenderRoot,
                    Transform::default(),
                    Visibility::Visible,
                ))
                .id();
            commands.entity(root_entity).add_child(root);
            runtime.render_root = Some(root);
            root
        }
    };

    let current_count = runtime.glyph_entities.len();

    // Despawn excess entities (only when we have more than needed)
    if glyph_count < current_count {
        for entity in runtime.glyph_entities.drain(glyph_count..) {
            commands.entity(entity).despawn();
        }
    }

    // Spawn missing entities
    for glyph_index in current_count.min(glyph_count)..glyph_count {
        let glyph = commands
            .spawn((
                Name::new(format!("Animated Glyph {glyph_index}")),
                TextAnimationGlyph,
                Sprite::default(),
                Transform::default(),
            ))
            .id();
        commands.entity(render_root).add_child(glyph);
        runtime.glyph_entities.push(glyph);
    }
    runtime.root_kind = Some(RootKind::World);
}

fn update_ui_root_node(
    render_root: Option<Entity>,
    computed_node: &ComputedNode,
    ui_transform: &UiGlobalTransform,
    render_root_nodes: &mut Query<
        &mut Node,
        (With<TextAnimationRenderRoot>, Without<TextAnimationGlyph>),
    >,
) {
    let Some(render_root) = render_root else {
        return;
    };
    let Ok(mut node) = render_root_nodes.get_mut(render_root) else {
        return;
    };
    let top_left = ui_transform
        .affine()
        .transform_point2(-0.5 * computed_node.size());
    node.left = Val::Px(top_left.x);
    node.top = Val::Px(top_left.y);
    node.width = Val::Px(computed_node.size().x);
    node.height = Val::Px(computed_node.size().y);
}

fn restore_source_styles(
    root: Entity,
    hidden_styles: &HiddenStyleCache,
    text_colors: &mut Query<&mut TextColor>,
    background_colors: &mut Query<&mut TextBackgroundColor>,
    underline_colors: &mut Query<&mut UnderlineColor>,
    strikethrough_colors: &mut Query<&mut StrikethroughColor>,
    ui_shadows: &mut Query<&mut TextShadow>,
    world_shadows: &mut Query<&mut Text2dShadow>,
) {
    for (entity, cached) in &hidden_styles.sections {
        if let Ok(mut color) = text_colors.get_mut(*entity) {
            color.0 = cached.text_color;
        }
        if let Some(background) = cached.background_color
            && let Ok(mut actual) = background_colors.get_mut(*entity)
        {
            *actual = background;
        }
        if let Some(underline) = cached.underline_color
            && let Ok(mut actual) = underline_colors.get_mut(*entity)
        {
            *actual = underline;
        }
        if let Some(strikethrough) = cached.strikethrough_color
            && let Ok(mut actual) = strikethrough_colors.get_mut(*entity)
        {
            *actual = strikethrough;
        }
    }
    if let Some(shadow) = hidden_styles.ui_shadow
        && let Ok(mut actual) = ui_shadows.get_mut(root)
    {
        *actual = shadow;
    }
    if let Some(shadow) = hidden_styles.world_shadow
        && let Ok(mut actual) = world_shadows.get_mut(root)
    {
        *actual = shadow;
    }
}

fn motion_reduced(
    preference: Option<TextMotionPreference>,
    accessibility: &TextAnimationAccessibility,
) -> bool {
    match preference.unwrap_or_default() {
        TextMotionPreference::Inherit => accessibility.reduced_motion,
        TextMotionPreference::Full => false,
        TextMotionPreference::Reduced => true,
    }
}

fn refresh_runtime_evaluation(
    config: &TextAnimationConfig,
    controller: &TextAnimationController,
    motion: Option<TextMotionPreference>,
    accessibility: &TextAnimationAccessibility,
    runtime: &mut TextAnimationRuntime,
) {
    runtime.reduced_motion_active = motion_reduced(motion, accessibility);
    runtime.evaluated_effect_elapsed_secs = effect_time(config, controller, &runtime.cache);
    runtime.evaluated_visible_units = runtime
        .cache
        .visible_units(runtime.evaluated_effect_elapsed_secs);
    runtime.evaluated_visible_graphemes = runtime
        .cache
        .visible_graphemes(runtime.evaluated_visible_units);
}

fn update_messages(
    entity: Entity,
    visible_units: usize,
    total_units: usize,
    reveal_units: &[RevealUnit],
    reveal_sound: Option<&TextRevealSound>,
    runtime: &mut TextAnimationRuntime,
    messages: &mut AnimationMessages,
) {
    if visible_units > 0 && !runtime.sent_started {
        messages.started.write(TextAnimationStarted { entity });
        runtime.sent_started = true;
    }

    if visible_units != runtime.last_visible_units {
        messages.checkpoints.write(TextRevealCheckpoint {
            entity,
            revealed_units: visible_units,
            total_units,
        });
        if visible_units > runtime.last_visible_units {
            messages.reveal_advanced.write(TextRevealAdvanced {
                entity,
                start_unit: runtime.last_visible_units,
                end_unit: visible_units,
                labels: reveal_units[runtime.last_visible_units..visible_units]
                    .iter()
                    .map(|unit| unit.label.clone())
                    .collect(),
            });
            if let Some(reveal_sound) = reveal_sound {
                let step = reveal_sound.every_n_units.max(1);
                for (unit_index, unit) in reveal_units
                    .iter()
                    .enumerate()
                    .take(visible_units)
                    .skip(runtime.last_visible_units)
                {
                    if reveal_sound.skip_whitespace && unit.label.chars().all(char::is_whitespace) {
                        continue;
                    }
                    if (unit_index + 1) % step != 0 {
                        continue;
                    }
                    messages.reveal_sound.write(TextRevealSoundRequested {
                        entity,
                        cue_id: reveal_sound.cue_id.clone(),
                        unit_index,
                        label: unit.label.clone(),
                    });
                }
            }
        }
        runtime.last_visible_units = visible_units;
    }

    if visible_units == total_units && !runtime.sent_completed && total_units > 0 {
        messages.completed.write(TextAnimationCompleted { entity });
        runtime.sent_completed = true;
    }

    if runtime.pending_loops > 0 {
        messages.looped.write(TextAnimationLoopFinished {
            entity,
            completed_loops: runtime.pending_loops,
        });
        runtime.pending_loops = 0;
    }
}

fn effect_time(
    config: &TextAnimationConfig,
    controller: &TextAnimationController,
    cache: &TextAnimationCache,
) -> f32 {
    if config.continue_effects_after_reveal {
        controller.elapsed_secs
    } else {
        controller.elapsed_secs.min(cache.total_duration_secs)
    }
}

fn layout_differs(
    layout_info: &TextLayoutInfo,
    cache: &TextAnimationCache,
    texture_atlases: &Assets<TextureAtlasLayout>,
) -> bool {
    if layout_info.glyphs.len() != cache.glyphs.len() {
        return true;
    }

    for (layout_glyph, cached) in layout_info.glyphs.iter().zip(&cache.glyphs) {
        let Some(rect) = texture_atlases
            .get(layout_glyph.atlas_info.texture_atlas)
            .and_then(|layout| {
                layout
                    .textures
                    .get(layout_glyph.atlas_info.location.glyph_index)
            })
            .map(|rect| rect.as_rect())
        else {
            return true;
        };

        if layout_glyph.span_index != cached.section_index
            || layout_glyph.atlas_info.texture != cached.texture
            || layout_glyph.position != cached.center
            || layout_glyph.size != cached.size
            || rect != cached.rect
        {
            return true;
        }
    }

    false
}

pub(crate) fn image_handle_for(
    images: &mut Assets<Image>,
    id: AssetId<Image>,
) -> Option<Handle<Image>> {
    match id {
        AssetId::Index { .. } => images.get_strong_handle(id),
        AssetId::Uuid { uuid } => Some(Handle::Uuid(uuid, PhantomData)),
    }
}
