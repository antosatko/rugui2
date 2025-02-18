use std::{
    cmp::Ordering, rc::Rc, sync::{Arc, Mutex, RwLock}
};

use ropey::Rope;
use swash::text::{
    cluster::{Parser, Token},
    Script,
};

use crate::text::{
    select_pref_font, FontIdx, GlyphKey, PhysicalChar, Rect, TextProccesor, DEFAULT_FONT_SIZE,
};

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum GlyphFlags {
    Bold = 1 << 0,
    Italic = 1 << 1,
}

impl GlyphFlags {
    pub fn section_styles_to_flags(styles: &SectionStyles) -> u8 {
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
pub struct TextStyles {
    pub align: f32,
    pub line_offset: f32,
    pub paragraph_offset: f32,
    pub left_to_right: bool,
    pub wrap_on_overflow: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct SectionStyles {
    pub left_pad: f32,
    pub right_pad: f32,
    pub color: [f32; 4],
    pub font_size: f32,
    pub font: FontIdx,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub styles: Rc<TextStyles>,
    pub shape: ShapeStorages,
    pub sections: Vec<TextSection>,
}

#[derive(Debug, Clone)]
pub struct TextSection {
    pub styles: Rc<SectionStyles>,
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
    pub fn from_str(text: &str) -> Self {
        Self {
            styles: Rc::new(TextStyles::default()),
            shape: ShapeStorages::Internal(TextShape::default()),
            sections: vec![TextSection::new(text)],
        }
    }
    pub fn new() -> Self {
        Self {
            styles: Rc::new(TextStyles::default()),
            shape: ShapeStorages::Internal(TextShape::default()),
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
            styles: &TextStyles,
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
                .max_by(|l, r| if l > r {Ordering::Greater} else {Ordering::Less})
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
                let flags = GlyphFlags::section_styles_to_flags(&section.styles);
                let font_size = section.styles.font_size;
                (left_pos, top_pos) = match section.kind {
                    SectionKinds::Section => {
                        (left_pos.max(section.styles.left_pad + bounds.left), top_pos)
                    }
                    SectionKinds::NewLine => {
                        let max_height = endl(shape, styles, line_index, bounds);
                        line_index += 1;
                        (
                            section.styles.left_pad + bounds.left,
                            top_pos + max_height + styles.line_offset,
                        )
                    }
                    SectionKinds::NewParagraph => {
                        let max_height = endl(shape, styles, line_index, bounds);
                        line_index += 1;
                        (
                            section.styles.left_pad + bounds.left,
                            top_pos + max_height + styles.line_offset + styles.paragraph_offset,
                        )
                    }
                };
                let mut phys_line = PhysicalLine {
                    line_index,
                    chars: Vec::new(),
                    color: section.styles.color,
                    height: section.styles.font_size,
                    bounds: Rect {
                        left: left_pos,
                        top: top_pos,
                        width: 0.0,
                        height: section.styles.font_size,
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
                                section.styles.font.raw() as usize,
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
        cb: impl FnOnce(&mut TextShape, &TextStyles, &mut Vec<TextSection>),
    ) {
        match shape {
            Some(s) => cb(s, &self.styles, &mut self.sections),
            None => match &mut self.shape {
                ShapeStorages::External => return,
                ShapeStorages::Internal(s) => cb(s, &self.styles, &mut self.sections),
                ShapeStorages::Shared(s) => match s.write() {
                    Ok(mut s) => cb(&mut s, &self.styles, &mut self.sections),
                    Err(_) => return,
                },
                ShapeStorages::ThreadSync(s) => match s.lock() {
                    Ok(mut s) => cb(&mut s, &self.styles, &mut self.sections),
                    Err(_) => return,
                },
            },
        };
    }
    pub fn with_shape(
        &self,
        shape: Option<&mut TextShape>,
        cb: impl FnOnce(&TextShape, &TextStyles, &Vec<TextSection>),
    ) {
        match shape {
            Some(s) => cb(s, &self.styles, &self.sections),
            None => match &self.shape {
                ShapeStorages::External => return,
                ShapeStorages::Internal(s) => cb(s, &self.styles, &self.sections),
                ShapeStorages::Shared(s) => match s.write() {
                    Ok(s) => cb(&s, &self.styles, &self.sections),
                    Err(_) => return,
                },
                ShapeStorages::ThreadSync(s) => match s.lock() {
                    Ok(s) => cb(&s, &self.styles, &self.sections),
                    Err(_) => return,
                },
            },
        };
    }
}

impl Default for TextStyles {
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

impl Default for SectionStyles {
    fn default() -> Self {
        SectionStyles {
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
    pub fn new(text: &str) -> Self {
        Self {
            styles: Rc::new(SectionStyles::default()),
            text: Rope::from_str(text),
            kind: SectionKinds::Section,
        }
    }
}