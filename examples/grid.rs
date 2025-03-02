use std::{num::NonZero, sync::Arc, time::Instant};

use common::{
    resize_event,
    rugui2_wgpu::{texture::Texture, Rugui2WGPU}, Drawing,
};
use rugui2::{
    widgets::{
        ScrollBounds,
        WidgetMsgs,
    },
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
    pub gui: Gui<Msgs, Texture>,
    pub rt: Runtime,
    pub drawing: Drawing,
    pub renderer: Rugui2WGPU,
    pub t: u64,
    pub frame_start: Instant,
    pub program_start: Instant,
}

#[derive(Debug, Clone)]
pub enum Msgs {
    Widgets(WidgetMsgs<Msgs, Texture, (), ()>),
    ScrollBounds(ScrollBounds),
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

        /*let container_builder = GridBuilder::new(5, 3)
            .with_scroll(Scroll::from_multiplier(0.1))
            .set_count(5 * 50)
            .with_auto_events(Msgs::Widgets);

        let inner_rows_builder = RowsBuilder::new(3)
            .set_count(10)
            .with_scroll(Scroll::from_multiplier(0.1))
            .with_auto_events(Msgs::Widgets);

        let inner_columns_builder = ColumnsBuilder::new(3)
            .set_count(10)
            .with_scroll(Scroll::from_multiplier(0.1))
            .with_auto_events(Msgs::Widgets);

        let container = container_builder.build(
            |container, _| {
                container
                    .events
                    .add(EventListener::new(ElemEventTypes::MouseMove));
                container.styles_mut().round.set(Some(Value::Px(50.0)));
                container.styles_mut().padding.set(Value::Value(
                    Container::Container,
                    Values::Min,
                    Portion::Mul(0.1),
                ));
            },
            |(x, y), element, gui| {
                element.allow_select = true;
                element
                    .events
                    .add(EventListener::new(ElemEventTypes::MouseEnter));
                element
                    .events
                    .add(EventListener::new(ElemEventTypes::MouseLeave));
                element
                    .events
                    .add(EventListener::new(ElemEventTypes::Click));

                let styles = element.styles_mut();

                styles.color.set(Colors::RED);
                styles.padding.set(Value::Value(
                    // it works now even without this yaaay
                    Container::This,
                    Values::Min,
                    Portion::Mul(0.01),
                ));

                let inner_list = if (x + y) % 2 == 0 {
                    inner_rows_builder.build(
                        |e, _| {
                            e.styles_mut().padding.set(Value::Px(50.0));
                        },
                        |row, element, _| {
                            let styles = element.styles_mut();

                            styles.padding.set(Value::Px(1.0));
                            styles.color.set(Colors::FRgba(
                                row as f32 * 0.1,
                                row as f32 * 0.1,
                                row as f32 * 0.1,
                                1.0,
                            ));

                            WidgetControlFlow::Done
                        },
                        gui,
                    )
                } else {
                    inner_columns_builder.build(
                        |e, _| {
                            e.styles_mut().padding.set(Value::Px(50.0));
                        },
                        |row, element, _| {
                            let styles = element.styles_mut();

                            styles.padding.set(Value::Px(1.0));
                            styles.color.set(Colors::FRgba(
                                row as f32 * 0.1,
                                row as f32 * 0.1,
                                row as f32 * 0.1,
                                1.0,
                            ));

                            WidgetControlFlow::Done
                        },
                        gui,
                    )
                };

                element.children = Some(vec![inner_list]);

                WidgetControlFlow::Done
            },
            &mut gui,
        );

        gui.set_entry(container);*/

        let program = Program {
            window,
            gui,
            rt,
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

        //rugui2_winit::event(&event, &mut this.gui);
        this.gui.prepare_events();
        /*while let Some(e) = this.gui.poll_event() {
            if let Some(Msgs::Widgets(action)) = &e.msg {
                //action.action(&e, &mut this.gui);
            }
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
                    rugui2::events::SelectionStates::Enter => element
                        .styles_mut()
                        .color
                        .set(Colors::FRgba(0.6, 0.0, 0.0, 1.0)),
                    rugui2::events::SelectionStates::Leave => {
                        element.styles_mut().color.set(Colors::RED)
                    }
                },
                ElemEvents::Scroll { delta, .. } => match &e.msg {
                    Some(Msgs::ScrollBounds(bounds)) => {
                        bounds.scroll(element, delta * 0.1);
                    }
                    _ => (),
                },
                ElemEvents::CursorMove { pos, .. } => {
                    let pos = pos / element.instance().container.size + 0.5;
                    element.styles_mut().grad_radial.set(Some(Gradient {
                        p1: (
                            Position {
                                container: Container::This,
                                height: Value::Value(
                                    Container::This,
                                    Values::Height,
                                    Portion::Mul(pos.1),
                                ),
                                width: Value::Value(
                                    Container::This,
                                    Values::Width,
                                    Portion::Mul(pos.0),
                                ),
                            },
                            Colors::YELLOW,
                        ),
                        p2: (
                            Position {
                                container: Container::This,
                                height: Value::Value(
                                    Container::This,
                                    Values::Height,
                                    Portion::Mul(pos.1 + 0.2),
                                ),
                                width: Value::Value(
                                    Container::This,
                                    Values::Width,
                                    Portion::Mul(pos.0 + 0.2),
                                ),
                            },
                            Colors::TRANSPARENT,
                        ),
                    }));
                }
                _ => (),
            }
        }*/

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
