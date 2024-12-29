use std::fmt::Debug;

use crate::{ElemEventTypes, ImageData, ListenerTypes, Styles, Vector};

pub struct Element<Msg: Clone, Img: Clone + ImageData> {
    pub label: Option<String>,
    pub events: Vec<EventListener<Msg>>,
    pub children: Option<Vec<ElementKey>>,
    pub(crate) instance: ElementInstance,
    pub(crate) styles: Styles<Img>,
    pub(crate) dirty_styles: bool,
    pub allow_select: bool,
    pub allow_text_input: bool,
}

pub struct EventListener<Msg: Clone> {
    pub event: ElemEventTypes,
    pub msg: Option<Msg>,
    pub kind: ListenerTypes,
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct ElementKey(pub(crate) u64);

impl ElementKey {
    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable, bytemuck::Pod))]
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct ElementInstance {
    pub container: Container,
    pub color: [f32; 4],
    pub flags: u32,
    pub round: [f32; 2],
    pub alpha: f32,
    pub lin_grad_p1: Vector,
    pub lin_grad_p2: Vector,
    pub lin_grad_color1: [f32; 4],
    pub lin_grad_color2: [f32; 4],
    pub rad_grad_p1: Vector,
    pub rad_grad_p2: Vector,
    pub rad_grad_color1: [f32; 4],
    pub rad_grad_color2: [f32; 4],
    pub image_tint: [f32; 4],
    pub image_size: [f32; 2],
}


#[repr(u8)]
pub enum Flags {
    LinearGradient = 0,
    RadialGradient,
    Image,
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
}
#[cfg_attr(feature = "bytemuck", derive(bytemuck::Zeroable, bytemuck::Pod))]
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
}

impl<Msg: Clone, Img: Clone + ImageData> Default for Element<Msg, Img> {
    fn default() -> Self {
        Self {
            label: None,
            events: Vec::new(),
            children: None,
            instance: ElementInstance::default(),
            styles: Styles::default(),
            dirty_styles: true,
            allow_select: false,
            allow_text_input: false,
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
        self.container.pos = v;
        self.dirty_pos = true;
    }

    pub fn set_size(&mut self, v: Vector) {
        self.container.size = v;
        self.dirty_size = true;
    }

    pub fn set_rotation(&mut self, v: f32) {
        self.container.rotation = v;
        self.dirty_rotation = true;
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
            round: [0.0; 2],
            alpha: 1.0,
            lin_grad_p1: Vector::default(),
            lin_grad_p2: Vector::default(),
            lin_grad_color1: [0.0; 4],
            lin_grad_color2: [0.0; 4],
            rad_grad_p1: Vector::default(),
            rad_grad_p2: Vector::default(),
            rad_grad_color1: [0.0; 4],
            rad_grad_color2: [0.0; 4],
            image_size: [0.0; 2],
            image_tint: [1.0; 4],
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
