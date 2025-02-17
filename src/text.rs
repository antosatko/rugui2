use ropey::Rope;
use swash::{
    shape::ShapeContext,
    text::{
        cluster::{CharCluster, Parser, Status, Token},
        Script,
    },
    Attributes, CacheKey, Charmap, FontRef, GlyphId,
};

use crate::events::EnvEventStates;

pub const DEFAULT_FONT_SIZE: f32 = 18.0;

pub struct TextProccesor {
    pub shape_ctx: ShapeContext,
    pub(crate) fonts: Vec<Font>,
    pub(crate) cluster: CharCluster,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct FontIdx(pub(crate) u16);

impl FontIdx {
    pub fn raw(&self) -> u16 {
        self.0
    }
}

impl TextProccesor {
    pub fn new() -> Self {
        let shape_ctx = ShapeContext::new();
        let fonts = Vec::new();
        let cluster = CharCluster::new();
        Self {
            shape_ctx,
            fonts,
            cluster,
        }
    }

    pub fn add_font(&mut self, font: Font) -> FontIdx {
        let idx = FontIdx(self.fonts.len() as u16);
        self.fonts.push(font);
        idx
    }

    pub fn get_font(&self, idx: FontIdx) -> FontRef {
        self.fonts[idx.0 as usize].as_ref()
    }

    pub(crate) fn procces(
        &mut self,
        font: FontIdx,
        text: &mut PhysicalText,
        font_size: f32,
        bounds: Rect,
        line_wrap: bool,
        line_align: f32,
        scroll: crate::Vector,
    ) {
        text.active_lines = 0;
        let mut char_idx = 0;
        let line_height = font_size;
        let mut lines_count = 0;
        for (i, (line, line_slice)) in text.lines.iter_mut().zip(text.text.lines()).enumerate() {
            char_idx = text.text.line_to_char(i);
            /*if lines_count as f32 * line_height > bounds.height {
                break;
            }*/
            line.dirty = false;
            line.active_wraps = 0;
            match line.wraps.first_mut() {
                Some(wrap) => {
                    wrap.active_chars = 0;
                    wrap.bb = Rect {
                        left: bounds.left + scroll.0,
                        top: bounds.top + lines_count as f32 * line_height + scroll.1,
                        width: 0.0,
                        height: line_height,
                    };
                }
                None => {
                    line.wraps.push(PhysicalWrap {
                        bb: Rect {
                            left: bounds.left + scroll.0,
                            top: bounds.top + lines_count as f32 * line_height + scroll.1,
                            width: 0.0,
                            height: line_height,
                        },
                        phys_chars: Vec::new(),
                        active_chars: 0,
                    });
                }
            };
            text.active_lines += 1;
            for chunk in line_slice.chunks() {
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
                while parser.next(&mut self.cluster) {
                    let i = match select_pref_font(&self.fonts, font.0 as usize, &mut self.cluster)
                    {
                        Some(i) => i,
                        None => continue,
                    };
                    let font_key = self.fonts[i].key;
                    let mut shaper = self
                        .shape_ctx
                        .builder(self.fonts[i].as_ref())
                        .size(font_size)
                        .build();

                    shaper.add_cluster(&self.cluster);

                    shaper.shape_with(|cluster| {
                        let src = cluster.source;
                        for glyph in cluster.glyphs {
                            let wrap = &mut line.wraps[line.active_wraps];
                            let glyph_key = GlyphKey {
                                font_idx: FontIdx(i as u16),
                                font_key,
                                glyph_id: glyph.id,
                                font_size: font_size.round() as u32,
                                flags: 0,
                            };
                            let phys_char = PhysicalChar {
                                /*start: src.start as usize,
                                end: src.end as usize,*/
                                idx: char_idx,
                                glyph_key,
                                width: glyph.advance,
                            };
                            char_idx += 1;
                            if wrap.bb.width + glyph.advance <= bounds.width || !line_wrap {
                                match wrap.phys_chars.get_mut(wrap.active_chars) {
                                    Some(old_phys_char) => old_phys_char.clone_from(&phys_char),
                                    None => wrap.phys_chars.push(phys_char),
                                }
                                wrap.active_chars += 1;
                                wrap.bb.width += phys_char.width;
                            } else {
                                lines_count += 1;
                                wrap.bb.width += phys_char.width;
                                match wrap.phys_chars.get_mut(wrap.active_chars) {
                                    Some(old_phys_char) => old_phys_char.clone_from(&phys_char),
                                    None => wrap.phys_chars.push(phys_char),
                                }
                                wrap.active_chars += 1;
                                wrap.bb.left += line_align * (bounds.width - wrap.bb.width);
                                line.active_wraps += 1;
                                match line.wraps.get_mut(line.active_wraps) {
                                    Some(wrap) => {
                                        wrap.active_chars = 0;
                                        wrap.bb = Rect {
                                            left: bounds.left + scroll.0,
                                            top: bounds.top
                                                + lines_count as f32 * line_height
                                                + scroll.1,
                                            width: 0.0,
                                            height: line_height,
                                        };
                                    }
                                    None => {
                                        line.wraps.push(PhysicalWrap {
                                            phys_chars: vec![],
                                            bb: Rect {
                                                left: bounds.left + scroll.0,
                                                top: bounds.top
                                                    + lines_count as f32 * line_height
                                                    + scroll.1,
                                                width: 0.0,
                                                height: line_height,
                                            },
                                            active_chars: 0,
                                        });
                                    }
                                };
                            }
                        }
                    });
                }
            }
            if line.wraps.first().is_none() {
                line.wraps.push(PhysicalWrap {
                    bb: Rect {
                        left: bounds.left + scroll.0,
                        top: bounds.top + lines_count as f32 * line_height + scroll.1,
                        width: 0.0,
                        height: line_height,
                    },
                    ..Default::default()
                })
            }
            lines_count += 1;
            line.active_wraps += 1;
            if let Some(wrap) = line.wraps.get_mut(line.active_wraps) {
                wrap.bb.left += line_align * (bounds.width - wrap.bb.width);
            }
        }
        text.bb = Rect::minimal(text.lines.iter().flat_map(|l| l.wraps.iter().map(|w| w.bb)));
    }
}

#[derive(Debug, Clone, Default)]
pub struct TextRepr {
    pub text: PhysicalText,
    pub variant: TextVariants,
}

#[derive(Debug, Clone, Default)]
pub enum TextVariants {
    #[default]
    Label,
    Paragraph {
        selection: Option<TextSelection>,
    },
    Editor {
        selection: Option<TextSelection>,
        editor: TextEditor,
    },
}

impl TextVariants {
    pub fn selection_mut(&mut self) -> Option<&mut Option<TextSelection>> {
        match self {
            Self::Editor { selection, .. } => Some(selection),
            Self::Paragraph { selection, .. } => Some(selection),
            Self::Label => None,
        }
    }

    pub fn editor_mut(&mut self) -> Option<&mut TextEditor> {
        match self {
            Self::Editor { editor, .. } => Some(editor),
            Self::Paragraph { .. } => None,
            Self::Label => None,
        }
    }
    pub fn selection(&self) -> Option<&Option<TextSelection>> {
        match self {
            Self::Editor { selection, .. } => Some(selection),
            Self::Paragraph { selection, .. } => Some(selection),
            Self::Label => None,
        }
    }

    pub fn editor(&self) -> Option<&TextEditor> {
        match self {
            Self::Editor { editor, .. } => Some(editor),
            Self::Paragraph { .. } => None,
            Self::Label => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Paragraph {
    pub selection: Option<TextSelection>,
}

#[derive(Debug, Clone, Default, Copy)]
pub struct TextSelection {
    pub start: usize,
    pub end: usize,
    pub sorted: (usize, usize),
}

impl TextSelection {
    pub fn sort(&mut self) {
        if self.start <= self.end {
            self.sorted = (self.start, self.end)
        } else {
            self.sorted = (self.end, self.start)
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TextEditor {
    pub cursor: Cursor,
}

#[derive(Debug, Clone, Default, Copy)]
pub struct Cursor {
    pub idx: usize,
    pub line: usize,
    pub column: usize,
    pub target_column: usize,
}

impl Cursor {
    /// move to `line`
    pub fn move_to_line(&mut self, line: i32, text: &PhysicalText) {
        if line < 0 {
            self.min();
            return;
        }
        let line = line as usize;
        if line == self.line {
            return;
        }
        if line >= text.text.len_lines() {
            self.max(text);
            return;
        }
        let bounds = text.line_bounds(line);
        self.line = line;
        if self.target_column + bounds.0 < bounds.1 {
            self.idx = self.target_column + bounds.0;
            return;
        }
        self.column = bounds.1 - bounds.0;
        self.idx = bounds.1 - 1;
    }

    pub fn move_by_line(&mut self, line: i32, text: &PhysicalText) {
        let line = self.line as i32 + line;
        self.move_to_line(line, text);
    }

    /// move by `column`
    pub fn move_by_column(&mut self, column: i32, text: &PhysicalText) {
        if column == 0 {
            return;
        }
        let new_idx = self.idx as i32 + column;
        if new_idx < 0 {
            self.min();
            return;
        }
        let new_idx = new_idx as usize;
        let len = text.text.len_chars();
        if new_idx >= len {
            self.max(text);
            return;
        }
        self.move_to_idx(new_idx, text);
    }

    pub fn move_to_idx(&mut self, idx: usize, text: &PhysicalText) {
        let sign = (idx as i32 - self.idx as i32).signum();
        if sign == 0 {
            return;
        }
        if idx > text.text.len_chars() {
            self.max(text);
            return;
        }
        self.idx = idx;
        loop {
            if self.line >= text.text.len_lines() {
                self.max(text);
                return;
            }
            let bounds = text.line_bounds(self.line);
            if self.idx >= bounds.0 && self.idx < bounds.1 {
                self.column = self.idx - bounds.0;
                self.target_column = self.column;
                return;
            }
            self.line = (self.line as i32 + sign) as usize;
        }
    }

    pub fn endl(&mut self, text: &PhysicalText) {
        let is_last = (self.line == text.text.len_lines() - 1)
            .then(|| 0)
            .unwrap_or_else(|| 1);
        let bounds = text.line_bounds(self.line);
        self.idx = bounds.1 - is_last;
        self.column = bounds.1 - bounds.0 - is_last;
        self.target_column = self.column;
    }

    pub fn startl(&mut self, text: &PhysicalText) {
        let bounds = text.line_bounds(self.line);
        self.idx = bounds.0;
        self.column = 0;
        self.target_column = 0;
    }

    fn min(&mut self) {
        self.column = 0;
        self.idx = 0;
        self.line = 0;
        self.target_column = 0;
    }

    fn max(&mut self, text: &PhysicalText) {
        let last_line_idx = text.text.len_lines() - 1;
        self.line = last_line_idx;
        self.endl(text);
    }
}

impl PhysicalText {
    fn line_bounds(&self, line: usize) -> (usize, usize) {
        (
            self.text.line_to_char(line),
            self.text
                .line_to_char((line + 1).min(self.text.len_lines())),
        )
    }
}

impl TextRepr {
    pub fn new_label(txt: &str) -> Self {
        let mut text = PhysicalText::default();
        let str = &txt.replace("\r\n", "\n");
        text.push_str(str);
        Self {
            text,
            ..Default::default()
        }
    }

    pub fn new_paragraph(txt: &str) -> Self {
        let mut text = PhysicalText::default();
        let str = &txt.replace("\r\n", "\n");
        text.push_str(str);
        Self {
            text,
            variant: TextVariants::Paragraph { selection: None },
        }
    }

    pub fn new_editor(txt: &str) -> Self {
        let mut text = PhysicalText::default();
        let str = &txt.replace("\r\n", "\n");
        text.push_str(str);
        Self {
            text,
            variant: TextVariants::Editor {
                selection: None,
                editor: TextEditor::default(),
            },
        }
    }

    fn line_bounds(&self, line: usize) -> (usize, usize) {
        self.text.line_bounds(line)
    }

    pub fn move_cursor(&mut self, cmd: MoveCommand) -> EnvEventStates {
        let (selection, editor) = match &self.variant {
            TextVariants::Editor { selection, editor } => (selection, editor),
            TextVariants::Paragraph { .. } => return EnvEventStates::Free,
            TextVariants::Label => return EnvEventStates::Free,
        };
        let mut cursor = editor.cursor;
        match cmd.cmd {
            MoveCommands::MoveChar => match cmd.direction {
                Directions::Up => cursor.move_by_line(-1, &self.text),
                Directions::Down => cursor.move_by_line(1, &self.text),
                Directions::Right => cursor.move_by_column(1, &self.text),
                Directions::Left => cursor.move_by_column(-1, &self.text),
            },
            MoveCommands::MoveWord => todo!(),
            MoveCommands::MoveLine => match cmd.direction {
                Directions::Up => cursor.min(),
                Directions::Down => cursor.max(&self.text),
                Directions::Right => cursor.endl(&self.text),
                Directions::Left => cursor.startl(&self.text),
            },
        }
        let (selection, editor) = match &mut self.variant {
            TextVariants::Editor { selection, editor } => (selection, editor),
            TextVariants::Paragraph { .. } => return EnvEventStates::Free,
            TextVariants::Label => return EnvEventStates::Free,
        };
        match cmd.hold_select {
            true => match selection {
                Some(selection) => {
                    selection.end = cursor.idx;
                    selection.sort();
                    editor.cursor = cursor;
                }
                None => {
                    let mut new = TextSelection {
                        start: editor.cursor.idx,
                        end: cursor.idx,
                        sorted: (0, 0),
                    };
                    new.sort();
                    *selection = Some(new);
                    editor.cursor = cursor;
                }
            },
            false => {
                *selection = None;
                editor.cursor = cursor;
            }
        }

        EnvEventStates::Consumed
    }

    pub fn insert_str(&mut self, str: &str) -> EnvEventStates {
        let (selection, editor) = match &mut self.variant {
            TextVariants::Label => return EnvEventStates::Free,
            TextVariants::Paragraph { .. } => return EnvEventStates::Free,
            TextVariants::Editor { selection, editor } => (selection, editor),
        };
        let str = &str.replace("\r\n", "\n");
        for _ in 0..str.chars().filter(|c| *c == '\n').count() {
            self.text.lines.push(PhysicalLine{
                ..Default::default()
            });
        }

        let len = str.chars().count();
        match selection {
            Some(selection) => {
                self.text
                    .text
                    .remove(selection.sorted.0..selection.sorted.1);
                self.text.text.insert(selection.sorted.0, str);
                editor
                    .cursor
                    .move_to_idx(selection.sorted.0 + 1, &self.text);
            }
            None => {
                let cursor = editor.cursor;
                self.text.text.insert(cursor.idx, str);
                editor.cursor.move_by_column(len as i32, &self.text);
            }
        }
        *selection = None;

        EnvEventStates::Consumed
    }

    pub fn remove(&mut self) -> EnvEventStates {
        let (selection, editor) = match &mut self.variant {
            TextVariants::Label => return EnvEventStates::Free,
            TextVariants::Paragraph { .. } => return EnvEventStates::Free,
            TextVariants::Editor { selection, editor } => (selection, editor),
        };

        match selection {
            Some(selection) => {
                self.text
                    .text
                    .remove(selection.sorted.0..selection.sorted.1);
                editor.cursor.move_to_idx(selection.sorted.0, &self.text);
            }
            None => {
                let cursor = editor.cursor;
                self.text
                    .text
                    .remove((cursor.idx).max(1) - 1..cursor.idx.min(self.text.text.len_chars()));
                editor.cursor.move_by_column(-1, &self.text);
            }
        }
        *selection = None;

        EnvEventStates::Consumed
    }

    pub fn delete(&mut self) -> EnvEventStates {
        let (selection, editor) = match &mut self.variant {
            TextVariants::Label => return EnvEventStates::Free,
            TextVariants::Paragraph { .. } => return EnvEventStates::Free,
            TextVariants::Editor { selection, editor } => (selection, editor),
        };

        match selection {
            Some(selection) => {
                self.text
                    .text
                    .remove(selection.sorted.0..selection.sorted.1);
                editor.cursor.move_to_idx(selection.sorted.0, &self.text);
            }
            None => {
                let cursor = editor.cursor;
                self.text
                    .text
                    .remove(cursor.idx..(cursor.idx + 1).min(self.text.text.len_chars()));
            }
        }
        *selection = None;

        EnvEventStates::Consumed
    }

    pub fn select_all(&mut self) -> EnvEventStates {
        let selection = match &mut self.variant {
            TextVariants::Label => return EnvEventStates::Free,
            TextVariants::Paragraph { selection } => selection,
            TextVariants::Editor { selection, .. } => selection,
        };

        let start = 0;
        let end = self.text.text.len_chars();

        *selection = Some(TextSelection { start, end, sorted: (start, end) });

        if let Some(editor) = self.variant.editor_mut() {
            editor.cursor.move_to_idx(end, &self.text);
        }

        EnvEventStates::Consumed
    }

    pub fn line_bounds_of_char(&self, idx: usize) -> (usize, usize) {
        for line in &self.text.lines {
            if line.start <= idx && line.end > idx {
                return (line.start, line.end);
            }
        }
        match self.text.lines.last() {
            Some(last) => (last.end, last.end),
            None => (0, 0),
        }
    }

    pub fn line_index_of_char(&self, idx: usize) -> usize {
        for (index, line) in self.text.lines.iter().enumerate() {
            if line.start <= idx && line.end > idx {
                return index;
            }
        }
        self.text.lines.len()
    }
}

#[derive(Debug, Clone, Default)]
pub struct PhysicalLine {
    pub wraps: Vec<PhysicalWrap>,
    pub start: usize,
    pub end: usize,
    //pub text: String,
    pub dirty: bool,
    pub active_wraps: usize,
}

#[derive(Debug, Clone, Default)]
pub struct PhysicalWrap {
    pub phys_chars: Vec<PhysicalChar>,
    pub active_chars: usize,
    pub bb: Rect,
}

#[derive(Debug, Clone, Default)]
pub struct PhysicalText {
    pub lines: Vec<PhysicalLine>,
    pub text: Rope,
    pub bb: Rect,
    pub active_lines: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct PhysicalChar {
    pub idx: usize,
    pub width: f32,
    pub glyph_key: GlyphKey,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct GlyphKey {
    pub font_key: CacheKey,
    pub glyph_id: GlyphId,
    pub font_size: u32,
    pub font_idx: FontIdx,
    pub flags: u8,
}

#[derive(Debug, Clone, Default, Copy)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

pub struct MoveCommand {
    pub cmd: MoveCommands,
    pub direction: Directions,
    pub hold_select: bool,
}

pub enum MoveCommands {
    MoveChar,
    MoveWord,
    MoveLine,
}

pub enum Directions {
    Up,
    Down,
    Right,
    Left,
}

impl PhysicalText {
    pub fn push_str(&mut self, txt: &str) {
        let str = &txt.replace("\r\n", "\n");
        let text = Rope::from_str(str);
        let mut input_lines = text.lines();
        if let Some(last_line) = self.lines.last_mut() {
            if let Some(first_input_line) = input_lines.next() {
                self.text.append(first_input_line.into());
                last_line.end = self.text.len_chars();
                //last_line.text.push_str(first_input_line);
                last_line.dirty = true;
            }
        }
        for line in input_lines {
            let start = self.text.len_chars();
            self.text.append(line.into());
            self.lines.push(PhysicalLine {
                wraps: Vec::new(),
                //text: line.to_string(),
                dirty: true,
                active_wraps: 0,
                start,
                end: self.text.len_chars(),
            });
        }
    }

    pub fn hit(&self, point: crate::Vector) -> Option<usize> {
        /*if !self.bb.hit(point) {
            return None;
        }*/
        for line in self.lines.iter().take(self.active_lines) {
            // incase we hit an empty wrap, there will be counter for char idx
            // I do not recommend using it unless necesarry
            let mut char_idx = line.start;
            for wrap in line.wraps.iter().take(line.active_wraps) {
                if !wrap.bb.hit_line(point) {
                    continue;
                }
                let mut left = wrap.bb.left;
                for char in wrap.phys_chars.iter().take(wrap.active_chars) {
                    if left + char.width >= point.0 {
                        let point = if left + char.width * 0.5 >= point.0 {
                            char.idx
                        } else {
                            char.idx + 1
                        };
                        return Some(point);
                    }
                    left += char.width;
                    char_idx += 1;
                }
                return match wrap.phys_chars.get(wrap.active_chars - 1) {
                    Some(char) => Some(char.idx + 1),
                    None => None, //Some(self.text.byte_to_char(char_idx)),
                };
            }
        }
        return None;
    }

    pub fn get_char(&self, index: usize) -> Option<char> {
        self.text.get_char(index)
    }

    pub fn clone_string_range(&self, start: usize, end: usize) -> Option<String> {
        self.text.get_slice(start..end).map(|s| s.to_string())
    }
}

impl Rect {
    pub const ZERO: Self = Self {
        left: 0.0,
        top: 0.0,
        width: 0.0,
        height: 0.0,
    };

    pub fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Rect {
            left,
            top,
            width,
            height,
        }
    }

    pub fn minimal<I: Iterator<Item = Rect>>(mut rects: I) -> Self {
        let mut this = match &rects.next() {
            Some(r) => *r,
            None => return Self::new(0.0, 0.0, 0.0, 0.0),
        };
        for rect in rects {
            if rect.left < this.left {
                this.left = rect.left
            }
            if rect.top < this.top {
                this.top = rect.top
            }
            let (r_right, r_bot) = (rect.left + rect.width, rect.top + rect.height);
            let (t_right, t_bot) = (this.left + this.width, this.top + this.height);
            if r_right > t_right {
                this.width = r_right - this.left
            }
            if r_bot > t_bot {
                this.height = r_bot - this.top
            }
        }
        this
    }
}

impl Rect {
    pub fn hit(&self, point: crate::Vector) -> bool {
        point.0 >= self.left
            && point.0 <= self.left + self.width
            && point.1 >= self.top
            && point.1 <= self.top + self.height
    }
    pub fn hit_line(&self, point: crate::Vector) -> bool {
        point.1 >= self.top && point.1 <= self.top + self.height
    }
}

pub struct Font {
    // Full content of the font file
    pub(crate) data: Vec<u8>,
    // Offset to the table directory
    pub(crate) offset: u32,
    // Cache key
    pub(crate) key: CacheKey,
}

impl Font {
    pub fn from_file(path: &str, index: usize) -> Option<Self> {
        // Read the full font file
        let data = std::fs::read(path).ok()?;
        // Create a temporary font reference for the first font in the file.
        // This will do some basic validation, compute the necessary offset
        // and generate a fresh cache key for us.
        let font = FontRef::from_index(&data, index)?;
        let (offset, key) = (font.offset, font.key);
        // Return our struct with the original file data and copies of the
        // offset and key from the font reference
        Some(Self { data, offset, key })
    }

    pub fn from_bytes(bytes: &[u8], index: usize) -> Option<Self> {
        // Create a temporary font reference for the first font in the file.
        // This will do some basic validation, compute the necessary offset
        // and generate a fresh cache key for us.
        let data = bytes.to_vec();
        let font = FontRef::from_index(&data, index)?;
        let (offset, key) = (font.offset, font.key);
        // Return our struct with the original file data and copies of the
        // offset and key from the font reference
        Some(Self { data, offset, key })
    }

    // As a convenience, you may want to forward some methods.
    pub fn attributes(&self) -> Attributes {
        self.as_ref().attributes()
    }

    pub fn charmap(&self) -> Charmap {
        self.as_ref().charmap()
    }

    // Create the transient font reference for accessing this crate's
    // functionality.
    pub fn as_ref(&self) -> FontRef {
        // Note that you'll want to initialize the struct directly here as
        // using any of the FontRef constructors will generate a new key which,
        // while completely safe, will nullify the performance optimizations of
        // the caching mechanisms used in this crate.
        FontRef {
            data: &self.data,
            offset: self.offset,
            key: self.key,
        }
    }
}

pub(crate) fn select_pref_font(fonts: &[Font], pref: usize, cluster: &mut CharCluster) -> Option<usize> {
    let mut best = None;
    {
        let charmap = match fonts.get(pref) {
            Some(f) => f.charmap(),
            None => return None,
        };
        match cluster.map(|ch| charmap.map(ch)) {
            Status::Complete => return Some(pref),
            Status::Keep => best = Some(pref),
            _ => (),
        }
    }
    for (i, font) in fonts.iter().enumerate().filter(|(i, _)| *i != pref) {
        let charmap = font.charmap();
        match cluster.map(|ch| charmap.map(ch)) {
            // This font provided a glyph for every character
            Status::Complete => return Some(i),
            // This font provided the most complete mapping so far
            Status::Keep => best = Some(i),
            // A previous mapping was more complete
            Status::Discard => {}
        }
    }
    best
}
