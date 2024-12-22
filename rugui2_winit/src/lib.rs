use rugui2::{events::EnvEventStates, math::Vector, Gui};
use winit::{dpi::PhysicalPosition, event::WindowEvent};

pub fn event<Msg: Clone>(winit: &WindowEvent, gui: &mut Gui<Msg>) -> EnvEventStates {
    match winit {
        WindowEvent::DroppedFile(path_buf) => todo!(),
        WindowEvent::HoveredFile(path_buf) => todo!(),
        WindowEvent::HoveredFileCancelled => todo!(),
        WindowEvent::KeyboardInput {
            device_id,
            event,
            is_synthetic,
        } => todo!(),
        WindowEvent::ModifiersChanged(modifiers) => panic!("mods: {modifiers:?}"),
        WindowEvent::CursorMoved {
            device_id,
            position,
        } => gui.env_event(rugui2::events::EnvEvents::CursorMove {
            pos: Vector(position.x as _, position.y as _),
        }),
        WindowEvent::MouseWheel {
            device_id,
            delta,
            phase,
        } => gui.env_event(rugui2::events::EnvEvents::Scroll {
            delta: match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => Vector(*x, *y),
                winit::event::MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
                    Vector(*x as _, *y as _)
                }
            },
        }),
        WindowEvent::MouseInput {
            device_id,
            state,
            button,
        } => gui.env_event(rugui2::events::EnvEvents::MouseButton {
            button: match button {
                winit::event::MouseButton::Left => rugui2::events::MouseButtons::Left,
                winit::event::MouseButton::Right => rugui2::events::MouseButtons::Right,
                winit::event::MouseButton::Middle => rugui2::events::MouseButtons::Middle,
                _ => return EnvEventStates::Free,
            },
            press: match state {
                winit::event::ElementState::Pressed => true,
                winit::event::ElementState::Released => false,
            },
        }),
        WindowEvent::PinchGesture {
            device_id,
            delta,
            phase,
        } => todo!(),
        WindowEvent::PanGesture {
            device_id,
            delta,
            phase,
        } => todo!(),
        WindowEvent::DoubleTapGesture { device_id } => todo!(),
        WindowEvent::RotationGesture {
            device_id,
            delta,
            phase,
        } => todo!(),
        WindowEvent::TouchpadPressure {
            device_id,
            pressure,
            stage,
        } => todo!(),
        WindowEvent::Touch(touch) => todo!(),
        _ => EnvEventStates::Free,
    }
}
