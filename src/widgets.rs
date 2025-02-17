use crate::{
    events::{ElemEvents, SelectOpts},
    styles::Container,
    variables::Variable,
    ElemEvent, ElemEventTypes, Element, ElementKey, EventListener, Gui, ImageData, MouseButtons,
    Overflow, Portion, Position, SelectionStates, Styles, Value, Values, Vector,
};

pub type OnEvent<Msg, Img, Data, Response> = fn(&mut EventArgs<Msg, Img, Data>) -> Response;

pub struct EventArgs<'a, Msg: Clone, Img: ImageData + Clone, Data> {
    pub element_key: ElementKey,
    pub gui: &'a mut Gui<Msg, Img>,
    pub data: &'a mut Data,
    pub mouse_based: bool,
}

pub struct Responses<Response: Clone> {
    responses: Vec<Response>,
    len: usize,
}

impl<Response: Clone> Responses<Response> {
    pub fn get(&mut self) -> &[Response] {
        let len = self.len;
        self.len = 0;
        &self.responses[..len]
    }

    fn add(&mut self, response: Response) {
        match self.responses.get_mut(self.len) {
            None => self.responses.push(response),
            Some(res) => *res = response,
        }
        self.len += 1;
    }
}

impl<'a, Msg: Clone, Img: ImageData + Clone, Data> EventArgs<'a, Msg, Img, Data> {
    pub fn element(&self) -> &Element<Msg, Img> {
        self.gui.get_element_unchecked(self.element_key)
    }

    pub fn element_mut(&mut self) -> &mut Element<Msg, Img> {
        self.gui.get_element_mut_unchecked(self.element_key)
    }

    pub fn styles(&self) -> &Styles<Img> {
        self.element().styles()
    }

    pub fn styles_mut(&mut self) -> &mut Styles<Img> {
        self.element_mut().styles_mut()
    }
}

pub struct WidgetManager<Msg: Clone, Img: Clone + ImageData, Data, Response: Clone = ()> {
    pub msg: fn(WidgetMsgs<Msg, Img, Data, Response>) -> Msg,
    pub responses: Responses<Response>,
}

impl<Msg: Clone, Img: Clone + ImageData, Data, Response: Clone>
    WidgetManager<Msg, Img, Data, Response>
{
    pub fn new(_: &Gui<Msg, Img>, msg: fn(WidgetMsgs<Msg, Img, Data, Response>) -> Msg) -> Self {
        Self {
            msg,
            responses: Responses {
                responses: Vec::new(),
                len: 0,
            },
        }
    }

    pub fn button(
        &self,
        element: &mut Element<Msg, Img>,
        confirm: OnEvent<Msg, Img, Data, Response>,
        enter: OnEvent<Msg, Img, Data, Response>,
        leave: OnEvent<Msg, Img, Data, Response>,
    ) {
        let msg = (self.msg)(WidgetMsgs::Button {
            confirm,
            enter,
            leave,
        });
        element
            .events
            .add(EventListener::new(ElemEventTypes::Selection).with_msg(msg.clone()));
        element
            .events
            .add(EventListener::new(ElemEventTypes::MouseEnter).with_msg(msg.clone()));
        element
            .events
            .add(EventListener::new(ElemEventTypes::MouseLeave).with_msg(msg.clone()));
        element
            .events
            .add(EventListener::new(ElemEventTypes::Click).with_msg(msg));
    }

    pub fn hover(
        &self,
        element: &mut Element<Msg, Img>,
        enter: OnEvent<Msg, Img, Data, Response>,
        leave: OnEvent<Msg, Img, Data, Response>,
    ) {
        element.events.add(
            EventListener::new(ElemEventTypes::MouseEnter)
                .with_msg((self.msg)(WidgetMsgs::Hover(enter, leave))),
        );
        element.events.add(
            EventListener::new(ElemEventTypes::MouseLeave)
                .with_msg((self.msg)(WidgetMsgs::Hover(enter, leave))),
        );
    }

    pub fn horizontal_split(
        &self,
        gui: &mut Gui<Msg, Img>,
        parent: ElementKey,
        left: ElementKey,
        right: ElementKey,
        opt: &SplitOptions,
        beam_press: OnEvent<Msg, Img, Data, Response>,
        beam_release: OnEvent<Msg, Img, Data, Response>,
    ) {
        let parent_e = gui.get_element_mut_unchecked(parent);
        match opt {
            SplitOptions::Dynamic { split, beam } => {
                let value = split.unwrap_or(0.5);
                parent_e
                .events
                .add(
                    EventListener::new(ElemEventTypes::MouseMove).with_msg((self.msg)(
                        WidgetMsgs::SplitBeam {
                            parent,
                            left,
                            right,
                            beam: *beam
                        },
                    )),
                );
                let beam_e = gui.get_element_mut_unchecked(*beam);
                beam_e.events.add(
                    EventListener::new(ElemEventTypes::Click)
                        .with_msg((self.msg)(WidgetMsgs::Hold { press: beam_press, release: beam_release})),
                );
                beam_e.styles_mut().position.get_mut().width =
                    Value::Value(Container::Container, Values::Width, Portion::Mul(value));
                let beam_width = beam_e.styles().width.get().clone();
                let half_beam_width = Value::Mul(Box::new((beam_width.clone(), Value::Px(0.5))));
                let splits_width = Value::Value(Container::Container, Values::Width, Portion::Full);
                let left = gui.get_element_mut_unchecked(left);
                let left_styles = left.styles_mut();
                left_styles.width.set(Value::Sub(Box::new((
                    Value::Mul(Box::new((splits_width.clone(), Value::Px(value)))),
                    half_beam_width.clone(),
                ))));
                left_styles.position.get_mut().width = Value::Px(0.0);
                left_styles.origin.get_mut().width = Value::Px(0.0);
                let right = gui.get_element_mut_unchecked(right);
                let right_styles = right.styles_mut();
                right_styles.width.set(Value::Sub(Box::new((
                    Value::Mul(Box::new((splits_width, Value::Px(1.0 - value)))),
                    half_beam_width,
                ))));
                right_styles.position.get_mut().width =
                    Value::Value(Container::Container, Values::Width, Portion::Full);
                right_styles.origin.get_mut().width =
                    Value::Value(Container::This, Values::Width, Portion::Full);
                let parent_e = gui.get_element_mut_unchecked(parent);
                parent_e.add_child(*beam);
            }
            SplitOptions::Fixed(value) => {
                let value = match value {
                    Some(value) => value,
                    None => &Value::Value(Container::Container, Values::Width, Portion::Half),
                };
                let left = gui.get_element_mut_unchecked(left);
                let left_styles = left.styles_mut();
                left_styles.width.set(value.clone());
                left_styles.position.get_mut().width =
                    Value::Value(Container::This, Values::Width, Portion::Full);
                left_styles.origin.get_mut().width =
                    Value::Value(Container::This, Values::Width, Portion::Full);
                let right = gui.get_element_mut_unchecked(right);
                let right_styles = right.styles_mut();
                right_styles.width.set(value.clone());
                right_styles.position.get_mut().width =
                    Value::Value(Container::This, Values::Width, Portion::Full);
                right_styles.origin.get_mut().width = Value::Px(0.0);
            }
        }
        let parent_e = gui.get_element_mut_unchecked(parent);
        parent_e.add_child(left);
        parent_e.add_child(right);
    }

    pub fn rows_builder(&self, rows: u32) -> RowsBuilder<Msg, Img, Data, Response> {
        let mut bldr = RowsBuilder::new(rows);
        bldr.events = Some(self.msg);
        bldr
    }

    pub fn columns_builder(&self, columns: u32) -> ColumnsBuilder<Msg, Img, Data, Response> {
        let mut bldr = ColumnsBuilder::new(columns);
        bldr.events = Some(self.msg);
        bldr
    }

    pub fn grid_builder(&self, columns: u32, rows: u32) -> GridBuilder<Msg, Img, Data, Response> {
        let mut bldr = GridBuilder::new(columns, rows);
        bldr.events = Some(self.msg);
        bldr
    }

    pub fn action(
        &mut self,
        msg: &WidgetMsgs<Msg, Img, Data, Response>,
        event: &ElemEvent<Msg>,
        gui: &mut Gui<Msg, Img>,
        data: &mut Data,
    ) -> &[Response] {
        msg.action(event, gui, data, &mut self.responses);
        self.responses.get()
    }
}

pub enum SplitOptions {
    Dynamic {
        split: Option<f32>,
        beam: ElementKey,
    },
    Fixed(Option<Value>),
}

#[derive(Debug, Clone, Copy)]
pub enum WidgetMsgs<Msg: Clone, Img: Clone + ImageData, Data, Response: Clone = ()> {
    Scroll(ScrollBounds, ScrollModifier, Option<Response>),
    Hover(
        OnEvent<Msg, Img, Data, Response>,
        OnEvent<Msg, Img, Data, Response>,
    ),
    Button {
        confirm: OnEvent<Msg, Img, Data, Response>,
        enter: OnEvent<Msg, Img, Data, Response>,
        leave: OnEvent<Msg, Img, Data, Response>,
    },
    SplitBeam {
        parent: ElementKey,
        left: ElementKey,
        right: ElementKey,
        beam: ElementKey,
    },
    Hold {
        press: OnEvent<Msg, Img, Data, Response>,
        release: OnEvent<Msg, Img, Data, Response>,
    },
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Scroll<Response: Clone> {
    pub modifier: ScrollModifier,
    pub response: Option<Response>,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ScrollModifier {
    Multiply(f32),
    MultiplyVec(Vector),
    Callback(fn(Vector) -> Vector),
    #[default]
    None,
}

impl<Response: Clone> Scroll<Response> {
    pub fn from_multiplier(mult: f32) -> Self {
        Self {
            modifier: ScrollModifier::Multiply(mult),
            response: None,
        }
    }

    pub fn with_response(mut self, response: Response) -> Self {
        self.response = Some(response);
        self
    }
}

impl ScrollModifier {
    pub fn modify(&self, scroll: Vector) -> Vector {
        match self {
            ScrollModifier::Multiply(v) => scroll * *v,
            ScrollModifier::MultiplyVec(vector) => scroll * *vector,
            ScrollModifier::Callback(cb) => cb(scroll),
            ScrollModifier::None => scroll,
        }
    }
}

impl<Msg: Clone, Img: Clone + ImageData, Data, Response: Clone>
    WidgetMsgs<Msg, Img, Data, Response>
{
    fn action(
        &self,
        event: &ElemEvent<Msg>,
        gui: &mut Gui<Msg, Img>,
        data: &mut Data,
        responses: &mut Responses<Response>,
    ) {
        macro_rules! args {
            ($mouse: expr) => {
                &mut EventArgs {
                    data,
                    element_key: event.element_key,
                    gui,
                    mouse_based: $mouse,
                }
            };
        }
        match self {
            Self::Scroll(bounds, modifier, response) => {
                let element = match gui.get_element_mut(event.element_key) {
                    None => return,
                    Some(e) => e,
                };
                let delta = match event.kind.get_scroll_delta() {
                    None => return,
                    Some(d) => modifier.modify(d),
                };
                bounds.scroll(element, delta);
                if let Some(res) = response {
                    responses.add(res.clone());
                }
            }
            Self::Hover(enter, leave) => match event.kind {
                crate::ElemEvents::CursorEnter { .. } => responses.add(enter(args!(true))),
                crate::ElemEvents::CursorLeave { .. } => responses.add(leave(args!(true))),
                _ => (),
            },
            WidgetMsgs::Button {
                confirm,
                enter,
                leave,
            } => match event.kind {
                crate::ElemEvents::Click {
                    button: MouseButtons::Left,
                    press: true,
                    ..
                } => {
                    responses.add(confirm(args!(true)));
                    responses.add(leave(args!(true)));
                }
                crate::ElemEvents::CursorEnter { .. } => responses.add(enter(args!(true))),
                crate::ElemEvents::CursorLeave { .. } => responses.add(leave(args!(true))),
                crate::ElemEvents::Selection {
                    state: SelectionStates::Confirm,
                } => {
                    responses.add(confirm(args!(false)));
                    responses.add(leave(args!(true)));
                }
                crate::ElemEvents::Selection {
                    state: SelectionStates::Enter,
                } => responses.add(enter(args!(false))),
                crate::ElemEvents::Selection {
                    state: SelectionStates::Leave,
                } => responses.add(leave(args!(false))),
                _ => (),
            },
            WidgetMsgs::SplitBeam {
                parent,
                left,
                right,
                beam,
            } => match event.kind {
                ElemEvents::CursorMove {
                    pos,
                    prev_pos,
                    vp_pos,
                } => {
                    if gui.selection.current != Some(*beam) {
                        return;
                    }
                    let parent_e = gui.get_element_mut_unchecked(*parent);
                    let (_, hit) = vp_pos.container_colision_with_pos(&parent_e.instance.container);
                    let size = parent_e.instance.container.size;
                    let value = ((hit + size * 0.5) / size).max(0.0).min(1.0);
                    let beam_e = gui.get_element_mut_unchecked(*beam);

                    beam_e.styles_mut().position.get_mut().width =
                        Value::Value(Container::Container, Values::Width, Portion::Mul(value.0));
                    let beam_width = beam_e.styles().width.get().clone();
                    let half_beam_width =
                        Value::Mul(Box::new((beam_width.clone(), Value::Px(0.5))));
                    let splits_width =
                        Value::Value(Container::Container, Values::Width, Portion::Full);
                    let left = gui.get_element_mut_unchecked(*left);
                    let left_styles = left.styles_mut();
                    left_styles.width.set(Value::Sub(Box::new((
                        Value::Mul(Box::new((splits_width.clone(), Value::Px(value.0)))),
                        half_beam_width.clone(),
                    ))));
                    left_styles.position.get_mut().width = Value::Px(0.0);
                    left_styles.origin.get_mut().width = Value::Px(0.0);
                    let right = gui.get_element_mut_unchecked(*right);
                    let right_styles = right.styles_mut();
                    right_styles.width.set(Value::Sub(Box::new((
                        Value::Mul(Box::new((splits_width, Value::Px(1.0 - value.0)))),
                        half_beam_width,
                    ))));
                    right_styles.position.get_mut().width =
                        Value::Value(Container::Container, Values::Width, Portion::Full);
                    right_styles.origin.get_mut().width =
                        Value::Value(Container::This, Values::Width, Portion::Full);
                }
                _ => (),
            },
            WidgetMsgs::Hold { press: enter, release: leave } => match event.kind {
                ElemEvents::Click { press: true, .. } => {
                    gui.env_event(crate::events::EnvEvents::Select {
                        opt: SelectOpts::SelectKey {
                            key: event.element_key,
                            force: true,
                        },
                    });
                    responses.add(enter(args!(true)));
                }
                ElemEvents::Click { press: false, .. } => {
                    gui.env_event(crate::events::EnvEvents::Select {
                        opt: SelectOpts::NoFocus,
                    });
                    responses.add(leave(args!(true)));
                }
                _ => (),
            },
        }
    }
}

pub struct GridBuilder<Msg: Clone, Img: Clone + ImageData, Data, Response: Clone> {
    pub columns: u32,
    pub rows: u32,
    pub count: Option<u32>,
    pub scroll: Option<Scroll<Response>>,
    pub events: Option<fn(WidgetMsgs<Msg, Img, Data, Response>) -> Msg>,
    pub width_modifier: fn(Value) -> Value,
    pub height_modifier: fn(Value) -> Value,
}

pub struct RowsBuilder<Msg: Clone, Img: Clone + ImageData, Data, Response: Clone> {
    pub rows: u32,
    pub count: Option<u32>,
    pub scroll: Option<Scroll<Response>>,
    pub events: Option<fn(WidgetMsgs<Msg, Img, Data, Response>) -> Msg>,
    pub height_modifier: fn(Value) -> Value,
}
pub struct ColumnsBuilder<Msg: Clone, Img: Clone + ImageData, Data, Response: Clone> {
    pub columns: u32,
    pub count: Option<u32>,
    pub scroll: Option<Scroll<Response>>,
    pub events: Option<fn(WidgetMsgs<Msg, Img, Data, Response>) -> Msg>,
    pub width_modifier: fn(Value) -> Value,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollBounds {
    pub direction: ScrollDirection,
    pub top: f32,
    pub bot: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum ScrollDirection {
    Horizontal,
    Vertical,
    Plane,
}

pub enum WidgetControlFlow {
    Done,
    Discard,
}

impl<Msg: Clone, Img: Clone + ImageData, Data, Response: Clone>
    GridBuilder<Msg, Img, Data, Response>
{
    pub fn new(columns: u32, rows: u32) -> Self {
        Self {
            columns,
            rows,
            count: None,
            scroll: None,
            events: None,
            width_modifier: |v| v,
            height_modifier: |v| v,
        }
    }

    pub fn modify_width(mut self, cb: fn(Value) -> Value) -> Self {
        self.width_modifier = cb;
        self
    }

    pub fn modify_height(mut self, cb: fn(Value) -> Value) -> Self {
        self.height_modifier = cb;
        self
    }

    pub fn set_count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }

    pub fn with_scroll(mut self, scroll: Scroll<Response>) -> Self {
        self.scroll = Some(scroll);
        self
    }

    pub fn gen_scroll_bounds(&self) -> ScrollBounds {
        let top = 0.0;
        let bot = match self.count {
            None => 0.0,
            Some(count) => {
                let a = 1.0 / self.rows as f32;
                (a * (count / self.columns) as f32 - 1.0).max(0.0)
            }
        };
        ScrollBounds {
            top,
            bot,
            direction: ScrollDirection::Vertical,
        }
    }

    pub fn build(
        &self,
        container_cb: impl FnOnce(&mut Element<Msg, Img>, &mut Gui<Msg, Img>),
        mut for_each: impl FnMut(
            (u32, u32),
            &mut Element<Msg, Img>,
            &mut Gui<Msg, Img>,
        ) -> WidgetControlFlow,
        gui: &mut Gui<Msg, Img>,
    ) -> ElementKey {
        let width_var = gui.variables.new(Variable::new_var());
        let height_var = gui.variables.new(Variable::new_var());

        let mut container = Element::default();
        container.procedures.push(Value::SetVariable(
            width_var,
            Box::new((self.width_modifier)(Value::Value(
                Container::This,
                Values::Width,
                Portion::Mul(1.0 / self.columns as f32),
            ))),
        ));
        container.procedures.push(Value::SetVariable(
            height_var,
            Box::new((self.height_modifier)(Value::Value(
                Container::This,
                Values::Height,
                Portion::Mul(1.0 / self.rows as f32),
            ))),
        ));

        if let (Some(scroll), Some(msg)) = (&self.scroll, &self.events) {
            let bounds = self.gen_scroll_bounds();
            let msg = msg(WidgetMsgs::Scroll(
                bounds,
                scroll.modifier,
                scroll.response.clone(),
            ));
            container
                .events
                .add(EventListener::new(ElemEventTypes::Scroll).with_msg(msg));
            container.styles_mut().overflow.set(Overflow::Hidden);
            container.styles_mut().scroll_y.set(Value::Value(
                Container::This,
                Values::Height,
                Portion::Mul(0.0),
            ));
        };

        let mut children = Vec::new();
        let mut i = 0;
        let count = match self.count {
            Some(c) => c,
            None => self.columns * self.rows,
        };
        'a: for row in 0.. {
            for column in 0..self.columns {
                if i >= count {
                    break 'a;
                }
                let mut element = Element::default();

                let styles = element.styles_mut();
                styles.width.set(Value::Variable(width_var));
                styles.height.set(Value::Variable(height_var));
                styles.position.set(Position {
                    container: Container::Container,
                    height: Value::Add(Box::new((
                        Value::Value(
                            Container::Container,
                            Values::Height,
                            Portion::Mul(row as f32 / self.rows as f32),
                        ),
                        Value::Mul(Box::new((Value::Variable(height_var), Value::Px(0.5)))),
                    ))),
                    width: Value::Add(Box::new((
                        Value::Value(
                            Container::Container,
                            Values::Width,
                            Portion::Mul(column as f32 / self.columns as f32),
                        ),
                        Value::Mul(Box::new((Value::Variable(width_var), Value::Px(0.5)))),
                    ))),
                });

                i += 1;
                match for_each((column, row), &mut element, gui) {
                    WidgetControlFlow::Discard => continue,
                    WidgetControlFlow::Done => (),
                };

                children.push(gui.add_element(element));
            }
        }
        container.children = Some(children);

        container_cb(&mut container, gui);

        gui.add_element(container)
    }
}

impl<Msg: Clone, Img: Clone + ImageData, Data, Response: Clone>
    RowsBuilder<Msg, Img, Data, Response>
{
    fn new(rows: u32) -> Self {
        Self {
            rows,
            count: None,
            events: None,
            scroll: None,
            height_modifier: |v| v,
        }
    }

    pub fn modify_height(mut self, cb: fn(Value) -> Value) -> Self {
        self.height_modifier = cb;
        self
    }

    pub fn set_count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }

    pub fn with_scroll(mut self, scroll: Scroll<Response>) -> Self {
        self.scroll = Some(scroll);
        self
    }

    pub fn gen_scroll_bounds(&self) -> ScrollBounds {
        let top = 0.0;
        let bot = match self.count {
            None => 0.0,
            Some(count) => {
                let a = 1.0 / self.rows as f32;
                (a * count as f32 - 1.0).max(0.0)
            }
        };
        ScrollBounds {
            top,
            bot,
            direction: ScrollDirection::Vertical,
        }
    }

    pub fn build(
        &self,
        container_cb: impl FnOnce(&mut Element<Msg, Img>, &mut Gui<Msg, Img>),
        mut for_each: impl FnMut(u32, &mut Element<Msg, Img>, &mut Gui<Msg, Img>) -> WidgetControlFlow,
        gui: &mut Gui<Msg, Img>,
    ) -> ElementKey {
        let height_var = gui.variables.new(Variable::new_var());

        let mut container = Element::default();
        container.procedures.push(Value::SetVariable(
            height_var,
            Box::new((self.height_modifier)(Value::Value(
                Container::This,
                Values::Height,
                Portion::Mul(1.0 / self.rows as f32),
            ))),
        ));

        if let (Some(scroll), Some(msg)) = (&self.scroll, &self.events) {
            let bounds = self.gen_scroll_bounds();
            let msg = msg(WidgetMsgs::Scroll(
                bounds,
                scroll.modifier,
                scroll.response.clone(),
            ));
            container
                .events
                .add(EventListener::new(ElemEventTypes::Scroll).with_msg(msg));
            container.styles_mut().overflow.set(Overflow::Hidden);
            container.styles_mut().scroll_y.set(Value::Value(
                Container::This,
                Values::Height,
                Portion::Mul(0.0),
            ));
        };

        let mut children = Vec::new();
        let count = match self.count {
            Some(c) => c,
            None => self.rows,
        };
        for row in 0..count {
            let mut element = Element::default();

            let styles = element.styles_mut();
            styles.height.set(Value::Variable(height_var));
            styles.position.set(Position {
                container: Container::Container,
                height: Value::Add(Box::new((
                    Value::Value(
                        Container::Container,
                        Values::Height,
                        Portion::Mul(row as f32 / self.rows as f32),
                    ),
                    Value::Mul(Box::new((Value::Variable(height_var), Value::Px(0.5)))),
                ))),
                width: Value::Value(Container::Container, Values::Width, Portion::Half),
            });

            match for_each(row, &mut element, gui) {
                WidgetControlFlow::Discard => continue,
                WidgetControlFlow::Done => (),
            };

            children.push(gui.add_element(element));
        }
        container.children = Some(children);

        container_cb(&mut container, gui);

        gui.add_element(container)
    }
}

impl<Msg: Clone, Img: Clone + ImageData, Data, Response: Clone>
    ColumnsBuilder<Msg, Img, Data, Response>
{
    fn new(columns: u32) -> Self {
        Self {
            columns,
            count: None,
            events: None,
            scroll: None,
            width_modifier: |v| v,
        }
    }

    pub fn modify_height(mut self, cb: fn(Value) -> Value) -> Self {
        self.width_modifier = cb;
        self
    }

    pub fn set_count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }

    pub fn with_scroll(mut self, scroll: Scroll<Response>) -> Self {
        self.scroll = Some(scroll);
        self
    }

    pub fn gen_scroll_bounds(&self) -> ScrollBounds {
        let top = 0.0;
        let bot = match self.count {
            None => 0.0,
            Some(count) => {
                let a = 1.0 / self.columns as f32;
                (a * count as f32 - 1.0).max(0.0)
            }
        };
        ScrollBounds {
            top,
            bot,
            direction: ScrollDirection::Horizontal,
        }
    }

    pub fn build(
        &self,
        container_cb: impl FnOnce(&mut Element<Msg, Img>, &mut Gui<Msg, Img>),
        mut for_each: impl FnMut(u32, &mut Element<Msg, Img>, &mut Gui<Msg, Img>) -> WidgetControlFlow,
        gui: &mut Gui<Msg, Img>,
    ) -> ElementKey {
        let width_var = gui.variables.new(Variable::new_var());

        let mut container = Element::default();
        container.procedures.push(Value::SetVariable(
            width_var,
            Box::new((self.width_modifier)(Value::Value(
                Container::This,
                Values::Width,
                Portion::Mul(1.0 / self.columns as f32),
            ))),
        ));

        if let (Some(scroll), Some(msg)) = (&self.scroll, &self.events) {
            let bounds = self.gen_scroll_bounds();
            let msg = msg(WidgetMsgs::Scroll(
                bounds,
                scroll.modifier,
                scroll.response.clone(),
            ));
            container
                .events
                .add(EventListener::new(ElemEventTypes::Scroll).with_msg(msg));
            container.styles_mut().overflow.set(Overflow::Hidden);
            container.styles_mut().scroll_x.set(Value::Value(
                Container::This,
                Values::Width,
                Portion::Mul(0.0),
            ));
        };

        let mut children = Vec::new();
        let count = match self.count {
            Some(c) => c,
            None => self.columns,
        };
        for row in 0..count {
            let mut element = Element::default();

            let styles = element.styles_mut();
            styles.width.set(Value::Variable(width_var));
            styles.position.set(Position {
                container: Container::Container,
                width: Value::Add(Box::new((
                    Value::Value(
                        Container::Container,
                        Values::Width,
                        Portion::Mul(row as f32 / self.columns as f32),
                    ),
                    Value::Mul(Box::new((Value::Variable(width_var), Value::Px(0.5)))),
                ))),
                height: Value::Value(Container::Container, Values::Height, Portion::Half),
            });

            match for_each(row, &mut element, gui) {
                WidgetControlFlow::Discard => continue,
                WidgetControlFlow::Done => (),
            };

            children.push(gui.add_element(element));
        }
        container.children = Some(children);

        container_cb(&mut container, gui);

        gui.add_element(container)
    }
}

impl ScrollBounds {
    pub fn scroll<A: Clone, B: ImageData + Clone>(
        &self,
        element: &mut Element<A, B>,
        delta: Vector,
    ) {
        match self.direction {
            ScrollDirection::Horizontal => match element.styles_mut().scroll_x.get_mut() {
                Value::Value(_, _, Portion::Mul(mul)) => {
                    *mul = (*mul + delta.1).min(self.top).max(-self.bot)
                }
                _ => (),
            },
            ScrollDirection::Vertical => match element.styles_mut().scroll_y.get_mut() {
                Value::Value(_, _, Portion::Mul(mul)) => {
                    *mul = (*mul + delta.1).min(self.top).max(-self.bot)
                }
                _ => (),
            },
            ScrollDirection::Plane => {}
        }
    }
}
