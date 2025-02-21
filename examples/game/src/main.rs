use std::{
    num::NonZero,
    panic::catch_unwind,
    rc::Rc,
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

use gui::{Actions, GuiManager, Pages};
use image::{EncodableLayout, GenericImageView};
use rugui2::{events::Key, rich_text::TextStyles, styles::Value, widgets::WidgetMsgs, Gui};
use rugui2_wgpu::{texture::Texture, Rugui2WGPU};
use rugui2_winit::EventContext;
use tokio::sync::{
    mpsc::{self, Sender},
    Mutex,
};

use winit::{
    application::ApplicationHandler,
    event_loop::{EventLoop, EventLoopProxy},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorIcon, Window},
};
use winit_controls::Controls;

mod drawing;
use drawing::*;
mod engine;
use engine::*;
mod game;
use game::*;
mod gui;

static RUNNING: AtomicBool = AtomicBool::new(true);

fn main() {
    match catch_unwind(|| {
        let event_loop = EventLoop::with_user_event().build().unwrap();
        let proxy = event_loop.create_proxy();

        let mut app = WinitAgentIAmLosingIt::Loading(proxy);

        event_loop.run_app(&mut app).unwrap();
    }) {
        Err(_) => {
            println!("Something bad happened");
            RUNNING.store(false, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(_) => (),
    }
}

pub enum WinitAgentIAmLosingIt {
    Loading(EventLoopProxy<Engine2Main>),
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
    pub rt: tokio::runtime::Runtime,
    pub drawing: Arc<Mutex<Drawing>>,
    pub controls: Arc<Mutex<Controls<Control>>>,
    pub gui_renderer: Rugui2WGPU,
    pub gui: Gui<Msgs, Texture>,
    pub gui_manager: GuiManager,
    pub start_time: Instant,
    pub game_transmiter: Sender<Main2Engine>,
    pub events: EventContext,
}

#[derive(Clone, Debug)]
pub enum Msgs {
    Widgets(WidgetMsgs<Msgs, Texture, (), Actions>),
    //GuiManager(Actions),
    Send(Main2Engine),
    Ingame,
    Settings,
}

impl ApplicationHandler<Engine2Main> for WinitAgentIAmLosingIt {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let proxy = match self {
            WinitAgentIAmLosingIt::Loading(proxy) => proxy.to_owned(),
            _ => panic!("yaaaaa"),
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_visible(false)
                        .with_maximized(true)
                        .with_title("Pong-Dong"),
                )
                .unwrap(),
        );

        let size = window.inner_size();
        let mut gui = Gui::new((
            NonZero::new(size.width).unwrap(),
            NonZero::new(size.height).unwrap(),
        ));
        let controls = Arc::new(Mutex::new(winit_controls::Controls::from_iter([
            (PhysicalKey::Code(KeyCode::KeyS), Control::LeftDown),
            (PhysicalKey::Code(KeyCode::KeyW), Control::LeftUp),
            (PhysicalKey::Code(KeyCode::ArrowDown), Control::RightDown),
            (PhysicalKey::Code(KeyCode::ArrowUp), Control::RightUp),
        ])));
        let drawing = rt.block_on(Drawing::new(window.clone()));
        let mut gui_renderer = Rugui2WGPU::new(&drawing.queue, &drawing.device, size.into());

        let dyn_imag = image::load_from_memory(include_bytes!("image.png")).unwrap();
        let imag = Texture::from_bytes(
            &drawing.device,
            &drawing.queue,
            dyn_imag.to_rgba8().as_bytes(),
            dyn_imag.dimensions(),
            None,
        );

        let gui_manager = GuiManager::new(&mut gui, drawing.game_tex.clone(), imag.unwrap());

        {
            let ctx = &mut gui.text_ctx;

            let mut text = rugui2::rich_text::Text::new();
            text.styles = Rc::new(TextStyles {
                align: 0.5,
                ..Default::default()
            });
            let mut section = rugui2::rich_text::TextSection::new("OH MY GAH! 😇 ");
            section.styles = std::rc::Rc::new(rugui2::rich_text::SectionStyles {
                color: [1.0, 0.0, 0.0, 1.0],
                ..Default::default()
            });
            text.sections.push(section);
            section = rugui2::rich_text::TextSection::new("I wish I were a bird. 🐦");
            section.styles = std::rc::Rc::new(rugui2::rich_text::SectionStyles {
                color: [0.0, 1.0, 0.0, 1.0],
                font: gui_manager.noto_font,
                font_size: 30.0,
                italic: true,
                ..Default::default()
            });
            text.sections.push(section);
            section = rugui2::rich_text::TextSection::new("Why are you speaking in English? 🫖");
            section.kind = rugui2::rich_text::SectionKinds::NewLine;
            section.styles = std::rc::Rc::new(rugui2::rich_text::SectionStyles {
                color: [0.0, 0.0, 1.0, 1.0],
                bold: true,
                font_size: 10.0,
                ..Default::default()
            });
            text.sections.push(section);
            section = rugui2::rich_text::TextSection::new("My daughter is going to America. 🍔");
            section.kind = rugui2::rich_text::SectionKinds::NewLine;
            section.styles = std::rc::Rc::new(rugui2::rich_text::SectionStyles {
                color: [0.0, 1.0, 1.0, 1.0],
                ..Default::default()
            });
            text.sections.push(section);

            let mut shape = rugui2::rich_text::TextShape::default();
            text.procces(
                ctx,
                Some(&mut shape),
                rugui2::text::Rect::new(0.0, 0.0, 500.0, 500.0),
            );
        }

        let drawing = Arc::new(Mutex::new(drawing));

        let (game_transmiter, game_reciever) = mpsc::channel(255);

        let clones = (window.clone(), drawing.clone(), controls.clone());
        rt.spawn(async {
            let engine = Engine::new(clones.0, clones.1, clones.2, game_reciever, proxy);
            engine.run().await;
        });

        let events = EventContext::new();

        window.set_visible(true);

        let start_time = Instant::now();

        let app = App {
            window,
            rt,
            drawing,
            controls,
            gui,
            gui_renderer,
            gui_manager,
            start_time,
            game_transmiter,
            events,
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
                this.rt.block_on(async {
                    let mut drawing = this.drawing.lock().await;
                    drawing.resize(&mut this.gui, (*size).into())
                });
            }
            WindowEvent::RedrawRequested => {
                let start = std::time::Instant::now();
                let elapsed = this.start_time.elapsed().as_secs_f32();
                {
                    let elem = this.gui.get_element_mut_unchecked(this.gui_manager.start_btn);
                    let text = elem.styles_mut().rich_text.get_mut().as_mut().unwrap();
                    for (i, s) in text.sections.iter_mut().enumerate() {
                        let mut style = *s.styles;
                        style.color[2] = (elapsed/2.0 + i as f32 / 6.0).sin().abs();
                        s.styles = Rc::new(style)
                    }
                }
                this.gui.update(elapsed);
                println!("update: {:?}", start.elapsed());
                this.rt.block_on(async {
                    let drawing = this.drawing.lock().await;
                    let start = std::time::Instant::now();
                    this.gui_renderer
                        .prepare(&mut this.gui, &drawing.queue, &drawing.device);
                    println!("prepare: {:?}", start.elapsed());
                    let start = std::time::Instant::now();
                    drawing.draw(&mut this.gui, &mut this.gui_renderer);
                    //println!("drawing: {:?}", start.elapsed());
                });
            }
            WindowEvent::CloseRequested => {
                RUNNING.store(false, std::sync::atomic::Ordering::Relaxed);
                event_loop.exit();
                *self = Self::Closing;
            }
            e => {
                this.rt.block_on(async {
                    this.controls.lock().await.update(&e);
                });
                this.events.event(&event, &mut this.gui);
                this.gui.prepare_events();
                while let Some(e) = this.gui.poll_event() {
                    match &e.msg {
                        Some(Msgs::Widgets(action)) => {
                            for res in this
                                .gui_manager
                                .widgets
                                .action(&action, &e, &mut this.gui, &mut ())
                                .to_vec()
                            {
                                match res {
                                    Actions::Button(name) => {
                                        this.window.set_cursor(CursorIcon::Default);
                                        match name {
                                            gui::ElementNames::Close => {
                                                RUNNING.store(
                                                    false,
                                                    std::sync::atomic::Ordering::Relaxed,
                                                );
                                                event_loop.exit();
                                            }
                                            gui::ElementNames::Orange => {
                                                this.gui_manager
                                                    .change_page(&mut this.gui, Pages::Settings);
                                            }
                                            gui::ElementNames::Start => {
                                                this.gui_manager
                                                    .change_page(&mut this.gui, Pages::Ingame);
                                                this.rt
                                                    .block_on(
                                                        this.game_transmiter
                                                            .send(Main2Engine::StartGame),
                                                    )
                                                    .unwrap();
                                            }
                                            gui::ElementNames::Resume => {
                                                this.gui_manager
                                                    .change_page(&mut this.gui, Pages::Ingame);
                                                this.rt
                                                    .block_on(
                                                        this.game_transmiter
                                                            .send(Main2Engine::ResumeGame),
                                                    )
                                                    .unwrap();
                                            }
                                            gui::ElementNames::ExitGame => {
                                                this.gui_manager
                                                    .change_page(&mut this.gui, Pages::Menu);
                                                this.rt
                                                    .block_on(
                                                        this.game_transmiter
                                                            .send(Main2Engine::PauseGame),
                                                    )
                                                    .unwrap();
                                            }
                                            gui::ElementNames::Settings => todo!(),
                                        }
                                    }
                                    Actions::Cursor(cursor) => {
                                        this.window.set_cursor(cursor);
                                    }
                                    Actions::ChangePage(page) => {
                                        this.gui_manager.change_page(&mut this.gui, page);
                                    }
                                    Actions::None => (),
                                }
                            }
                        }
                        Some(Msgs::Ingame) => match e.kind {
                            rugui2::events::ElemEvents::KeyPress {
                                press: true,
                                key: Key::Escape,
                            } => {
                                this.rt
                                    .block_on(this.game_transmiter.send(Main2Engine::PauseGame))
                                    .unwrap();
                                this.gui_manager
                                    .change_page(&mut this.gui, Pages::IngamePause);
                            }
                            _ => (),
                        },
                        Some(Msgs::Settings) => match e.kind {
                            rugui2::events::ElemEvents::KeyPress {
                                press: true,
                                key: Key::KeyH,
                            } => {
                                this.gui_manager.change_page(&mut this.gui, Pages::Menu);
                            }
                            _ => (),
                        },
                        Some(Msgs::Send(msg)) => {
                            this.rt.block_on(this.game_transmiter.send(*msg)).unwrap();
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

    fn user_event(&mut self, _: &winit::event_loop::ActiveEventLoop, event: Engine2Main) {
        let this = match self {
            WinitAgentIAmLosingIt::Running(app) => app,
            _ => return,
        };
        match event {
            Engine2Main::GameEvent(game_events) => match game_events {
                GameEvents::Score(_) => {}
                GameEvents::Win(side) => {
                    this.gui_manager
                        .change_page(&mut this.gui, Pages::IngameEnd(side));
                    this.rt
                        .block_on(this.game_transmiter.send(Main2Engine::PauseGame))
                        .unwrap();
                }
            },
        }
    }
}
