use timer::Timer;
use winit_controls::Controls;

use crate::Control;

pub const WIDTH: f32 = 1000.0;
pub const HEIGHT: f32 = 800.0;
pub const PADDLE_WIDTH: f32 = 20.0;
pub const PADDLE_HEIGHT: f32 = 200.0;
pub const BALL_RADIUS: f32 = 12.0;
pub const PADDLE_OFFSET: f32 = 160.0;
pub const SPEED: f32 = 20.0;
pub const AI_SPEED: f32 = 9.0;
pub const SPEED_MUL: f32 = 1.02;
pub const BASE_BALL_SPEED: f32 = 6.0;
pub const GRAVITY: f32 = 0.08; // why not :)
pub const WIN_SCORE: u8 = 5;

pub struct Game {
    pub left: Paddle,
    pub right: Paddle,
    pub ball: Ball,
    pub events: Vec<GameEvents>,
}

#[derive(Debug, Copy, Clone, PartialEq)]

pub enum Sides {
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GameEvents {
    Score(Sides),
    Win(Sides),   
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ControllerTypes {
    Player,
    AI,
}

pub struct Paddle {
    pub x: f32,
    pub y: f32,
    pub controller: ControllerTypes,
    pub score: u8,
}

pub struct Ball {
    pub x: f32,
    pub y: f32,
    pub xs: f32,
    pub ys: f32,
}

impl Ball {
    fn new() -> Self {
        Ball {
            x: WIDTH * 0.5,
            y: HEIGHT * 0.5,
            xs: BASE_BALL_SPEED,
            ys: BASE_BALL_SPEED,
        }
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            ball: Ball::new(),
            left: Paddle {
                x: PADDLE_OFFSET,
                y: 100.0,
                controller: ControllerTypes::AI,
                score: 0,
            },
            right: Paddle {
                x: WIDTH - PADDLE_OFFSET - PADDLE_WIDTH,
                y: 100.0,
                controller: ControllerTypes::AI,
                score: 0,
            },
            events: Vec::new()
        }
    }

    pub fn tick(&mut self, _timer: &Timer, controls: &Controls<Control>) {
        if self.left.controller == ControllerTypes::Player {
            if controls.key(&Control::LeftDown) > 0 {
                self.left.y = (self.left.y + SPEED).min(HEIGHT - PADDLE_HEIGHT);
            }
            if controls.key(&Control::LeftUp) > 0 {
                self.left.y = (self.left.y - SPEED).max(0.0);
            }
        } else {
            let paddle_dist = self.right.x - self.left.x;
            let ball_dist = self.ball.x - self.left.x;
            let predict = (ball_dist / paddle_dist) * 50.0 + 9.0;
            self.left.y += (self.ball.y + self.ball.ys * predict - self.left.y - PADDLE_HEIGHT * 0.5)
                .max(-AI_SPEED)
                .min(AI_SPEED);
            self.left.y = self.left.y.max(0.0).min(HEIGHT - PADDLE_HEIGHT);
        }

        if self.right.controller == ControllerTypes::Player {
            if controls.key(&Control::RightDown) > 0 {
                self.right.y = (self.right.y + SPEED).min(HEIGHT - PADDLE_HEIGHT);
            }
            if controls.key(&Control::RightUp) > 0 {
                self.right.y = (self.right.y - SPEED).max(0.0);
            }
        } else {
            let paddle_dist = self.left.x - self.right.x;
            let ball_dist = self.ball.x - self.right.x;
            let predict = (ball_dist / paddle_dist) * 50.0 + 9.0;
            self.right.y += (self.ball.y + self.ball.ys * predict - self.right.y - PADDLE_HEIGHT * 0.5)
                .max(-AI_SPEED)
                .min(AI_SPEED);
            self.right.y = self.right.y.max(0.0).min(HEIGHT - PADDLE_HEIGHT);
        }

        self.ball.ys += GRAVITY;
        self.ball.x += self.ball.xs;
        self.ball.y += self.ball.ys;

        if self.ball.y - BALL_RADIUS < 0.0 {
            self.ball.xs *= SPEED_MUL;
            self.ball.ys *= -0.9;
            self.ball.y = BALL_RADIUS;
        }
        if self.ball.y + BALL_RADIUS > HEIGHT {
            self.ball.xs *= SPEED_MUL;
            self.ball.ys *= -0.9;
            self.ball.y = HEIGHT - BALL_RADIUS;
        }
        if self.ball.x - BALL_RADIUS > WIDTH {
            self.left.y += 40.0;
            self.left.score += 1;
            self.events.push(GameEvents::Score(Sides::Left));
            if self.left.score == WIN_SCORE {
                self.events.push(GameEvents::Win(Sides::Left));
            }
            self.ball = Ball::new();
            self.ball.xs += (self.left.score as f32 + self.right.score as f32) + BASE_BALL_SPEED;
            self.ball.xs = -self.ball.xs;
            self.ball.x = self.right.x - BALL_RADIUS * 2.0;
        }
        if self.ball.x + BALL_RADIUS < 0.0 {
            self.left.x -= 40.0;
            self.right.score += 1;
            self.events.push(GameEvents::Score(Sides::Right));
                if self.right.score == WIN_SCORE {
                self.events.push(GameEvents::Win(Sides::Right));
            }
            self.ball = Ball::new();
            self.ball.xs += (self.left.score as f32 + self.right.score as f32) * 0.5 + BASE_BALL_SPEED;
            self.ball.x = self.left.x + PADDLE_WIDTH + BALL_RADIUS * 2.0;
        }

        if collision(&self.ball, &self.left) {
            self.ball.x = self.left.x + BALL_RADIUS + PADDLE_WIDTH;
            self.ball.xs = (self.ball.xs * -SPEED_MUL).min(BALL_RADIUS*2.0);
            self.ball.ys = (self.ball.y - (self.left.y + PADDLE_HEIGHT * 0.5)) / (PADDLE_HEIGHT * 0.03);
        }
        if collision(&self.ball, &self.right) {
            self.ball.x = self.right.x - BALL_RADIUS;
            self.ball.xs = (self.ball.xs * -SPEED_MUL).min(BALL_RADIUS*2.0);
            self.ball.ys = (self.ball.y - (self.right.y + PADDLE_HEIGHT * 0.5)) / (PADDLE_HEIGHT * 0.03);
        }
    }
}

fn collision(ball: &Ball, paddle: &Paddle) -> bool {
    let (ball_x, ball_y) = (ball.x - BALL_RADIUS * 0.8, ball.y - BALL_RADIUS);
    ball_x < paddle.x + PADDLE_WIDTH
        && ball_x + BALL_RADIUS * 1.6 > paddle.x
        && ball_y < paddle.y + PADDLE_HEIGHT
        && ball_y + BALL_RADIUS * 2.0 > paddle.y
}