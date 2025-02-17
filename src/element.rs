use std::fmt::Debug;

use crate::{text::DEFAULT_FONT_SIZE, EventListeners, ImageData, Styles, Value, Vector};

pub struct Element<Msg: Clone, Img: Clone + ImageData> {
    pub label: Option<String>,
    pub events: EventListeners<Msg>,
    pub children: Option<Vec<ElementKey>>,
    pub(crate) instance: ElementInstance,
    pub(crate) styles: Styles<Img>,
    pub(crate) dirty_styles: bool,
    pub procedures: Vec<Value>,
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct ElementKey(pub(crate) u64);

impl ElementKey {
    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ElementInstance {
    pub container: Container,
    pub color: [f32; 4],
    pub flags: u32,
    pub round: f32,
    pub shadow: f32,
    pub alpha: f32,
    /// x, y
    pub lin_grad_p1: Vector,
    /// x, y
    pub lin_grad_p2: Vector,
    pub lin_grad_color1: [f32; 4],
    pub lin_grad_color2: [f32; 4],
    /// x, y
    pub rad_grad_p1: Vector,
    /// x, y
    pub rad_grad_p2: Vector,
    pub rad_grad_color1: [f32; 4],
    pub rad_grad_color2: [f32; 4],
    pub image_tint: [f32; 4],
    pub image_size: Vector,
    pub scroll: Vector,
    pub padding: f32,
    pub shadow_alpha: f32,
    pub font: u16,
    pub font_size: f32,
    pub font_color: [f32; 4],
    pub text_wrap: bool,
    pub text_align: f32,
    pub margin: f32,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Flags {
    LinearGradient = 0,
    RadialGradient,
    Image,
    OverflowHidden,
    Count,
}

impl From<Flags> for f64 {
    fn from(value: Flags) -> Self {
        (1 << value as u64) as f64
    }
}

impl From<Flags> for u32 {
    fn from(value: Flags) -> Self {
        1 << value as u32
    }
}

impl Flags {
    pub const NONE: u64 = 0;

    #[inline]
    pub fn contained_in(self, flags: u32) -> bool {
        flags & self.into_u32() > 0
    }

    #[inline]
    pub fn into_u32(self) -> u32 {
        1 << self as u32
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
#[repr(C)]
pub struct Container {
    pub pos: Vector,
    pub size: Vector,
    pub rotation: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct ContainerWrapper {
    container: Container,
    dirty_pos: bool,
    dirty_size: bool,
    dirty_rotation: bool,
}

impl<Msg: Clone, Img: Clone + ImageData> Element<Msg, Img> {
    pub fn instance(&self) -> &ElementInstance {
        &self.instance
    }

    pub fn styles(&self) -> &Styles<Img> {
        &self.styles
    }

    pub fn styles_mut(&mut self) -> &mut Styles<Img> {
        self.dirty_styles = true;
        &mut self.styles
    }

    pub fn child(&self, idx: usize) -> Option<&ElementKey> {
        match &self.children {
            Some(c) => c.get(idx),
            None => None,
        }
    }

    pub fn add_child(&mut self, key: ElementKey) {
        match &mut self.children {
            Some(children) => {
                children.push(key);
            }
            None => {
                self.children = Some(vec![key])
            }
        }
    }
}

impl<Msg: Clone, Img: Clone + ImageData> Default for Element<Msg, Img> {
    fn default() -> Self {
        Self {
            label: None,
            events: EventListeners::new(),
            children: None,
            instance: ElementInstance::default(),
            styles: Styles::default(),
            procedures: Vec::new(),
            dirty_styles: true,
        }
    }
}

impl ContainerWrapper {
    pub const fn new(c: &Container) -> Self {
        Self {
            container: *c,
            dirty_pos: false,
            dirty_size: false,
            dirty_rotation: false,
        }
    }

    pub const fn new_dirty(c: &Container) -> Self {
        Self {
            container: *c,
            dirty_pos: true,
            dirty_size: true,
            dirty_rotation: true,
        }
    }

    pub fn get(&self) -> &Container {
        &self.container
    }

    pub fn set_pos(&mut self, v: Vector) {
        self.dirty_pos = self.container.pos != v;
        self.container.pos = v;
    }

    pub fn set_size(&mut self, v: Vector) {
        self.dirty_size = true;
        self.container.size = v;
    }

    pub fn set_rotation(&mut self, v: f32) {
        self.dirty_rotation = true;
        self.container.rotation = v;
    }

    pub fn clean(&mut self) {
        self.dirty_pos = false;
        self.dirty_size = false;
        self.dirty_rotation = false;
    }

    pub fn fix_pos(&mut self) -> Option<&Vector> {
        if !self.dirty_pos {
            return None;
        }
        self.dirty_pos = false;
        Some(&self.container.pos)
    }

    pub fn fix_size(&mut self) -> Option<&Vector> {
        if !self.dirty_size {
            return None;
        }
        self.dirty_size = false;
        Some(&self.container.size)
    }

    pub fn fix_rotation(&mut self) -> Option<&f32> {
        if !self.dirty_rotation {
            return None;
        }
        self.dirty_rotation = false;
        Some(&self.container.rotation)
    }

    pub fn dirty_pos(&self) -> bool {
        self.dirty_pos
    }

    pub fn dirty_size(&self) -> bool {
        self.dirty_size
    }

    pub fn dirty_rotation(&self) -> bool {
        self.dirty_rotation
    }

    pub fn pos_mut(&mut self) -> &mut Vector {
        self.dirty_pos = true;
        &mut self.container.pos
    }

    pub fn size_mut(&mut self) -> &mut Vector {
        self.dirty_size = true;
        &mut self.container.size
    }

    pub fn rot_mut(&mut self) -> &mut f32 {
        self.dirty_rotation = true;
        &mut self.container.rotation
    }
}

impl Default for ElementInstance {
    fn default() -> Self {
        Self {
            container: Container::default(),
            color: [0.0; 4],
            flags: 0,
            round: 0.0,
            shadow: 0.0,
            alpha: 1.0,
            lin_grad_p1: Vector::default(),
            lin_grad_p2: Vector::default(),
            lin_grad_color1: [0.0; 4],
            lin_grad_color2: [0.0; 4],
            rad_grad_p1: Vector::default(),
            rad_grad_p2: Vector::default(),
            rad_grad_color1: [0.0; 4],
            rad_grad_color2: [0.0; 4],
            image_size: Vector::ZERO,
            image_tint: [1.0; 4],
            scroll: Vector::ZERO,
            padding: 0.0,
            shadow_alpha: 1.0,
            font: 0,
            font_size: DEFAULT_FONT_SIZE,
            font_color: [1.0, 1.0, 1.0, 1.0],
            text_wrap: true,
            text_align: 0.0,
            margin: 0.0,
        }
    }
}

impl ElementInstance {
    pub fn set_flag(&mut self, flag: Flags) {
        self.flags |= u32::from(flag);
    }

    pub fn remove_flag(&mut self, flag: Flags) {
        self.flags &= !u32::from(flag);
    }
}
