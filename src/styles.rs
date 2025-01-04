use std::fmt::Debug;

use crate::{variables::{VarKey, Variables}, Colors, Vector};

#[derive(Debug, Clone)]
pub struct Styles<Img: Clone + ImageData> {
    /// Width of the element
    pub width: StyleComponent<Value>,
    /// Maximum width of the element
    pub max_width: StyleComponent<Option<Value>>,
    /// Minimum width of the element
    pub min_width: StyleComponent<Option<Value>>,
    /// Height of the element
    pub height: StyleComponent<Value>,
    /// Maximum height of the element
    pub max_height: StyleComponent<Option<Value>>,
    /// Minimum height of the element
    pub min_height: StyleComponent<Option<Value>>,
    /// Gap between the element and its container
    pub padding: StyleComponent<Value>,
    /// Gap between the element and its children
    pub margin: StyleComponent<Option<Value>>,
    /// Color of the element
    pub color: StyleComponent<Colors>,
    /// Rotation of the element
    pub rotation: StyleComponent<Rotation>,
    /// Round edges
    /// 
    /// Describes the radius of edge circle
    pub round: StyleComponent<Option<Round>>,
    /// Overall opacity of element
    pub alpha: StyleComponent<f32>,
    /// Position of the Element
    /// 
    /// Defaults to the middle of container
    pub position: StyleComponent<Position>,
    /// Describes where the origin of the element is
    /// 
    /// Defaults to element center
    pub origin: StyleComponent<Position>,
    /// Linear gradient
    pub grad_linear: StyleComponent<Option<Gradient>>,
    /// Radial gradient
    pub grad_radial: StyleComponent<Option<Gradient>>,
    /// Image
    /// 
    /// Images are not part of Rugui2 API, see documentation
    /// of your drawing layer to learn about their Images
    pub image: StyleComponent<Option<Image<Img>>>,
    /// Image tint
    /// 
    /// Images are not part of Rugui2 API, see documentation
    /// of your drawing layer to learn about their Images
    pub image_tint: StyleComponent<Colors>,
    /// Vertical scroll
    pub scroll_y: StyleComponent<Value>,
    /// Horizontal scroll
    pub scroll_x: StyleComponent<Value>,
    /// Define how to render overflow
    pub overflow: StyleComponent<Overflow>,
}

#[derive(Debug)]
pub enum Style {
    Width,
    MaxWidth,
    MinWidth,
    Height,
    MaxHeight,
    MinHeight,
    Color,
    Rotation,
    Round,
    Alpha,
    Center,
    Align,
    GradLinear,
    GradRadial,
    Image,
    ImageTint,
    ScrollY,
    ScrollX,
    Overflow,
    Padding,
    Margin,
}

#[derive(Clone, Debug)]
pub enum Overflow {
    Shown,
    Hidden,
}


#[derive(Clone)]
pub struct Image<Img: Clone + ImageData> {
    pub data: Img,
}

impl<Img: Clone + ImageData> std::fmt::Debug for Image<Img> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("size", &self.data.get_size())
            .finish()
    }
}

pub trait ImageData {
    fn get_size(&self) -> (u32, u32);
}

impl ImageData for () {
    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }
}

#[derive(Debug, Clone)]
pub struct Gradient {
    pub p1: (Position, Colors),
    pub p2: (Position, Colors),
}

#[derive(Debug, Clone)]
pub struct Position {
    pub width: Value,
    pub height: Value,
    pub container: Container,
}

#[derive(Debug, Clone)]
pub struct Round {
    pub size: Value,
    pub anti_aliasing: Value,
}

#[derive(Debug, Clone)]
pub enum Portion {
    Full,
    Half,
    Zero,
    Percent(f32),
    Mul(f32),
    Div(f32),
}

#[derive(Debug, Clone)]
pub enum Value {
    Px(f32),
    Time,
    Value(Container, Values, Portion),
    Variable(VarKey),
    SetVariable(VarKey, Box<Value>),
    Debug(Box<Value>, Option<String>),
    Add(Box<(Value, Value)>),
    Mul(Box<(Value, Value)>),
    Negative(Box<Value>),
    Zero,
}

#[derive(Debug, Clone)]
pub enum Values {
    Width,
    Height,
    Diameter,
    Max,
    Min,
    Avg,
}

#[derive(Debug, Clone)]
pub struct StyleComponent<T: Debug + Clone> {
    val: T,
    dirty: bool,
    dynamic: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Rotation {
    pub rot: Rotations,
    pub cont: Container,
}

#[derive(Debug, Clone, Default)]
pub enum Rotations {
    #[default]
    None,
    Deg(f32),
    Rad(f32),
    CalcDeg(Value),
    CalcRad(Value),
}

#[derive(Debug, Copy, Clone, Default)]
pub enum Container {
    ViewPort,
    #[default]
    Container,
    This,
    Image,
}

pub(crate) struct Containers<'a> {
    pub container: &'a crate::element::Container,
    pub vp: &'a crate::element::Container,
    pub this: &'a crate::element::Container,
    pub image: &'a Vector,
    pub time: f32,
}

impl<Tex: ImageData + Clone> Default for Styles<Tex> {
    fn default() -> Self {
        let val = StyleComponent::new;
        let color = StyleComponent::new;
        let rot = StyleComponent::new;
        let opt_rnd = StyleComponent::new;
        let float = StyleComponent::new;
        let pos = StyleComponent::new;
        let opt_val = StyleComponent::new;
        let opt_grad = StyleComponent::new(None);
        let opt_img = StyleComponent::new(None);
        let overflow = StyleComponent::new;
        Self {
            width: val(Value::Value(
                Container::Container,
                Values::Width,
                Portion::Full,
            )),
            max_width: opt_val(None),
            min_width: opt_val(None),
            height: val(Value::Value(
                Container::Container,
                Values::Height,
                Portion::Full,
            )),
            max_height: opt_val(None),
            min_height: opt_val(None),
            color: color(Colors::FRgba(0.0, 0.0, 0.0, 0.0)),
            rotation: rot(Rotation {
                rot: Rotations::None,
                cont: Container::Container,
            }),
            round: opt_rnd(None),
            alpha: float(1.0),
            position: pos(Position {
                width: Value::Value(Container::Container, Values::Width, Portion::Half),
                height: Value::Value(Container::Container, Values::Height, Portion::Half),
                container: Container::Container,
            }),
            origin: pos(Position {
                width: Value::Value(Container::This, Values::Width, Portion::Half),
                height: Value::Value(Container::This, Values::Height, Portion::Half),
                container: Container::This,
            }),
            grad_linear: opt_grad.clone(),
            grad_radial: opt_grad,
            image: opt_img,
            image_tint: color(Colors::ALPHA_FULL),
            scroll_y: val(Value::Zero),
            scroll_x: val(Value::Zero),
            overflow: overflow(Overflow::Shown),
            padding: val(Value::Zero),
            margin: opt_val(None),
        }
    }
}

impl Value {
    pub(crate) fn calc(
        &self,
        containers: &Containers,
        variables: &mut Variables,
    ) -> f32 {
        match self {
            Self::Value(c, v, p) => {
                let c = match c {
                    Container::Container => containers.container.size,
                    Container::ViewPort => containers.vp.size,
                    Container::This => containers.this.size,
                    Container::Image => *containers.image,
                };
                let p = match p {
                    Portion::Full => 1.0,
                    Portion::Half => 0.5,
                    Portion::Zero => 0.0,
                    Portion::Percent(p) => *p / 100.0,
                    Portion::Mul(p) => *p,
                    Portion::Div(p) => 1.0 / *p,
                };
                let v = match v {
                    Values::Width => c.0,
                    Values::Height => c.1,
                    Values::Diameter => (c.0 * c.0 + c.1 * c.1).sqrt(),
                    Values::Max => c.0.max(c.1),
                    Values::Min => c.0.min(c.1),
                    Values::Avg => (c.0 + c.1) / 2.0,
                };
                v * p
            }
            Self::Px(px) => *px,
            Self::Zero => 0.0,
            Self::Variable(key) => variables.get(*key).unwrap(),
            Self::SetVariable(key, value) => {
                let val = value.calc(containers, variables);
                variables.set(*key, val).unwrap()
            },
            Self::Time => containers.time,
            Self::Add(v) => {
                let v = v.as_ref();
                v.0.calc(containers, variables) + v.1.calc(containers, variables)
            }
            Self::Mul(v) => {
                let v = v.as_ref();
                v.0.calc(containers, variables) * v.1.calc(containers, variables)
            }
            Self::Debug(v, label) => {
                let value = v.calc(containers, variables);
                println!("Style: '{label:?}' = {value}px");
                value
            }
            Self::Negative(v) => -v.calc(containers, variables),
        }
    }
}

impl From<Vector> for Position {
    fn from(value: Vector) -> Self {
        Self {
            width: Value::Px(value.0),
            height: Value::Px(value.1),
            container: Container::This,
        }
    }
}

impl Position {
    pub(crate) fn calc(
        &self,
        containers: &Containers,
        variables: &mut Variables,
    ) -> Vector {
        let c = match self.container {
            Container::Container => containers.container,
            Container::ViewPort => containers.vp,
            Container::This => containers.this,
            Container::Image => &crate::element::Container {
                pos: containers.container.pos,
                size: *containers.image,
                rotation: 0.0,
            }
        };

        let pos = Vector::new(self.width.calc(containers, variables), self.height.calc(containers, variables));

        pos - c.size * 0.5 + c.pos
    }

    pub(crate) fn calc_rot(&self, containers: &Containers,
        variables: &mut Variables,) -> Vector {
        let c = match self.container {
            Container::Container => containers.container,
            Container::ViewPort => containers.vp,
            Container::This => containers.this,
            Container::Image => &crate::element::Container {
                pos: containers.container.pos,
                size: *containers.image,
                rotation: containers.this.rotation,
            }
        };

        let x = self.width.calc(containers, variables);
        let y = self.height.calc(containers, variables);
        let rot =
            Vector::new(x - c.size.0 * 0.5, y - c.size.1 * 0.5).rotate_around_origin(c.rotation);

        Vector::new(c.pos.0, c.pos.1) + rot
    }

    pub(crate) fn calc_relative(
        &self,
        containers: &Containers,
        variables: &mut Variables,
    ) -> Vector {
        let c = match self.container {
            Container::Container => containers.container,
            Container::ViewPort => containers.vp,
            Container::This => containers.this,
            Container::Image => &crate::element::Container {
                pos: containers.container.pos,
                size: *containers.image,
                rotation: 0.0,
            }
        };
        Vector::new(self.width.calc(containers, variables), self.height.calc(containers, variables)) - c.size * 0.5
    }
}

impl Rotation {
    pub(crate) fn calc(
        &self,
        containers: &Containers,
        variables: &mut Variables,
    ) -> f32 {
        let c = match self.cont {
            Container::Container => containers.container.rotation,
            Container::ViewPort => containers.vp.rotation,
            Container::This => containers.this.rotation,
            Container::Image => 0.0,
        };
        match &self.rot {
            Rotations::None => c,
            Rotations::Deg(v) => c + v.to_radians(),
            Rotations::Rad(v) => c + v,
            Rotations::CalcDeg(val) => c + val.calc(containers, variables).to_radians(),
            Rotations::CalcRad(val) => c + val.calc(containers, variables),
        }
    }
}

impl<T: Debug + Clone> StyleComponent<T> {
    pub fn new(v: T) -> Self {
        Self {
            val: v,
            dirty: false,
            dynamic: false,
        }
    }

    pub fn get(&self) -> &T {
        &self.val
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.dirty = true;
        &mut self.val
    }

    pub fn set(&mut self, val: T) {
        self.val = val;
        self.dirty = true;
    }

    pub fn set_dirty(&mut self) {
        self.dirty = true;
    }

    pub(crate) fn fix_dirty(&mut self) -> Option<&T> {
        if !self.dirty {
            return None;
        }
        self.dirty = self.dynamic;
        Some(&self.val)
    }

    pub(crate) fn fix_dirty_force(&mut self) -> &T {
        self.dirty = self.dynamic;
        &self.val
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn is_dirty_clear(&mut self) -> bool {
        let d = self.dirty;
        self.dirty = self.dynamic;
        d
    }

    pub fn is_dynamic(&self) -> bool {
        self.dynamic
    }

    pub fn set_dynamic(&mut self, dynamic: bool) {
        if dynamic {
            self.dirty = true;
        }
        self.dynamic = dynamic;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn style_enum_validity() {
        let style = Style::Height;
        let styles: Styles<()> = Styles::default();

        match style {
            Style::Height => {
                let _ = styles.height;
            }
            Style::Width => {
                let _ = styles.width;
            }
            Style::Color => {
                let _ = styles.color;
            }
            Style::Rotation => {
                let _ = styles.rotation;
            }
            Style::Round => {
                let _ = styles.round;
            }
            Style::Alpha => {
                let _ = styles.alpha;
            }
            Style::Center => {
                let _ = styles.position;
            }
            Style::Align => {
                let _ = styles.origin;
            }
            Style::MaxHeight => {
                let _ = styles.max_height;
            }
            Style::MaxWidth => {
                let _ = styles.max_width;
            }
            Style::MinHeight => {
                let _ = styles.min_height;
            }
            Style::MinWidth => {
                let _ = styles.min_width;
            }
            Style::GradRadial => {
                let _ = styles.grad_radial;
            }
            Style::GradLinear => {
                let _ = styles.grad_linear;
            }
            Style::Image => {
                let _ = styles.image;
            }
            Style::ImageTint => {
                let _ = styles.image_tint;
            }
            Style::ScrollY => {
                let _ = styles.scroll_y;
            }
            Style::ScrollX => {
                let _ = styles.scroll_x;
            }
            Style::Overflow => {
                let _ = styles.overflow;
            }
            Style::Padding => {
                let _ = styles.padding;
            }
            Style::Margin => {
                let _ = styles.margin;
            }
        }

        let Styles {
            width,
            height,
            color,
            rotation,
            round,
            alpha,
            position: center,
            origin: align,
            max_width,
            min_width,
            max_height,
            min_height,
            grad_radial,
            grad_linear,
            image,
            image_tint,
            scroll_y,
            scroll_x,
            overflow,
            margin,
            padding,
        } = styles;
        let _ = (width, Style::Width);
        let _ = (height, Style::Height);
        let _ = (color, Style::Color);
        let _ = (rotation, Style::Rotation);
        let _ = (round, Style::Round);
        let _ = (alpha, Style::Alpha);
        let _ = (center, Style::Center);
        let _ = (align, Style::Align);
        let _ = (max_height, Style::MaxHeight);
        let _ = (max_width, Style::MaxWidth);
        let _ = (min_height, Style::MinHeight);
        let _ = (min_width, Style::MinWidth);
        let _ = (grad_radial, Style::GradRadial);
        let _ = (grad_linear, Style::GradLinear);
        let _ = (image, Style::Image);
        let _ = (image_tint, Style::ImageTint);
        let _ = (scroll_y, Style::ScrollY);
        let _ = (scroll_x, Style::ScrollX);
        let _ = (overflow, Style::Overflow);
        let _ = (padding, Style::Padding);
        let _ = (margin, Style::Margin);
    }
}
