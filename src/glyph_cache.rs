use bevy::asset::AssetId;
use bevy::image::{Image, TextureAtlasLayout};
use bevy::math::{Rect, Vec2};
use bevy::prelude::*;
use bevy::text::{ComputedTextBlock, PositionedGlyph, TextFont, TextLayoutInfo};
use unicode_segmentation::UnicodeSegmentation;

use crate::config::{PunctuationDelayConfig, RevealMode, TextAnimationConfig};

#[derive(Debug, Clone)]
pub(crate) struct SectionSnapshot {
    pub entity: Entity,
    pub text: String,
    #[allow(dead_code)]
    pub font: TextFont,
    pub base_color: Color,
}

#[derive(Debug, Clone)]
pub(crate) struct GraphemeEntry {
    pub index: usize,
    pub section_index: usize,
    #[allow(dead_code)]
    pub section_entity: Entity,
    pub text: String,
    pub byte_range: std::ops::Range<usize>,
    pub is_whitespace: bool,
    pub line_index: usize,
    pub word_index: Option<usize>,
    pub unit_index: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct GlyphEntry {
    pub primary_index: usize,
    pub grapheme_indices: Vec<usize>,
    pub section_index: usize,
    pub texture: AssetId<Image>,
    pub rect: Rect,
    pub center: Vec2,
    pub size: Vec2,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct RevealUnit {
    pub grapheme_range: std::ops::Range<usize>,
    pub reveal_time_secs: f32,
    pub label: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TextAnimationCache {
    pub sections: Vec<SectionSnapshot>,
    pub graphemes: Vec<GraphemeEntry>,
    pub glyphs: Vec<GlyphEntry>,
    pub units: Vec<RevealUnit>,
    pub total_duration_secs: f32,
}

impl TextAnimationCache {
    pub fn visible_units(&self, elapsed_secs: f32) -> usize {
        if self.units.is_empty() {
            return 0;
        }
        self.units
            .partition_point(|unit| unit.reveal_time_secs <= elapsed_secs)
    }

    pub fn visible_graphemes(&self, visible_units: usize) -> usize {
        self.graphemes
            .iter()
            .filter(|grapheme| grapheme.unit_index < visible_units)
            .count()
    }
}

pub(crate) fn build_cache(
    config: &TextAnimationConfig,
    sections: Vec<SectionSnapshot>,
    computed: &ComputedTextBlock,
    layout_info: &TextLayoutInfo,
    texture_atlases: &Assets<TextureAtlasLayout>,
) -> TextAnimationCache {
    let mut graphemes = segment_graphemes(&sections);
    let glyphs = map_glyphs(
        &sections,
        &graphemes,
        computed,
        layout_info,
        texture_atlases,
    );
    let units = build_units(config, &mut graphemes);
    let total_duration_secs = units
        .last()
        .map(|unit| unit.reveal_time_secs)
        .unwrap_or(0.0);
    TextAnimationCache {
        sections,
        graphemes,
        glyphs,
        units,
        total_duration_secs,
    }
}

/// Recalculate only reveal unit timing from existing graphemes.
/// Used when animation config changes (speed, effects) without text content changing.
/// Much cheaper than a full `build_cache` because it skips grapheme segmentation
/// and glyph remapping.
pub(crate) fn recalc_units(cache: &mut TextAnimationCache, config: &TextAnimationConfig) {
    let units = build_units(config, &mut cache.graphemes);
    cache.total_duration_secs = units
        .last()
        .map(|unit| unit.reveal_time_secs)
        .unwrap_or(0.0);
    cache.units = units;
}

fn segment_graphemes(sections: &[SectionSnapshot]) -> Vec<GraphemeEntry> {
    let mut graphemes = Vec::new();
    let mut global_line = 0usize;
    let mut global_word = None;
    let mut next_word = 0usize;

    for (section_index, section) in sections.iter().enumerate() {
        for (byte_start, grapheme_text) in section.text.grapheme_indices(true) {
            let byte_end = byte_start + grapheme_text.len();
            let is_whitespace = grapheme_text.chars().all(char::is_whitespace);
            let current_word = if is_whitespace {
                global_word
            } else if let Some(word) = global_word {
                Some(word)
            } else {
                let word = next_word;
                next_word += 1;
                global_word = Some(word);
                Some(word)
            };

            let entry = GraphemeEntry {
                index: graphemes.len(),
                section_index,
                section_entity: section.entity,
                text: grapheme_text.to_string(),
                byte_range: byte_start..byte_end,
                is_whitespace,
                line_index: global_line,
                word_index: current_word,
                unit_index: 0,
            };
            graphemes.push(entry);

            if grapheme_text == "\n" || grapheme_text == "\r\n" {
                global_line += 1;
                global_word = None;
            } else if is_whitespace {
                global_word = None;
            }
        }
    }

    graphemes
}

fn map_glyphs(
    sections: &[SectionSnapshot],
    graphemes: &[GraphemeEntry],
    _computed: &ComputedTextBlock,
    layout_info: &TextLayoutInfo,
    texture_atlases: &Assets<TextureAtlasLayout>,
) -> Vec<GlyphEntry> {
    let mut section_ranges: Vec<Vec<(usize, std::ops::Range<usize>)>> =
        vec![Vec::new(); sections.len()];
    for grapheme in graphemes {
        section_ranges[grapheme.section_index].push((grapheme.index, grapheme.byte_range.clone()));
    }

    let mut glyphs = Vec::with_capacity(layout_info.glyphs.len());
    let mut section_cursors = vec![0usize; sections.len()];

    for PositionedGlyph {
        position,
        size,
        atlas_info,
        span_index,
        byte_index,
        byte_length,
        ..
    } in &layout_info.glyphs
    {
        let section_index = *span_index;
        if section_index >= section_ranges.len() {
            continue;
        }

        let glyph_range = *byte_index..(*byte_index + *byte_length);
        let ranges = &section_ranges[section_index];
        let mut cursor = section_cursors[section_index];
        while cursor < ranges.len() && ranges[cursor].1.end <= glyph_range.start {
            cursor += 1;
        }

        let mut grapheme_indices = Vec::new();
        let mut probe = cursor;
        while probe < ranges.len() && ranges[probe].1.start < glyph_range.end {
            if overlaps(&ranges[probe].1, &glyph_range) {
                grapheme_indices.push(ranges[probe].0);
            }
            probe += 1;
        }

        if grapheme_indices.is_empty() && cursor < ranges.len() {
            grapheme_indices.push(ranges[cursor].0);
        }
        if grapheme_indices.is_empty() {
            continue;
        }

        let Some(rect) = texture_atlases
            .get(atlas_info.texture_atlas)
            .and_then(|layout| layout.textures.get(atlas_info.location.glyph_index))
            .map(|rect| rect.as_rect())
        else {
            continue;
        };

        section_cursors[section_index] = cursor;
        glyphs.push(GlyphEntry {
            primary_index: grapheme_indices[0],
            grapheme_indices,
            section_index,
            texture: atlas_info.texture,
            rect,
            center: *position,
            size: *size,
        });
    }

    glyphs
}

fn build_units(config: &TextAnimationConfig, graphemes: &mut [GraphemeEntry]) -> Vec<RevealUnit> {
    if graphemes.is_empty() {
        return Vec::new();
    }

    if !config.typewriter.enabled {
        for grapheme in graphemes.iter_mut() {
            grapheme.unit_index = 0;
        }
        return vec![RevealUnit {
            grapheme_range: 0..graphemes.len(),
            reveal_time_secs: 0.0,
            label: graphemes.iter().map(|g| g.text.as_str()).collect(),
        }];
    }

    match config.typewriter.reveal_mode {
        RevealMode::Instant => {
            for grapheme in graphemes.iter_mut() {
                grapheme.unit_index = 0;
            }
            vec![RevealUnit {
                grapheme_range: 0..graphemes.len(),
                reveal_time_secs: 0.0,
                label: graphemes.iter().map(|g| g.text.as_str()).collect(),
            }]
        }
        RevealMode::Grapheme => grapheme_units(graphemes, &config.typewriter),
        RevealMode::Word => word_units(graphemes, &config.typewriter),
        RevealMode::Line => line_units(graphemes, &config.typewriter),
    }
}

fn grapheme_units(
    graphemes: &mut [GraphemeEntry],
    config: &crate::config::TypewriterConfig,
) -> Vec<RevealUnit> {
    let mut units = Vec::with_capacity(graphemes.len());
    let mut current_time = 0.0;
    for (index, grapheme) in graphemes.iter_mut().enumerate() {
        grapheme.unit_index = index;
        units.push(RevealUnit {
            grapheme_range: index..index + 1,
            reveal_time_secs: current_time,
            label: grapheme.text.clone(),
        });
        current_time += step_duration(config.units_per_second)
            + punctuation_delay(&grapheme.text, &config.punctuation_delay);
    }
    units
}

fn word_units(
    graphemes: &mut [GraphemeEntry],
    config: &crate::config::TypewriterConfig,
) -> Vec<RevealUnit> {
    let mut units = Vec::new();
    let mut unit_start = 0usize;
    let mut current_time = 0.0;
    let mut active = false;

    for index in 0..graphemes.len() {
        let is_boundary = graphemes[index].is_whitespace && active;
        if !graphemes[index].is_whitespace && !active {
            unit_start = index;
            active = true;
        }

        if active {
            graphemes[index].unit_index = units.len();
        } else {
            graphemes[index].unit_index = units.len().saturating_sub(1);
        }

        let end_of_text = index + 1 == graphemes.len();
        let finalize = is_boundary || (active && end_of_text);
        if finalize {
            let end = if is_boundary { index } else { index + 1 };
            if unit_start < end {
                for grapheme in &mut graphemes[unit_start..end] {
                    grapheme.unit_index = units.len();
                }
                let label: String = graphemes[unit_start..end]
                    .iter()
                    .map(|g| g.text.as_str())
                    .collect();
                units.push(RevealUnit {
                    grapheme_range: unit_start..end,
                    reveal_time_secs: current_time,
                    label: label.clone(),
                });
                current_time += step_duration(config.units_per_second)
                    + punctuation_delay(last_non_whitespace(&label), &config.punctuation_delay);
            }
            active = false;
        }
    }

    if units.is_empty() {
        grapheme_units(graphemes, config)
    } else {
        units
    }
}

fn line_units(
    graphemes: &mut [GraphemeEntry],
    config: &crate::config::TypewriterConfig,
) -> Vec<RevealUnit> {
    let mut units = Vec::new();
    let mut current_time = 0.0;
    let mut start = 0usize;
    let mut current_line = graphemes.first().map(|g| g.line_index).unwrap_or(0);

    for index in 0..graphemes.len() {
        let line_changed = graphemes[index].line_index != current_line;
        if line_changed {
            finalize_line_unit(graphemes, &mut units, start, index, current_time);
            let label = unit_label(graphemes, start, index);
            current_time += step_duration(config.units_per_second)
                + punctuation_delay(last_non_whitespace(&label), &config.punctuation_delay);
            current_line = graphemes[index].line_index;
            start = index;
        }
    }

    finalize_line_unit(graphemes, &mut units, start, graphemes.len(), current_time);
    units
}

fn finalize_line_unit(
    graphemes: &mut [GraphemeEntry],
    units: &mut Vec<RevealUnit>,
    start: usize,
    end: usize,
    current_time: f32,
) {
    if start >= end {
        return;
    }
    for grapheme in &mut graphemes[start..end] {
        grapheme.unit_index = units.len();
    }
    units.push(RevealUnit {
        grapheme_range: start..end,
        reveal_time_secs: current_time,
        label: unit_label(graphemes, start, end),
    });
}

fn unit_label(graphemes: &[GraphemeEntry], start: usize, end: usize) -> String {
    graphemes[start..end]
        .iter()
        .map(|grapheme| grapheme.text.as_str())
        .collect()
}

fn punctuation_delay(text: &str, config: &PunctuationDelayConfig) -> f32 {
    match text {
        "\n" | "\r\n" => config.newline_extra_secs,
        "..." | "\u{2026}" => config.ellipsis_extra_secs,
        "." | "!" | "?" | "。" | "！" | "？" => config.sentence_extra_secs,
        "," | ";" | ":" | "，" | "、" | "；" | "：" => config.clause_extra_secs,
        _ => 0.0,
    }
}

fn last_non_whitespace(label: &str) -> &str {
    label
        .graphemes(true)
        .rev()
        .find(|grapheme| !grapheme.chars().all(char::is_whitespace))
        .unwrap_or(label)
}

fn step_duration(units_per_second: f32) -> f32 {
    1.0 / units_per_second.max(0.001)
}

fn overlaps(a: &std::ops::Range<usize>, b: &std::ops::Range<usize>) -> bool {
    a.start < b.end && b.start < a.end
}
