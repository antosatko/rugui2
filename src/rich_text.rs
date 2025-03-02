use std::{
    cmp::Ordering,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
};

use ropey::Rope;
use swash::text::{
    cluster::{Parser, Token},
    Script,
};

use crate::{
    colors::Colors,
    styles::{Portion, StyleComponent, TextAlign, Value},
    text::{
        select_pref_font, FontIdx, GlyphKey, PhysicalChar, Rect, TextProccesor, DEFAULT_FONT_SIZE,
    },
};

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum GlyphFlags {
    Bold = 1 << 0,
    Italic = 1 << 1,
}

impl GlyphFlags {
    pub fn section_styles_to_flags(styles: &SectionStylesInstance) -> u8 {
        let mut result = 0;
        if styles.bold {
            result |= GlyphFlags::Bold as u8;
        }
        if styles.italic {
            result |= GlyphFlags::Italic as u8;
        }
        result
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TextStylesInstance {
    pub align: f32,
    pub line_offset: f32,
    pub paragraph_offset: f32,
    pub left_to_right: bool,
    pub wrap_on_overflow: bool,
}

#[derive(Debug, Clone)]
pub struct TextStyles {
    pub align: StyleComponent<TextAlign>,
    pub line_offset: StyleComponent<Portion>,
    pub paragraph_offset: StyleComponent<Portion>,
    pub wrap_on_overflow: StyleComponent<bool>,
}

#[derive(Debug, Copy, Clone)]
pub struct SectionStylesInstance {
    pub left_pad: f32,
    pub right_pad: f32,
    pub color: [f32; 4],
    pub font_size: f32,
    pub font: FontIdx,
    pub bold: bool,
    pub italic: bool,
}
#[derive(Debug, Clone)]
pub struct SectionStyles {
    pub left_pad: StyleComponent<Value>,
    pub right_pad: StyleComponent<Value>,
    pub color: StyleComponent<Colors>,
    pub font_size: StyleComponent<Value>,
    pub font: FontIdx,
    pub bold: StyleComponent<bool>,
    pub italic: StyleComponent<bool>,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub(crate)instance_data: TextStylesInstance,
    pub styles: TextStyles,
    pub shape: ShapeStorages,
    pub sections: Vec<TextSection>,
}

#[derive(Debug, Clone)]
pub struct TextSection {
    pub(crate)instance_data: SectionStylesInstance,
    pub styles: SectionStyles,
    pub text: Rope,
    pub kind: SectionKinds,
}

#[derive(Debug, Copy, Clone)]
pub enum SectionKinds {
    Section,
    NewLine,
    NewParagraph,
}

#[derive(Debug, Clone)]
pub enum ShapeStorages {
    External,
    Internal(TextShape),
    Shared(Rc<RwLock<TextShape>>),
    ThreadSync(Arc<Mutex<TextShape>>),
}

#[derive(Debug, Clone)]
pub struct TextShape {
    pub lines: Vec<PhysicalLine>,
    pub bounds: Rect,
}

#[derive(Debug, Clone)]
pub struct PhysicalLine {
    pub line_index: usize,
    pub bounds: Rect,
    pub chars: Vec<PhysicalChar>,
    pub height: f32,
    pub color: [f32; 4],
}

impl Text {
    pub const DEFAULT_STYLES: TextStyles = TextStyles {
        align: StyleComponent::new(TextAlign::Left),
        line_offset: StyleComponent::new(Portion::Full),
        paragraph_offset: StyleComponent::new(Portion::Mul(1.75)),
        wrap_on_overflow: StyleComponent::new(false),
    };

    pub fn from_str(text: &str) -> Self {
        Self {
            instance_data: TextStylesInstance::default(),
            shape: ShapeStorages::Internal(TextShape::default()),
            styles: Self::DEFAULT_STYLES,
            sections: vec![TextSection::new(text)],
        }
    }

    pub fn new() -> Self {
        Self {
            instance_data: TextStylesInstance::default(),
            shape: ShapeStorages::Internal(TextShape::default()),
            styles: Self::DEFAULT_STYLES,
            sections: Vec::new(),
        }
    }

    pub fn procces(
        &mut self,
        ctx: &mut TextProccesor,
        shape: Option<&mut TextShape>,
        bounds: Rect,
    ) {
        fn endl(
            shape: &mut TextShape,
            styles: &TextStylesInstance,
            line_index: usize,
            bounds: Rect,
        ) -> f32 {
            let first_in_line = shape
                .lines
                .iter()
                .enumerate()
                .rev()
                .take_while(|(_, line)| line.line_index == line_index)
                .map(|(i, _)| i)
                .last()
                .unwrap_or(0);
            let lines_slice = &mut shape.lines[first_in_line..];
            let total_width: f32 = lines_slice.iter().map(|l| l.bounds.width).sum();
            let alignment = (-total_width + bounds.width) * styles.align;
            let max_height = lines_slice
                .iter()
                .map(|l| l.height)
                .max_by(|l, r| {
                    if l > r {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    }
                })
                .unwrap_or(1.0);
            for line_section in shape
                .lines
                .iter_mut()
                .rev()
                .take_while(|l| l.line_index == line_index)
            {
                line_section.height = max_height;
                line_section.bounds.left += alignment;
            }
            max_height
        }
        self.with_shape_mut(shape, |shape, styles, sections| {
            shape.lines = Vec::new();
            shape.bounds = bounds;
            let mut char_idx = 0;
            let mut line_index = 0;
            let mut top_pos = bounds.top;
            let mut left_pos = bounds.left;
            for section in sections {
                let flags = GlyphFlags::section_styles_to_flags(&section.instance_data);
                let font_size = section.instance_data.font_size;
                (left_pos, top_pos) = match section.kind {
                    SectionKinds::Section => {
                        (left_pos.max(section.instance_data.left_pad + bounds.left), top_pos)
                    }
                    SectionKinds::NewLine => {
                        let max_height = endl(shape, styles, line_index, bounds);
                        line_index += 1;
                        (
                            section.instance_data.left_pad + bounds.left,
                            top_pos + max_height + styles.line_offset,
                        )
                    }
                    SectionKinds::NewParagraph => {
                        let max_height = endl(shape, styles, line_index, bounds);
                        line_index += 1;
                        (
                            section.instance_data.left_pad + bounds.left,
                            top_pos + max_height + styles.line_offset + styles.paragraph_offset,
                        )
                    }
                };
                let mut phys_line = PhysicalLine {
                    line_index,
                    chars: Vec::new(),
                    color: section.instance_data.color,
                    height: section.instance_data.font_size,
                    bounds: Rect {
                        left: left_pos,
                        top: top_pos,
                        width: 0.0,
                        height: section.instance_data.font_size,
                    },
                };
                for line in section.text.lines() {
                    for chunk in line.chunks() {
                        let mut parser = Parser::new(
                            Script::Latin,
                            chunk.char_indices().map(|(i, ch)| Token {
                                // The character
                                ch,
                                // Offset of the character in code units
                                offset: i as u32,
                                // Length of the character in code units
                                len: ch.len_utf8() as u8,
                                // Character information
                                info: ch.into(),
                                // Pass through user data
                                data: 0,
                            }),
                        );

                        while parser.next(&mut ctx.cluster) {
                            let i = match select_pref_font(
                                &ctx.fonts,
                                section.instance_data.font.raw() as usize,
                                &mut ctx.cluster,
                            ) {
                                Some(i) => i,
                                None => continue,
                            };
                            let font_key = ctx.fonts[i].key;
                            let mut shaper = ctx
                                .shape_ctx
                                .builder(ctx.fonts[i].as_ref())
                                .size(font_size)
                                .build();

                            shaper.add_cluster(&ctx.cluster);
                            shaper.shape_with(|cluster| {
                                for glyph in cluster.glyphs {
                                    let glyph_key = GlyphKey {
                                        font_idx: FontIdx(i as u16),
                                        font_key,
                                        glyph_id: glyph.id,
                                        font_size: font_size.round() as u32,
                                        flags,
                                    };
                                    let phys_char = PhysicalChar {
                                        idx: char_idx,
                                        glyph_key,
                                        width: glyph.advance,
                                    };
                                    phys_line.chars.push(phys_char);
                                    char_idx += 1;
                                    left_pos += glyph.advance;
                                    phys_line.bounds.width += glyph.advance;
                                }
                            });
                        }
                    }
                }
                phys_line.bounds.left /= 2.0;
                shape.lines.push(phys_line);
            }
            endl(shape, styles, line_index, bounds);
        });
    }

    pub fn with_shape_mut(
        &mut self,
        shape: Option<&mut TextShape>,
        cb: impl FnOnce(&mut TextShape, &TextStylesInstance, &mut Vec<TextSection>),
    ) {
        match shape {
            Some(s) => cb(s, &self.instance_data, &mut self.sections),
            None => match &mut self.shape {
                ShapeStorages::External => (),
                ShapeStorages::Internal(s) => cb(s, &self.instance_data, &mut self.sections),
                ShapeStorages::Shared(s) => match s.write() {
                    Ok(mut s) => cb(&mut s, &self.instance_data, &mut self.sections),
                    Err(_) => (),
                },
                ShapeStorages::ThreadSync(s) => match s.lock() {
                    Ok(mut s) => cb(&mut s, &self.instance_data, &mut self.sections),
                    Err(_) => (),
                },
            },
        };
    }
    pub fn with_shape(
        &self,
        shape: Option<&mut TextShape>,
        cb: impl FnOnce(&TextShape, &TextStylesInstance, &Vec<TextSection>),
    ) {
        match shape {
            Some(s) => cb(s, &self.instance_data, &self.sections),
            None => match &self.shape {
                ShapeStorages::External => return,
                ShapeStorages::Internal(s) => cb(s, &self.instance_data, &self.sections),
                ShapeStorages::Shared(s) => match s.write() {
                    Ok(s) => cb(&s, &self.instance_data, &self.sections),
                    Err(_) => return,
                },
                ShapeStorages::ThreadSync(s) => match s.lock() {
                    Ok(s) => cb(&s, &self.instance_data, &self.sections),
                    Err(_) => return,
                },
            },
        };
    }
}

impl Default for TextStylesInstance {
    fn default() -> Self {
        Self {
            align: 0.0,
            line_offset: 0.0,
            paragraph_offset: DEFAULT_FONT_SIZE * 0.75,
            left_to_right: true,
            wrap_on_overflow: true,
        }
    }
}

impl Default for SectionStylesInstance {
    fn default() -> Self {
        SectionStylesInstance {
            left_pad: 0.0,
            right_pad: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            font_size: DEFAULT_FONT_SIZE,
            font: FontIdx(0),
            bold: false,
            italic: false,
        }
    }
}

impl Default for TextShape {
    fn default() -> Self {
        TextShape {
            lines: Vec::new(),
            bounds: Rect::ZERO,
        }
    }
}

impl TextSection {
    pub const DEFAULT_STYLES: SectionStyles = SectionStyles {
        color: StyleComponent::new(Colors::FRgba(1.0, 1.0, 1.0, 1.0)),
        left_pad: StyleComponent::new(Value::Zero),
        right_pad: StyleComponent::new(Value::Zero),
        font_size: StyleComponent::new(Value::Px(DEFAULT_FONT_SIZE)),
        font: FontIdx(0),
        bold: StyleComponent::new(false),
        italic: StyleComponent::new(false),
    };
    
    pub fn new(text: &str) -> Self {
        Self {
            instance_data: SectionStylesInstance::default(),
            text: Rope::from_str(text),
            kind: SectionKinds::Section,
            styles: Self::DEFAULT_STYLES
        }
    }
}
