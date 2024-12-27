use std::{collections::HashMap, num::NonZero, sync::Arc, time::Instant};

use common::{resize_event, rugui2_wgpu::Rugui2WGPU, rugui2_winit, Drawing};
use rugui2::{
    colors::Colors,
    element::{Element, ElementKey},
    events::{ElemEvents, SelectionStates},
    styles::{Container, Portion, Position, Value, Values},
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
    pub gui: Gui,
    pub rt: Runtime,
    pub element_key: ElementKey,
    pub drawing: Drawing,
    pub renderer: Rugui2WGPU,
    pub t: u64,
    pub frame_start: Instant,
    pub text_fields: HashMap<ElementKey, String>,
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
        gui.selection.menu_accessibility = true;

        let mut elem = Element::default();
        elem.label = Some(String::from("Container"));
        const CHILDREN: f32 = 5.0;
        let mut children = Vec::new();
        let mut text_fields = HashMap::new();
        for i in 0..CHILDREN as u32 {
            let mut child = Element::default();
            child.label = Some(format!("Child: {i}"));

            child.allow_select = true;
            child.allow_text_input = true;

            let styles = child.styles_mut();
            let ratio = 1.0 / CHILDREN;
            styles.center.set(Position {
                width: Value::Value(Container::Container, Values::Width, Portion::Half),
                height: Value::Value(
                    Container::Container,
                    Values::Height,
                    Portion::Mul((ratio * i as f32) + ratio * 0.5),
                ),
                container: Container::Container,
            });
            styles.height.set(Value::Value(
                Container::Container,
                Values::Height,
                Portion::Mul(ratio),
            ));

            let child_key = gui.add_element(child);
            text_fields.insert(child_key, String::new());
            children.push(child_key);
        }
        elem.children = Some(children);

        let element_key = gui.add_element(elem);
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
            text_fields,
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
            match e.kind {
                ElemEvents::Selection { state } => match state {
                    SelectionStates::Confirm => println!("yaaaay"),
                    SelectionStates::Enter => {
                        if let Some(txt) = this.text_fields.get(&e.element_key) {
                            this.window.set_title(&txt);
                        }
                        println!("selected: {}", e.element_key.raw());
                        let e = this.gui.get_element_mut(e.element_key).unwrap();
                        e.styles_mut().color.set(Colors::ORANGE);
                    }
                    SelectionStates::Leave => {
                        if let Some(txt) = this.text_fields.get(&e.element_key) {
                            this.window.set_title(&txt);
                        }
                        println!("unselected: {}", e.element_key.raw());
                        let e = this.gui.get_element_mut(e.element_key).unwrap();
                        e.styles_mut().color.set(Colors::BLACK);
                    }
                },
                ElemEvents::TextInput { text } => match this.text_fields.get_mut(&e.element_key) {
                    Some(txt) => {
                        txt.push_str(&text);
                        this.window.set_title(&txt);
                    }
                    None => (),
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
                this.gui.update();
                this.renderer.prepare(&mut this.gui, &this.drawing.queue);
                this.drawing.draw(&mut this.gui, &mut this.renderer);
                this.window.request_redraw();
            }
            _ => (),
        }
    }
}