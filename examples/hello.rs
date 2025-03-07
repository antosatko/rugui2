use std::{num::NonZero, sync::Arc, time::Instant};

use common::{
    resize_event,
    rugui2_wgpu::{texture::Texture, Rugui2WGPU}, Drawing,
};
use rugui2::{
    colors::Colors,
    element::{Element, ElementKey},
    events::{ElemEventTypes, EventListener},
    styles::{Container, Overflow, Portion, Rotation, Value, Values},
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
    pub element_key2: ElementKey,
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
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let rt = Runtime::new().unwrap();
        let drawing = rt.block_on(Drawing::new(window.clone()));
        let renderer = Rugui2WGPU::new(&drawing.queue, &drawing.device, window.inner_size().into());

        let mut gui = Gui::new((
            NonZero::new(window.inner_size().width).unwrap(),
            NonZero::new(window.inner_size().height).unwrap(),
        ));

        let mut elem = Element::default();
        elem.label = Some(String::from("First"));
        elem.styles_mut().max_width.set(Some(Value::Px(500.0)));
        elem.styles_mut().color.set(Colors::RED);
        elem.styles_mut().width.set(Value::Value(
            Container::Container,
            Values::Width,
            Portion::Half,
        ));
        elem.styles_mut().height.set(Value::Value(
            Container::Container,
            Values::Height,
            Portion::Full,
        ));
        elem.styles_mut().rotation.set(Rotation {
            rot: rugui2::styles::Rotations::Deg(1.0),
            cont: Container::This,
        });
        elem.styles_mut().round.set(Some(Value::Value(Container::This, Values::Min, Portion::Half)));
        elem.styles_mut().scroll_y.set(Value::Px(0.0));
        elem.styles_mut().overflow.set(Overflow::Hidden);
        elem.events.add(EventListener {
            event: rugui2::events::ElemEventTypes::Scroll,
            msg: None,
            kind: rugui2::events::ListenerTypes::Listen,
        });

        let mut elem2 = Element::default();
        elem2.label = Some(String::from("Second"));
        /*elem2.styles_mut().width.set(Value::Value(
            Container::Container,
            Values::Width,
            Portion::Mul(0.65),
        ));*/
        elem2.styles_mut().height.set(Value::Value(
            Container::Container,
            Values::Height,
            Portion::Half,
        ));
        elem2.styles_mut().round.set(Some(Value::Value(Container::This, Values::Min, Portion::Half)));
        elem2.styles_mut().color.set(Colors::GREEN);
        elem2
            .events
            .add(EventListener::new(ElemEventTypes::MouseMove));
        let element_key2 = gui.add_element(elem2);

        elem.children = Some(vec![element_key2]);

        let element_key = gui.add_element(elem);
        gui.set_entry(element_key);

        let program = Program {
            window,
            gui,
            rt,
            element_key,
            element_key2,
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
        while let Some(e) = this.gui.poll_event() {
            match e.kind {
                /*rugui2::events::ElemEvents::CursorMove { pos, .. } => {
                    let elem = this.gui.get_element_mut(this.element_key2).unwrap();
                    let size = elem.instance().container.size;
                    let grad = elem.styles_mut().grad_radial.get_mut().as_mut().unwrap();
                    grad.p1.0 = (pos + size * 0.5).into();
                    grad.p2.0 = (pos + size * 0.5 + 50.0).into();
                }*/
                rugui2::events::ElemEvents::Scroll { delta, pos: _ } => {
                    let elem = this.gui.get_element_mut(e.element_key).unwrap();
                    if let Value::Px(px) = elem.styles_mut().scroll_y.get_mut() {
                        *px += delta.1 * 65.0;
                    }
                }
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
                //println!("FPS: {:?}", 1.0 / this.frame_start.elapsed().as_secs_f64());
                //this.frame_start = Instant::now();
                this.gui.update(this.program_start.elapsed().as_secs_f32());
                println!("update took: {:?}", start.elapsed());
                this.t += 1;
                //println!("t: {}", this.t);
                //let start = std::time::Instant::now();
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
