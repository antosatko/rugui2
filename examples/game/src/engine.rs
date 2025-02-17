use std::sync::Arc;

use canvas::Rgba;
use tokio::{
    sync::{mpsc::Receiver, Mutex},
    time::timeout,
};

use winit::{event_loop::EventLoopProxy, window::Window};
use winit_controls::Controls;

use crate::{Control, Drawing, Game, GameEvents, BALL_RADIUS, PADDLE_HEIGHT, PADDLE_WIDTH, RUNNING};

pub struct Engine {
    pub window: Arc<Window>,
    pub timer: timer::Timer,
    pub drawing: Arc<Mutex<Drawing>>,
    pub controls: Arc<Mutex<Controls<Control>>>,
    pub game: Game,
    reciever: Receiver<Main2Engine>,
    proxy: EventLoopProxy<Engine2Main>,
    state: State,
}

pub enum State {
    Ingame,
    Idle,
}

impl Engine {
    pub fn new(
        window: Arc<Window>,
        drawing: Arc<Mutex<Drawing>>,
        controls: Arc<Mutex<Controls<Control>>>,
        reciever: Receiver<Main2Engine>,
        proxy: EventLoopProxy<Engine2Main>,
    ) -> Self {
        let clock = timer::Timer::new(60.0);
        let game = Game::new();

        Self {
            window,
            timer: clock,
            drawing,
            controls,
            game,
            reciever,
            proxy,
            state: State::Idle,
        }
    }

    pub async fn run(mut self) {
        while RUNNING.load(std::sync::atomic::Ordering::Relaxed) {
            while let Ok(msg) = self.reciever.try_recv() {
                match msg {
                    Main2Engine::ResumeGame => self.state = State::Ingame,
                    Main2Engine::PauseGame => self.state = State::Idle,
                    Main2Engine::StartGame => {
                        self.state = State::Ingame;
                        self.game = Game::new();
                    }
                }
            }

            match self.state {
                State::Idle => (),
                State::Ingame => {
                    {
                        let mut controls = self.controls.lock().await;

                        self.game.tick(&self.timer, &controls);

                        
                        controls.tick();
                    }
                    for e in self.game.events.iter().rev() {
                        self.proxy.send_event(Engine2Main::GameEvent(*e)).unwrap();
                    }
                    self.game.events.clear();
                    'drawing: {
                        let time_frame = match self.timer.remaining_time() {
                            Some(t) => t - std::time::Duration::from_millis(5),
                            None => break 'drawing,
                        };
                        let mut drawing = match timeout(time_frame, self.drawing.lock()).await {
                            Ok(d) => d,
                            Err(_) => break 'drawing,
                        };
                        drawing.canvas.clear(Rgba::BLACK);
                        let x_mult = drawing.canvas.pixels.dimensions().0 as f32 / crate::WIDTH;
                        let y_mult = drawing.canvas.pixels.dimensions().1 as f32 / crate::HEIGHT;

                        drawing.canvas.draw_shape(
                            canvas::Shapes::Circle {
                                x: (self.game.ball.x * x_mult) as i32,
                                y: (self.game.ball.y * y_mult) as i32,
                                radius: (BALL_RADIUS * (x_mult + y_mult) * 0.5) as i32,
                            },
                            Rgba::WHITE,
                        );

                        drawing.canvas.draw_shape(
                            canvas::Shapes::Rectangle {
                                x: (self.game.left.x * x_mult) as i32,
                                y: (self.game.left.y * y_mult) as i32,
                                width: (PADDLE_WIDTH * x_mult) as i32,
                                height: (PADDLE_HEIGHT * y_mult) as i32,
                            },
                            Rgba::WHITE,
                        );
                        drawing.canvas.draw_shape(
                            canvas::Shapes::Rectangle {
                                x: (self.game.right.x * x_mult) as i32,
                                y: (self.game.right.y * y_mult) as i32,
                                width: (PADDLE_WIDTH * x_mult) as i32,
                                height: (PADDLE_HEIGHT * y_mult) as i32,
                            },
                            Rgba::WHITE,
                        );
                    }
                }
            }

            self.window.request_redraw();
            self.timer.sleep_tick(); //.unwrap_or_else(|| panic!());
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Main2Engine {
    PauseGame,
    ResumeGame,
    StartGame,
}

#[derive(Debug, Copy, Clone)]
pub enum Engine2Main {
    GameEvent(GameEvents)
}
