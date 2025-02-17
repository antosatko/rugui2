use std::{collections::HashMap, path::PathBuf};

use cgmath::{Point2, Vector2};
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};
use std::hash::Hash;

#[derive(Debug)]
pub struct Controls<Code> where Code: Copy {
    pub locked: bool,
    pub binds: HashMap<Code, PhysicalKey>,
    keys: HashMap<PhysicalKey, i32>,
    pub mouse: Mouse,
    pub text_input: Vec<TextEdit>,
    pub file_hover: FileHover,
}

#[derive(Debug)]
pub enum FileHover {
    None,
    Hover(PathBuf),
    Drop(PathBuf),
}

#[derive(Debug)]
pub struct Mouse {
    pub now: Point2<f64>,
    pub prev: Point2<f64>,
    pub delta: Vector2<f64>,
    pub wheel: f32,
    pub left_click: i32,
    pub right_click: i32,
    pub middle_click: i32,
}

#[derive(Debug, Clone, Hash)]
pub enum TextEdit {
    Text(String),
    Enter,
    Delete,
    Backspace,
    MoveCursor(Directions),
}

#[derive(Debug, Clone, Hash)]
pub enum Directions {
    Up,
    Down,
    Left,
    Right,
}

impl<Code> Controls <Code> where Code: Copy + Eq + Hash {
    pub fn new() -> Self {
        Self {
            locked: false,
            binds: HashMap::new(),
            keys: HashMap::new(),
            mouse: Mouse {
                now: Point2::new(0.0, 0.0),
                prev: Point2::new(0.0, 0.0),
                delta: Vector2::new(0.0, 0.0),
                wheel: 0.0,
                left_click: 0,
                right_click: 0,
                middle_click: 0,
            },
            text_input: Vec::new(),
            file_hover: FileHover::None,
        }
    }

    pub fn key_input(
        &mut self,
        event: &winit::keyboard::PhysicalKey,
        state: &ElementState,
        logical: &winit::keyboard::Key,
    ) {
        if self.locked {
            return;
        }
        if state.is_pressed() {
            match event {
                PhysicalKey::Code(KeyCode::Delete) => self.text_input.push(TextEdit::Delete),
                PhysicalKey::Code(KeyCode::Backspace) => self.text_input.push(TextEdit::Backspace),
                PhysicalKey::Code(KeyCode::Enter) => self.text_input.push(TextEdit::Enter),
                PhysicalKey::Code(KeyCode::ArrowLeft) => {
                    self.text_input.push(TextEdit::MoveCursor(Directions::Left))
                }
                PhysicalKey::Code(KeyCode::ArrowRight) => self
                    .text_input
                    .push(TextEdit::MoveCursor(Directions::Right)),
                PhysicalKey::Code(KeyCode::ArrowUp) => {
                    self.text_input.push(TextEdit::MoveCursor(Directions::Up))
                }
                PhysicalKey::Code(KeyCode::ArrowDown) => {
                    self.text_input.push(TextEdit::MoveCursor(Directions::Down))
                }
                _ => {
                    if let Some(txt) = logical.to_text() {
                        if txt.chars().all(|c| !c.is_control()) {
                            self.text_input.push(TextEdit::Text(txt.to_string()));
                        }
                    }
                }
            }
        }
        for (key, value) in self.keys.iter_mut() {
            if event == key {
                *value = if state == &ElementState::Pressed {
                    if *value <= 0 {
                        1
                    } else {
                        *value
                    }
                } else {
                    0
                };
            }
        }
    }

    pub fn mouse_input(&mut self, x: f64, y: f64) {
        if self.locked {
            return;
        }
        self.mouse.prev = self.mouse.now;
        self.mouse.now = Point2::new(x, y);
        self.mouse.delta = self.mouse.now - self.mouse.prev;
    }

    pub fn mouse_wheel(&mut self, delta: &winit::event::MouseScrollDelta) {
        if self.locked {
            return;
        }
        match delta {
            winit::event::MouseScrollDelta::LineDelta(_, y) => self.mouse.wheel = *y,
            winit::event::MouseScrollDelta::PixelDelta(_) => (),
        }
    }

    pub fn mouse_click(&mut self, state: &ElementState, mouse_button: winit::event::MouseButton) {
        if self.locked {
            return;
        }
        let new_state = if state == &ElementState::Pressed {
            1
        } else {
            0
        };
        match mouse_button {
            winit::event::MouseButton::Left => self.mouse.left_click = new_state,
            winit::event::MouseButton::Right => self.mouse.right_click = new_state,
            winit::event::MouseButton::Middle => self.mouse.middle_click = new_state,
            _ => (),
        }
    }

    pub fn tick(&mut self) {
        for (_, value) in self.keys.iter_mut() {
            if *value > 0 {
                *value += 1;
            }else {
                *value -= 1;
            }
        }
        self.mouse.delta = Vector2::new(0.0, 0.0);
        self.mouse.prev = self.mouse.now;
        self.mouse.wheel = 0.0;
        if self.mouse.left_click > 0 {
            self.mouse.left_click += 1;
        } else {
            self.mouse.left_click -= 1;
        }
        if self.mouse.right_click > 0 {
            self.mouse.right_click += 1;
        } else {
            self.mouse.right_click -= 1;
        }
        if self.mouse.middle_click > 0 {
            self.mouse.middle_click += 1;
        } else {
            self.mouse.middle_click -= 1;
        }
        self.text_input.clear();
        match &self.file_hover {
            FileHover::Drop(_) => self.file_hover = FileHover::None,
            _ => (),
        }
    }

    pub fn bind(&mut self, key: PhysicalKey, code: Code) {
        self.binds.insert(code, key);
        if !self.keys.contains_key(&key) {
            self.keys.insert(key, 0);
        }
    }

    pub fn key(&self, code: &Code) -> i32 {
        let bind = self.binds.get(code).unwrap();
        *self.keys.get(bind).unwrap()
    }

    pub fn key_on(&mut self, code: &Code) {
        let bind = self.binds.get(code).unwrap();
        self.keys.insert(*bind, 1);
    }

    pub fn key_off(&mut self, code: &Code) {
        let bind = self.binds.get(code).unwrap();
        self.keys.insert(*bind, 0);
    }

    pub fn update(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                if !is_synthetic {
                    self.key_input(&event.physical_key, &event.state, &event.logical_key);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.mouse_click(&state, *button);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_input(position.x, position.y);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.mouse_wheel(&delta);
            }
            WindowEvent::DroppedFile(path) => {
                self.file_hover = FileHover::Drop(path.clone());
            }
            WindowEvent::HoveredFile(path) => {
                self.file_hover = FileHover::Hover(path.clone());
            }
            WindowEvent::HoveredFileCancelled => {
                self.file_hover = FileHover::None;
            }
            _ => (),
        }
    }
}

impl <Code: Copy + Eq + Hash> FromIterator<(PhysicalKey, Code)> for Controls<Code> {
    fn from_iter<T: IntoIterator<Item = (PhysicalKey, Code)>>(iter: T) -> Self {
        let mut this = Self::new();
        for (key, code) in iter {
            this.bind(key, code);
        }
        this
    }
}

pub struct TextField {
    txt: String,
    cursor: usize,
    pub text_navigation: bool,
}

impl TextField {
    pub fn new() -> Self {
        Self {
            txt: String::new(),
            cursor: 0,
            text_navigation: true,
        }
    }

    pub fn set_text_navigation(mut self, allow: bool) -> Self {
        self.text_navigation = allow;
        self.cursor = self.txt.chars().count();
        self
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn txt(&self) -> &str {
        &self.txt
    }

    pub fn text_navigation(&self) -> bool {
        self.text_navigation
    }

    pub fn command(&mut self, command: &TextEdit) {
        match command {
            TextEdit::Text(txt) => self.write(txt),
            TextEdit::Enter => self.write("\n"),
            TextEdit::Delete => {
                if self.cursor < self.txt.chars().count() && self.cursor > 0 {
                    self.txt.remove(self.cursor);
                }
            }
            TextEdit::Backspace => {
                if self.cursor > 0 {
                    self.txt.remove(self.cursor - 1);
                    self.cursor -= 1;
                }
            }
            TextEdit::MoveCursor(directions) => {
                if self.text_navigation {
                    match directions {
                        Directions::Up => (),
                        Directions::Down => (),
                        Directions::Left => {
                            if self.cursor > 0 {
                                self.cursor -= 1
                            }
                        }
                        Directions::Right => {
                            if self.cursor < self.txt.chars().count() {
                                self.cursor += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn write(&mut self, txt: &str) {
        self.txt.insert_str(self.cursor, txt);
        self.cursor += txt.chars().count();
    }
}

impl From<&str> for TextField {
    fn from(value: &str) -> Self {
        Self {
            txt: value.to_string(),
            cursor: value.chars().count(),
            text_navigation: true,
        }
    }
}
