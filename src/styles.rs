use std::fmt::Debug;

use crate::{Colors, Vector};

#[derive(Debug, Clone)]
pub struct Styles <Img: Clone + ImageData> {
    pub width: StyleComponent<Value>,
    pub max_width: StyleComponent<Option<Value>>,
    pub min_width: StyleComponent<Option<Value>>,
    pub height: StyleComponent<Value>,
    pub max_height: StyleComponent<Option<Value>>,
    pub min_height: StyleComponent<Option<Value>>,
    pub color: StyleComponent<Colors>,
    pub rotation: StyleComponent<Rotation>,
    pub round: StyleComponent<Option<Round>>,
    pub alpha: StyleComponent<f32>,
    pub center: StyleComponent<Position>,
    pub align: StyleComponent<Position>,
    pub grad_linear: StyleComponent<Option<Gradient>>,
    pub grad_radial: StyleComponent<Option<Gradient>>,
    pub image: StyleComponent<Option<Image<Img>>>,
    pub image_tint: StyleComponent<Colors>,
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
}

#[derive(Clone)]
pub struct Image <Img: Clone + ImageData> {
    pub data: Img,
}

impl <Img: Clone + ImageData> std::fmt::Debug for Image<Img> {
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
    pub smooth: Value,
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
    Value(Container, Values, Portion),
    Debug(Box<Value>, Option<String>),
    Add(Box<(Value, Value)>),
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

#[derive(Debug, Copy, Clone, Default)]
pub struct Rotation {
    pub rot: Rotations,
    pub cont: Container,
}

#[derive(Debug, Copy, Clone, Default)]
pub enum Rotations {
    #[default]
    None,
    Deg(f32),
    Rad(f32),
}

#[derive(Debug, Copy, Clone, Default)]
pub enum Container {
    ViewPort,
    #[default]
    Container,
    This,
}

impl <Tex: ImageData + Clone> Default for Styles <Tex> {
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
            center: pos(Position {
                width: Value::Value(Container::Container, Values::Width, Portion::Half),
                height: Value::Value(Container::Container, Values::Height, Portion::Half),
                container: Container::Container,
            }),
            align: pos(Position {
                width: Value::Value(Container::This, Values::Width, Portion::Half),
                height: Value::Value(Container::This, Values::Height, Portion::Half),
                container: Container::This,
            }),
            grad_linear: opt_grad.clone(),
            grad_radial: opt_grad,
            image: opt_img,
            image_tint: color(Colors::ALPHA_FULL),
        }
    }
}

impl Value {
    pub(crate) fn calc(
        &self,
        container: &crate::element::Container,
        vp: &crate::element::Container,
        this: &crate::element::Container,
    ) -> f32 {
        match self {
            Self::Value(c, v, p) => {
                let c = match c {
                    Container::Container => container,
                    Container::ViewPort => vp,
                    Container::This => this,
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
                    Values::Width => c.size.0,
                    Values::Height => c.size.1,
                    Values::Diameter => (c.size.0 * c.size.0 + c.size.1 * c.size.1).sqrt(),
                    Values::Max => c.size.0.max(c.size.1),
                    Values::Min => c.size.0.min(c.size.1),
                    Values::Avg => (c.size.0 + c.size.1) / 2.0,
                };
                v * p
            }
            Self::Px(px) => *px,
            Self::Zero => 0.0,
            Self::Add(v) => {
                let v = v.as_ref();
                v.0.calc(container, vp, this) + v.1.calc(container, vp, this)
            }
            Self::Debug(v, label) => {
                let value = v.calc(container, vp, this);
                println!("Style: '{label:?}' = {value}px");
                value
            }
            Self::Negative(v) => -v.calc(container, vp, this),
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
        container: &crate::element::Container,
        vp: &crate::element::Container,
        this: &crate::element::Container,
    ) -> Vector {
        let c = match self.container {
            Container::Container => container,
            Container::ViewPort => vp,
            Container::This => this,
        };

        let pos = Vector::new(self.width.calc(c, vp, this), self.height.calc(c, vp, this));

        pos - c.size * 0.5 + c.pos
    }

    pub(crate) fn calc_rot(
        &self,
        container: &crate::element::Container,
        vp: &crate::element::Container,
        this: &crate::element::Container,
    ) -> Vector {
        let c = match self.container {
            Container::Container => container,
            Container::ViewPort => vp,
            Container::This => this,
        };

        let x = self.width.calc(c, vp, this);
        let y = self.height.calc(c, vp, this);
        let rot =
            Vector::new(x - c.size.0 * 0.5, y - c.size.1 * 0.5).rotate_around_origin(c.rotation);

        Vector::new(c.pos.0, c.pos.1) + rot
    }

    pub fn calc_relative(
        &self,
        container: &crate::element::Container,
        vp: &crate::element::Container,
        this: &crate::element::Container,
    ) -> Vector {
        let c = match self.container {
            Container::Container => container,
            Container::ViewPort => vp,
            Container::This => this,
        };
        Vector::new(self.width.calc(c, vp, this), self.height.calc(c, vp, this)) - c.size * 0.5
    }
}

impl Rotation {
    pub(crate) fn calc(
        &self,
        container: &crate::element::Container,
        vp: &crate::element::Container,
        this: &crate::element::Container,
    ) -> f32 {
        let c = match self.cont {
            Container::Container => container.rotation,
            Container::ViewPort => vp.rotation,
            Container::This => this.rotation,
        };
        match self.rot {
            Rotations::None => c,
            Rotations::Deg(v) => c + v.to_radians(),
            Rotations::Rad(v) => c + v,
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
                let _ = styles.center;
            }
            Style::Align => {
                let _ = styles.align;
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
        }

        let Styles {
            width,
            height,
            color,
            rotation,
            round,
            alpha,
            center,
            align,
            max_width,
            min_width,
            max_height,
            min_height,
            grad_radial,
            grad_linear,
            image,
            image_tint,
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
    }
}
