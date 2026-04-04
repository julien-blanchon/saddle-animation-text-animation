use unicode_segmentation::UnicodeSegmentation;

use crate::config::{
    AlphaPulseEffect, ScaleEffect, ShakeEffect, TextEffect, TextRangeSelector, WaveEffect,
};

#[derive(Debug, Default)]
pub(crate) struct ParsedMarkup {
    pub sections: Vec<String>,
    pub effects: Vec<TextEffect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkupTag {
    Wave,
    Shake,
    Rainbow,
    Alpha,
    Scale,
}

#[derive(Debug, Clone, Copy)]
struct OpenTag {
    kind: MarkupTag,
    start_grapheme: usize,
}

pub(crate) fn parse_sections(sections: &[String]) -> ParsedMarkup {
    let mut parsed = ParsedMarkup::default();
    let mut open_tags = Vec::<OpenTag>::new();
    let mut global_graphemes = 0usize;

    for section in sections {
        let mut clean = String::new();
        let mut index = 0usize;

        while index < section.len() {
            if let Some(close_index) = section[index..].find('>') {
                let candidate = &section[index..index + close_index + 1];
                if let Some((kind, closing)) = parse_tag(candidate) {
                    if closing {
                        if let Some(open_index) =
                            open_tags.iter().rposition(|entry| entry.kind == kind)
                        {
                            let open = open_tags.remove(open_index);
                            if open.start_grapheme < global_graphemes {
                                parsed.effects.push(tag_effect(
                                    kind,
                                    open.start_grapheme,
                                    global_graphemes,
                                ));
                            }
                        }
                    } else {
                        open_tags.push(OpenTag {
                            kind,
                            start_grapheme: global_graphemes,
                        });
                    }
                    index += candidate.len();
                    continue;
                }
            }

            let next_tag = section[index + 1..]
                .find('<')
                .map(|offset| index + 1 + offset)
                .unwrap_or(section.len());
            let chunk_end = if next_tag == index {
                index + section[index..].chars().next().unwrap().len_utf8()
            } else {
                next_tag
            };
            let chunk = &section[index..chunk_end];
            clean.push_str(chunk);
            global_graphemes += chunk.graphemes(true).count();
            index = chunk_end;
        }

        parsed.sections.push(clean);
    }

    for open in open_tags {
        if open.start_grapheme < global_graphemes {
            parsed
                .effects
                .push(tag_effect(open.kind, open.start_grapheme, global_graphemes));
        }
    }

    parsed
}

fn parse_tag(token: &str) -> Option<(MarkupTag, bool)> {
    let inner = token.strip_prefix('<')?.strip_suffix('>')?.trim();
    if inner.is_empty() {
        return None;
    }

    let (closing, tag_name) = if let Some(tag_name) = inner.strip_prefix('/') {
        (true, tag_name)
    } else {
        (false, inner)
    };

    let tag = match tag_name.trim().to_ascii_lowercase().as_str() {
        "wave" => MarkupTag::Wave,
        "shake" => MarkupTag::Shake,
        "rainbow" => MarkupTag::Rainbow,
        "alpha" | "pulse" => MarkupTag::Alpha,
        "scale" => MarkupTag::Scale,
        _ => return None,
    };

    Some((tag, closing))
}

fn tag_effect(kind: MarkupTag, start: usize, end: usize) -> TextEffect {
    let range = TextRangeSelector::GraphemeRange { start, end };
    match kind {
        MarkupTag::Wave => TextEffect::Wave(WaveEffect {
            range,
            ..Default::default()
        }),
        MarkupTag::Shake => TextEffect::Shake(ShakeEffect {
            range,
            ..Default::default()
        }),
        MarkupTag::Rainbow => TextEffect::Rainbow(crate::config::RainbowEffect {
            range,
            ..Default::default()
        }),
        MarkupTag::Alpha => TextEffect::AlphaPulse(AlphaPulseEffect {
            range,
            ..Default::default()
        }),
        MarkupTag::Scale => TextEffect::Scale(ScaleEffect {
            range,
            ..Default::default()
        }),
    }
}
