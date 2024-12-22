use crate::{ElementKey, Vector};

#[derive(Debug, Copy, Clone)]
pub enum EnvEvents {
    MouseButton {
        button: MouseButtons,
        press: bool,
    },
    CursorMove {
        pos: Vector,
    },
    KeyInput {
        press: bool,
    },
    Scroll {
        delta: Vector,
    },
    Select {
        opt: SelectOpts,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum SelectOpts {
    Next,
    Prev,
    Confirm,
    Lock,
    Unlock,
}

#[derive(Debug, Copy, Clone)]
pub enum MouseButtons {
    Left,
    Right,
    Middle,
}

pub struct ElemEvent<Msg: Clone> {
    pub kind: ElemEvents,
    pub element_key: ElementKey,
    pub msg: Option<Msg>,
}

#[derive(Debug, Copy, Clone)]
pub enum ElemEvents {
    CursorEnter {
        pos: Vector,
    },
    CursorLeave {
        prev_pos: Vector,
    },
    CursorMove {
        pos: Vector,
        prev_pos: Vector,
    },
    Click {
        button: MouseButtons,
        press: bool,
        pos: Vector,
    },
    Selection {
        state: SelectionStates,
    },
    Scroll {
        delta: Vector,
        pos: Vector,
    }
}

pub enum ListenerTypes {
    Listen,
    Peek,
    Force,
}

pub enum ElemEventTypes {
    MouseMove,
    Click,
    Hover,
    Scroll,
}

pub enum EnvEventStates {
    Free,
    Consumed,
}

#[derive(Debug, Copy, Clone)]
pub enum SelectionStates {
    Confirm,
    Enter,
    Leave,
}