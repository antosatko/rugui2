use std::{
    num::NonZero,
    panic::catch_unwind,
    sync::Arc, time::Instant,
};

use common::{Drawing, resize_event};
use gui::init;
use rugui2::{text::Font, widgets::{WidgetManager, WidgetMsgs}, Gui};
use rugui2_wgpu::{Rugui2WGPU, texture::Texture};
use rugui2_winit::EventContext;

use winit::{
    application::ApplicationHandler, event_loop::EventLoop, platform::windows::WindowAttributesExtWindows, window::{CursorIcon, Window, WindowButtons}
};

mod gui;

fn main() {
    match catch_unwind(|| {
        let event_loop = EventLoop::with_user_event().build().unwrap();

        let mut app = WinitAgentIAmLosingIt::Loading;

        event_loop.run_app(&mut app).unwrap();
    }) {
        Err(_) => {
            println!("Something bad happened");
        }
        Ok(_) => (),
    }
}

pub enum WinitAgentIAmLosingIt {
    Loading,
    Running(App),
    Closing,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Control {
    LeftUp = 0,
    LeftDown,
    RightUp,
    RightDown,

    Count,
}

pub struct App {
    pub window: Arc<Window>,
    pub drawing: Drawing,
    pub gui_renderer: Rugui2WGPU,
    pub gui: Gui<Msgs, Texture>,
    pub events: EventContext,
    pub widgets: WidgetManager<Msgs, Texture, WidgetData, Actions>,
    widget_data: WidgetData,
    pub start_time: Instant,
}

#[derive(Debug, Copy, Clone)]
pub enum Actions {
    Cursor(CursorIcon),
    None
}

#[derive(Debug, Clone)]
pub struct WidgetData {
    pub window: Arc<Window>
}

#[derive(Clone, Debug)]
pub enum Msgs {
    Widgets(WidgetMsgs<Msgs, Texture, WidgetData, Actions>),
    Cursor(CursorIcon)
}

impl ApplicationHandler for WinitAgentIAmLosingIt {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_visible(false)
                        .with_title("deedit"),
                )
                .unwrap(),
        );

        let size = window.inner_size();
        let mut gui = Gui::new((
            NonZero::new(size.width).unwrap(),
            NonZero::new(size.height).unwrap(),
        ));
        gui.text_ctx.add_font(Font::from_bytes(include_bytes!("SpaceMono-Regular.ttf"), 0).unwrap());
        gui.text_ctx.add_font(Font::from_bytes(include_bytes!("NotoEmoji-Regular.ttf"), 0).unwrap());
        let drawing = pollster::block_on(Drawing::new(window.clone()));
        let gui_renderer = Rugui2WGPU::new(&drawing.queue, &drawing.device, size.into());
        let mut widgets = WidgetManager::new(&gui, Msgs::Widgets);
        let widget_data = WidgetData { window: window.clone() };
        
        init(&mut widgets, &mut gui);

        let events = EventContext::new();

        window.set_visible(true);

        let start_time = Instant::now();


        let app = App {
            window,
            drawing,
            gui,
            gui_renderer,
            events,
            widgets,
            widget_data,
            start_time,
        };

        *self = Self::Running(app);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let this = match self {
            WinitAgentIAmLosingIt::Running(app) => app,
            _ => return,
        };

        use winit::event::WindowEvent;
        match &event {
            WindowEvent::Resized(size) => {
                resize_event(&mut this.gui, &mut this.drawing, (*size).into());
            }
            WindowEvent::RedrawRequested => {
                let start = Instant::now();
                this.gui.update(this.start_time.elapsed().as_secs_f32());
                println!("update: {:?}", start.elapsed());
                this.gui_renderer
                    .prepare(&mut this.gui, &this.drawing.queue, &this.drawing.device);
                this.drawing.draw(&mut this.gui, &mut this.gui_renderer);
                this.window.request_redraw();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
                *self = Self::Closing;
            }
            e => {
                this.events.event(&event, &mut this.gui);
                this.gui.prepare_events();
                this.window.request_redraw();
                while let Some(e) = this.gui.poll_event() {
                    match &e.msg {
                        Some(Msgs::Widgets(action)) => {
                            for res in this
                                .widgets
                                .action(&action, &e, &mut this.gui, &mut this.widget_data)
                            {
                                match res {
                                    Actions::None => (),
                                    Actions::Cursor(cursor) => {
                                        this.window.set_cursor(*cursor);
                                    }
                                }
                            }
                        }
                        Some(Msgs::Cursor(cursor)) => {
                            this.window.set_cursor(*cursor);
                        }
                        None => match e.kind {
                            rugui2::events::ElemEvents::TextInput { text } => {
                                let elem = this.gui.get_element_mut_unchecked(e.element_key);
                                elem.styles_mut()
                                    .text
                                    .get_mut()
                                    .as_mut()
                                    .unwrap()
                                    .insert_str(&text);
                            }
                            kind => println!("event: {:?}", kind),
                        },
                    }
                }
            }
        }
    }
}
