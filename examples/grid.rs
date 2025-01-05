use std::{num::NonZero, sync::Arc, time::Instant};

use common::{
    resize_event,
    rugui2_wgpu::{texture::Texture, Rugui2WGPU},
    rugui2_winit, Drawing,
};
use rugui2::{
    colors::Colors,
    element::{Element, ElementKey},
    events::{ElemEventTypes, ElemEvents, EventListener, MouseButtons},
    styles::{Container, Gradient, Portion, Position, Round, Value, Values},
    variables::Variable,
    Gui,
};
use tokio::runtime::Runtime;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::Window,
};

pub fn main() {
    let event_loop = EventLoop::new().unwrap();

    let mut app = App::Loading;

    event_loop.run_app(&mut app).unwrap();
}

pub enum App {
    Loading,
    Running(Program),
}

pub struct Program {
    pub window: Arc<Window>,
    pub gui: Gui<(), Texture>,
    pub rt: Runtime,
    pub element_key: ElementKey,
    pub drawing: Drawing,
    pub renderer: Rugui2WGPU,
    pub t: u64,
    pub frame_start: Instant,
    pub program_start: Instant,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Grid"))
                .unwrap(),
        );
        let rt = Runtime::new().unwrap();
        let drawing = rt.block_on(Drawing::new(window.clone()));
        let renderer = Rugui2WGPU::new(&drawing.queue, &drawing.device, window.inner_size().into());

        let mut gui = Gui::new((
            NonZero::new(window.inner_size().width).unwrap(),
            NonZero::new(window.inner_size().height).unwrap(),
        ));
        gui.selection.menu_accessibility = true;

        const ROWS: u32 = 3;
        const COLUMNS: u32 = 5;
        const WIDTH: Option<Value> = None;
        const HEIGHT: Option<Value> = None;

        let width_var = gui.variables.new(Variable::new_var());
        let height_var = gui.variables.new(Variable::new_var());

        let mut container = Element::default();
        container
            .styles_mut()
            .overflow
            .set(rugui2::styles::Overflow::Hidden);
        container.styles_mut().scroll_y.set(Value::Value(
            Container::This,
            Values::Height,
            Portion::Mul(0.0),
        ));
        container.styles_mut().scroll_y.set_dynamic(true);
        container.styles_mut().grad_linear.set(Some(Gradient {
            p1: (
                Position {
                    container: Container::This,
                    height: Value::Zero,
                    width: Value::Zero,
                },
                Colors::RED.with_alpha(0.5),
            ),
            p2: (
                Position {
                    container: Container::This,
                    height: Value::Value(Container::This, Values::Height, Portion::Full),
                    width: Value::Zero,
                },
                Colors::BLACK,
            ),
        }));
        container.styles_mut().round.set(Some(Round {
            size: Value::Px(50.0),
            anti_aliasing: Value::Px(0.0),
        }));
        container.styles_mut().padding.set(Value::Value(
            Container::Container,
            Values::Min,
            Portion::Mul(0.1),
        ));
        container
            .events
            .add(EventListener::new(ElemEventTypes::Scroll));
        container.procedures.push(Value::SetVariable(
            width_var,
            Box::new(WIDTH.unwrap_or_else(|| {
                Value::Value(
                    Container::This,
                    Values::Width,
                    Portion::Mul(1.0 / COLUMNS as f32),
                )
            })),
        ));
        container.procedures.push(Value::SetVariable(
            height_var,
            Box::new(HEIGHT.unwrap_or_else(|| {
                Value::Value(
                    Container::This,
                    Values::Height,
                    Portion::Mul(1.0 / ROWS as f32),
                )
            })),
        ));
        let mut children = Vec::new();
        for row in 0..ROWS + 5 {
            for column in 0..COLUMNS {
                let mut element = Element::default();

                element.allow_select = true;
                element
                    .events
                    .add(EventListener::new(ElemEventTypes::Hover));
                element
                    .events
                    .add(EventListener::new(ElemEventTypes::Click));

                let styles = element.styles_mut();
                styles.position.set(Position {
                    container: Container::Container,
                    height: Value::Add(Box::new((
                        Value::Value(
                            Container::Container,
                            Values::Height,
                            Portion::Mul(row as f32 / ROWS as f32),
                        ),
                        Value::Mul(Box::new((Value::Variable(height_var), Value::Px(0.5)))),
                    ))),
                    width: Value::Add(Box::new((
                        Value::Value(
                            Container::Container,
                            Values::Width,
                            Portion::Mul(column as f32 / COLUMNS as f32),
                        ),
                        Value::Mul(Box::new((Value::Variable(width_var), Value::Px(0.5)))),
                    ))),
                });

                styles.width.set(Value::Variable(width_var));
                styles.height.set(Value::Variable(height_var));

                styles.color.set(Colors::RED);
                /*styles.padding.set(Value::Value( // it works now even without this yaaay
                    Container::This,
                    Values::Min,
                    Portion::Mul(0.1),
                ));*/
                styles.round.set(Some(Round {
                    size: Value::Px(50.0),
                    anti_aliasing: Value::Px(0.0),
                }));

                children.push(gui.add_element(element));
            }
        }
        container.children = Some(children);

        let element_key = gui.add_element(container);
        gui.set_entry(element_key);

        let program = Program {
            window,
            gui,
            rt,
            element_key,
            drawing,
            renderer,
            t: 0,
            frame_start: Instant::now(),
            program_start: Instant::now(),
        };
        *self = Self::Running(program)
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let this = match self {
            App::Loading => return,
            App::Running(this) => this,
        };

        rugui2_winit::event(&event, &mut this.gui);
        this.gui.prepare_events();
        while let Some(e) = this.gui.poll_event() {
            let element = this.gui.get_element_mut_unchecked(e.element_key);
            match e.kind {
                ElemEvents::CursorEnter { .. } => {
                    this.gui.env_event(rugui2::events::EnvEvents::Select {
                        opt: rugui2::events::SelectOpts::SelectKey(e.element_key),
                    });
                }
                ElemEvents::Click {
                    button: MouseButtons::Left,
                    press: true,
                    ..
                } => {
                    this.gui.env_event(rugui2::events::EnvEvents::Select {
                        opt: rugui2::events::SelectOpts::SelectKey(e.element_key),
                    });
                    this.gui.env_event(rugui2::events::EnvEvents::Select {
                        opt: rugui2::events::SelectOpts::Confirm,
                    });
                    this.gui.prepare_events();
                }
                ElemEvents::Selection { state } => match state {
                    rugui2::events::SelectionStates::Confirm => {
                        element.styles_mut().color.set(Colors::GREEN)
                    }
                    rugui2::events::SelectionStates::Enter => {
                        element.styles_mut().color.set(Colors::YELLOW)
                    }
                    rugui2::events::SelectionStates::Leave => {
                        element.styles_mut().color.set(Colors::RED)
                    }
                },
                ElemEvents::Scroll { delta, .. } => match element.styles_mut().scroll_y.get_mut() {
                    Value::Value(_, _, Portion::Mul(px)) => {
                        *px = (*px + delta.1 * 0.1).min(0.0).max(-5.0 / 3.0)
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        match event {
            WindowEvent::Resized(size) => {
                resize_event(&mut this.gui, &mut this.drawing, size.into());
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let start = std::time::Instant::now();
                this.gui.update(this.program_start.elapsed().as_secs_f32());
                println!("update took: {:?}", start.elapsed());
                this.t += 1;
                this.renderer
                    .prepare(&mut this.gui, &this.drawing.queue, &this.drawing.device);
                println!("prepare took: {:?}", start.elapsed());
                this.drawing.draw(&mut this.gui, &mut this.renderer);
                this.window.request_redraw();
            }
            _ => (),
        }
    }
}
