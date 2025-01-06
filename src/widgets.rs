use crate::{
    styles::Container, variables::Variable, ElemEventTypes, Element, ElementKey, EventListener,
    Gui, ImageData, Overflow, Portion, Position, Value, Values,
};

pub struct GridBuilder<Msg: Clone> {
    pub columns: u32,
    pub rows: u32,
    pub count: Option<u32>,
    pub scrollable: bool,
    pub scroll_msg: Option<Msg>,
    pub width_modifier: fn(Value) -> Value,
    pub height_modifier: fn(Value) -> Value,
}
pub struct RowsBuilder<Msg: Clone> {
    pub rows: u32,
    pub count: Option<u32>,
    pub scrollable: bool,
    pub scroll_msg: Option<Msg>,
    pub height_modifier: fn(Value) -> Value,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollBounds {
    pub top: f32,
    pub bot: f32,
}

pub enum WidgetControlFlow {
    Done,
    Discard,
}

impl<Msg: Clone> GridBuilder<Msg> {
    pub fn new(columns: u32, rows: u32) -> Self {
        Self {
            columns,
            rows,
            count: None,
            scrollable: false,
            scroll_msg: None,
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

    pub fn set_scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }

    pub fn set_scroll_msg(mut self, msg: Msg) -> Self {
        self.scroll_msg = Some(msg);
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
        ScrollBounds { top, bot }
    }

    pub fn build<B: ImageData + Clone>(
        &self,
        container_cb: impl FnOnce(&mut Element<Msg, B>, &mut Gui<Msg, B>),
        mut for_each: impl FnMut(
            (u32, u32),
            &mut Element<Msg, B>,
            &mut Gui<Msg, B>,
        ) -> WidgetControlFlow,
        gui: &mut Gui<Msg, B>,
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

        if self.scrollable {
            match &self.scroll_msg {
                Some(msg) => container
                    .events
                    .add(EventListener::new(ElemEventTypes::Scroll).with_msg(msg.clone())),
                None => container
                    .events
                    .add(EventListener::new(ElemEventTypes::Scroll)),
            }
            container.styles_mut().overflow.set(Overflow::Hidden);
            container.styles_mut().scroll_y.set(Value::Value(
                Container::This,
                Values::Height,
                Portion::Mul(0.0),
            ));
            container.styles_mut().scroll_y.set_dynamic(true);
        }

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

impl<Msg: Clone> RowsBuilder<Msg> {
    pub fn new(rows: u32) -> Self {
        Self {
            rows,
            count: None,
            scrollable: false,
            scroll_msg: None,
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

    pub fn set_scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }

    pub fn set_scroll_msg(mut self, msg: Msg) -> Self {
        self.scroll_msg = Some(msg);
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
        ScrollBounds { top, bot }
    }

    pub fn build<B: ImageData + Clone>(
        &self,
        container_cb: impl FnOnce(&mut Element<Msg, B>, &mut Gui<Msg, B>),
        mut for_each: impl FnMut(
            u32,
            &mut Element<Msg, B>,
            &mut Gui<Msg, B>,
        ) -> WidgetControlFlow,
        gui: &mut Gui<Msg, B>,
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

        if self.scrollable {
            match &self.scroll_msg {
                Some(msg) => container
                    .events
                    .add(EventListener::new(ElemEventTypes::Scroll).with_msg(msg.clone())),
                None => container
                    .events
                    .add(EventListener::new(ElemEventTypes::Scroll)),
            }
            container.styles_mut().overflow.set(Overflow::Hidden);
            container.styles_mut().scroll_y.set(Value::Value(
                Container::This,
                Values::Height,
                Portion::Mul(0.0),
            ));
            container.styles_mut().scroll_y.set_dynamic(true);
        }

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

impl ScrollBounds {
    pub fn scroll<A: Clone, B: ImageData + Clone>(&self, element: &mut Element<A, B>, delta: f32) {
        match element.styles_mut().scroll_y.get_mut() {
            Value::Value(_, _, Portion::Mul(mul)) => {
                *mul = (*mul + delta).min(self.top).max(-self.bot)
            }
            _ => (),
        }
    }
}
