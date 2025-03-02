use std::{fs::File, io::Read, num::NonZero, sync::Arc};

use common::{
    resize_event,
    rugui2_wgpu::{texture::Texture, Rugui2WGPU},
    rugui2_winit::EventContext, Drawing,
};
use rugui2::{
    element::{Element, ElementKey}, events::{self, ElemEventTypes, EventListener, Key}, styles::{self, Value}, text::{Font, TextRepr}, widgets::{WidgetManager, WidgetMsgs}, Gui
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

#[derive(Debug, Clone)]
pub enum Msgs {
    Widgets(WidgetMsgs<Msgs, Texture, (), ()>)
}

pub struct Program {
    pub window: Arc<Window>,
    pub gui: Gui<Msgs, Texture>,
    pub rt: Runtime,
    pub element_key: ElementKey,
    pub drawing: Drawing,
    pub renderer: Rugui2WGPU,
    pub events: EventContext,
    pub widgets: WidgetManager<Msgs, Texture, (), ()>
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("yomama - experimental").with_maximized(true).with_visible(false))
                .unwrap(),
        );
        let rt = Runtime::new().unwrap();
        let drawing = rt.block_on(Drawing::new(window.clone()));
        let renderer = Rugui2WGPU::new(&drawing.queue, &drawing.device, window.inner_size().into());

        let mut gui = Gui::new((
            NonZero::new(window.inner_size().width).unwrap(),
            NonZero::new(window.inner_size().height).unwrap(),
        ));
        gui.text_ctx.add_font(Font::from_bytes(include_bytes!("game/src/SpaceMono-Regular.ttf"), 0).unwrap());
        gui.text_ctx.add_font(Font::from_bytes(include_bytes!("game/src/NotoEmoji-Regular.ttf"), 0).unwrap());
        let widgets = WidgetManager::new(&gui, Msgs::Widgets);
        gui.selection.locked = true;

        let mut elem = Element::default();
        elem.events.add(EventListener::new(ElemEventTypes::Scroll));
        elem.events.add(EventListener::new(events::ElemEventTypes::TextInput));
        elem.events.add(EventListener::new(events::ElemEventTypes::KeyPress));
        elem.events.add(EventListener::new(events::ElemEventTypes::FileDrop));
        elem.styles_mut().text.set(Some(TextRepr::new_editor(include_str!("../rugui2_wgpu/src/shaders/glyph.wgsl"))));
        elem.styles_mut().font_size.set(Value::Px(14.0));
        elem.styles_mut().text_wrap.set(styles::TextWrap::Overflow);
        elem.styles_mut().scroll_y.set(Value::Px(0.0));


        let element_key = gui.add_element(elem);
        gui.set_entry(element_key);

        window.set_visible(true);

        let program = Program {
            window,
            widgets,
            gui,
            rt,
            element_key,
            drawing,
            renderer,
            events: EventContext::new(),
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

        this.events.event(&event, &mut this.gui);
        this.gui.prepare_events();
        while let Some(e) = this.gui.poll_event() {
            match e.kind {
                rugui2::events::ElemEvents::TextInput { text } => {
                    let elem = this.gui.get_element_mut_unchecked(e.element_key);
                    elem.styles_mut().text.get_mut().as_mut().unwrap().insert_str(&text);
                }
                rugui2::events::ElemEvents::Scroll { delta, pos: _ } => {
                    let elem = this.gui.get_element_mut_unchecked(e.element_key);
                    if this.events.pressed_ctrl {
                        if let Value::Px(px) = elem.styles_mut().font_size.get_mut() {
                            *px = (*px + 1.0 * delta.1).max(1.0);
                        }
                    }else if let Value::Px(px) = elem.styles_mut().scroll_y.get_mut() {
                        *px = (*px + 65.0 * delta.1).min(0.0);
                    }
                }
                events::ElemEvents::KeyPress { press: true, key } => {
                    match (this.events.pressed_ctrl, key) {
                        (true, Key::NumpadAdd) => {
                            if let Value::Px(px) = this.gui.get_element_mut_unchecked(this.element_key).styles_mut().font_size.get_mut() {
                                *px += 1.0;
                            }
                        }
                        (true, Key::NumpadSubtract) => {
                            if let Value::Px(px) = this.gui.get_element_mut_unchecked(this.element_key).styles_mut().font_size.get_mut() {
                                *px -= 1.0;
                            }
                        }
                        _=> ()
                    }
                }
                events::ElemEvents::FileDrop { path, pos: _ } => {
                    let elem = this.gui.get_element_mut_unchecked(this.element_key);
                    let mut txt = String::new();
                    match File::open(&path) {
                        Ok(mut file) => 
                        if let Err(err) = file.read_to_string(&mut txt) { txt = format!("Could not read file: {path:?}\nReason: {err:?}") }
                        Err(err) => txt = format!("Could open read file: {path:?}\nReason: {err:?}")
                    }

                    elem.styles_mut().text.set(Some(TextRepr::new_editor(&txt)));
                    this.window.request_redraw();
                }
                _ => println!("kind: {:?}", e.kind),
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
                this.gui.update(0.0);
                this.renderer
                    .prepare(&mut this.gui, &this.drawing.queue, &this.drawing.device);
                this.drawing.draw(&mut this.gui, &mut this.renderer);
                this.window.request_redraw();
            }
            _ => (),
        }
    }
}
