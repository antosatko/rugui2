use std::{collections::HashMap, fmt::Debug, num::NonZero};

use colors::*;
use element::{Container, *};
use events::*;
use math::*;
use styles::*;

pub mod colors;
pub mod element;
pub mod events;
pub mod math;
pub mod styles;

pub struct Gui<Msg: Clone> {
    elements: HashMap<ElementKey, Element<Msg>>,
    next_elem_id: u64,
    viewport: ContainerWrapper,
    size: (u32, u32),
    entry: Option<ElementKey>,
    cursor: Cursor,
    events: Vec<events::ElemEvent<Msg>>,
    selection: Selection,
}

impl<Msg: Clone> Gui<Msg> {
    pub fn new(size: (NonZero<u32>, NonZero<u32>)) -> Self {
        let size = (size.0.get(), size.1.get());
        Self {
            elements: HashMap::new(),
            viewport: ContainerWrapper::new_dirty(&Container {
                pos: Vector::ZERO,
                size: Vector(size.0 as f32, size.1 as f32),
                rotation: 0.0,
            }),
            size,
            entry: None,
            next_elem_id: 0,
            cursor: Cursor::default(),
            events: Vec::new(),
            selection: Selection::default(),
        }
    }

    pub fn resize(&mut self, size: (NonZero<u32>, NonZero<u32>)) {
        let size = (size.0.get(), size.1.get());
        self.size = size;
        let s = Vector(size.0 as f32, size.1 as f32);
        self.viewport.set_size(s);
        self.viewport.set_pos(s * 0.5);
    }

    pub fn update(&mut self) {
        let entry = match self.entry {
            Some(e) => e,
            None => return,
        };

        let vp_copy = self.viewport;
        let container = &vp_copy;
        let vp = vp_copy.get();

        self.update_element(entry, container, vp);

        self.viewport.clean();
    }

    fn update_element(&mut self, key: ElementKey, container: &ContainerWrapper, vp: &Container) {
        let element = match self.elements.get_mut(&key) {
            Some(e) => e,
            None => return,
        };

        let mut element_container = ContainerWrapper::new(&element.instance.container);

        // --- TRANSFORMS ---
        //
        // SIZE
        //
        if element.styles.width.is_dirty()
            || container.dirty_size()
            || element.styles.max_width.is_dirty()
            || element.styles.min_width.is_dirty()
        {
            let width = element.styles.width.fix_dirty_force();
            let max = element.styles.max_width.fix_dirty_force();
            let min = element.styles.min_width.fix_dirty_force();

            let mut width = width.calc(container.get(), vp, element_container.get());
            if let Some(max) = max {
                width = width.min(max.calc(container.get(), vp, element_container.get()));
            }
            if let Some(min) = min {
                width = width.max(min.calc(container.get(), vp, element_container.get()));
            }

            if element_container.get().size.0 != width {
                element_container.size_mut().0 = width;
            }
        }

        if element.styles.height.is_dirty()
            || container.dirty_size()
            || element.styles.max_height.is_dirty()
            || element.styles.min_height.is_dirty()
        {
            let style = element.styles.height.fix_dirty_force();
            let max = element.styles.max_height.fix_dirty_force();
            let min = element.styles.min_height.fix_dirty_force();

            let mut height = style.calc(container.get(), vp, element_container.get());
            if let Some(max) = max {
                height = height.min(max.calc(container.get(), vp, element_container.get()));
            }
            if let Some(min) = min {
                height = height.max(min.calc(container.get(), vp, element_container.get()));
            }

            if element_container.get().size.1 != height {
                element_container.size_mut().1 = height;
            }
        }

        //
        // POSITION
        // - dependent on size
        //
        if container.dirty_pos()
            || container.dirty_rotation()
            || container.dirty_size()
            || element.styles.align.is_dirty()
            || element.styles.center.is_dirty()
        {
            let cont = container.get();
            element_container.set_pos(cont.pos);

            let center = element
                .styles
                .center
                .get()
                .calc(cont, vp, element_container.get());
            let align = center
                - element
                    .styles
                    .align
                    .get()
                    .calc(cont, vp, element_container.get());

            element_container.set_pos(center + align);
        }

        //
        // ROTATION
        // - dependent on position
        //
        if element_container.dirty_pos() || container.dirty_rotation() {
            let cont = container.get();
            let elem = element_container.get();
            if cont.rotation != 0.0 && cont.pos != elem.pos {
                let pos = elem.pos.rotate_around(&cont.pos, cont.rotation);
                element_container.set_pos(pos);
            };
        }
        if element.styles.rotation.is_dirty() || container.dirty_rotation() {
            let rot =
                element
                    .styles
                    .rotation
                    .get()
                    .calc(container.get(), vp, element_container.get());
            element_container.set_rotation(rot);
        }
        // --- TRANSFORMS ---

        // --- TRANSFORM-DEPENDENT ---
        let transform_update = element_container.dirty_size()
            || element_container.dirty_pos()
            || element_container.dirty_rotation();
        if transform_update || element.styles.round.is_dirty() {
            if let Some(rnd) = element.styles.round.get() {
                let size = rnd.size.calc(container.get(), vp, element_container.get());
                let smooth = rnd
                    .smooth
                    .calc(container.get(), vp, element_container.get());
                element.instance.round = [size, smooth];
            }
        }
        if transform_update || element.styles.grad_linear.is_dirty() {
            if let Some(grad) = element.styles.grad_linear.fix_dirty_force() {
                let p1 = grad.p1.0.calc(container.get(), vp, element_container.get());
                let p2 = grad.p2.0.calc(container.get(), vp, element_container.get());
                element.instance.lin_grad_p1 = p1;
                element.instance.lin_grad_p2 = p2;
                element.instance.lin_grad_color1 = grad.p1.1.into();
                element.instance.lin_grad_color2 = grad.p2.1.into();
                element.instance.set_flag(Flags::LinearGradient);
            } else {
                element.instance.remove_flag(Flags::LinearGradient);
            }
        }
        if transform_update || element.styles.grad_radial.is_dirty() {
            if let Some(grad) = element.styles.grad_radial.fix_dirty_force() {
                let p1 = grad
                    .p1
                    .0
                    .calc_rot(container.get(), vp, element_container.get());
                let p2 = grad
                    .p2
                    .0
                    .calc_rot(container.get(), vp, element_container.get());
                element.instance.rad_grad_p1 = p1;
                element.instance.rad_grad_p2 = p2;
                element.instance.rad_grad_color1 = grad.p1.1.into();
                element.instance.rad_grad_color2 = grad.p2.1.into();
                element.instance.set_flag(Flags::RadialGradient);
            } else {
                element.instance.remove_flag(Flags::RadialGradient);
            }
        }
        // --- TRANSFORM-DEPENDENT ---

        // --- TRANSFORM-INDEPENDENT ---
        if element.dirty_styles {
            match element.styles.color.fix_dirty() {
                None => (),
                Some(c) => element.instance.color = (*c).into(),
            }
            match element.styles.alpha.fix_dirty() {
                None => (),
                Some(a) => element.instance.alpha = *a,
            }
            element.dirty_styles = false;
        }
        // --- TRANSFORM-INDEPENDENT ---

        let last = element.instance.container.clone();
        element
            .instance
            .container
            .clone_from(element_container.get());

        // --- EVENTS ---
        if transform_update {
            let over_last = self.cursor.last.container_colision(&last);
            let over_current = self
                .cursor
                .current
                .container_colision(element_container.get());

            /*match (over_last, over_current) {
                (Some(last), None) => {
                    self.events.push(ElemEvent {
                        kind: ElemEvents::CursorLeave { prev_pos: last },
                        element_key: key,
                        msg: None,
                    });
                }
                (None, Some(current)) => {
                    self.events.push(ElemEvent {
                        kind: ElemEvents::CursorEnter { pos: current },
                        element_key: key,
                        msg: None,
                    });
                }
                (Some(last), Some(current)) => {
                    self.events.push(ElemEvent {
                        kind: ElemEvents::CursorMove {
                            pos: current,
                            prev_pos: last,
                        },
                        element_key: key,
                        msg: None,
                    });
                }
                _ => (),
            }*/
        }
        // --- EVENTS ---

        if let Some(children) = element.children.take() {
            for child in &children {
                self.update_element(*child, &element_container, vp);
            }
            if let Some(e) = self.elements.get_mut(&key) {
                e.children = Some(children);
            };
        }
    }

    pub fn env_event(&mut self, event: EnvEvents) -> EnvEventStates {
        match event {
            EnvEvents::MouseButton { button, press } => (),
            EnvEvents::CursorMove { pos } => {
                self.cursor.last = self.cursor.current;
                self.cursor.current = pos;
            }
            EnvEvents::KeyInput { press } => (),
            EnvEvents::Scroll { delta } => (),
            EnvEvents::Select { opt } => {
                match opt {
                    SelectOpts::Next => {
                        if self.selection.locked {
                            return EnvEventStates::Free;
                        }
                        if let Some(key) = self.selection.current {
                            self.events.push(ElemEvent {
                                kind: ElemEvents::Selection {
                                    state: SelectionStates::Leave,
                                },
                                element_key: key,
                                msg: None,
                            });
                        }
                        if let Some(key) = self.selection.next() {
                            self.events.push(ElemEvent {
                                kind: ElemEvents::Selection {
                                    state: SelectionStates::Enter,
                                },
                                element_key: key,
                                msg: None,
                            });
                        }
                    }
                    SelectOpts::Prev => {
                        if self.selection.locked {
                            return EnvEventStates::Free;
                        }
                        if let Some(key) = self.selection.current {
                            self.events.push(ElemEvent {
                                kind: ElemEvents::Selection {
                                    state: SelectionStates::Leave,
                                },
                                element_key: key,
                                msg: None,
                            });
                        }
                        if let Some(key) = self.selection.prev() {
                            self.events.push(ElemEvent {
                                kind: ElemEvents::Selection {
                                    state: SelectionStates::Enter,
                                },
                                element_key: key,
                                msg: None,
                            });
                        }
                    }
                    SelectOpts::Confirm => {
                        if let Some(key) = self.selection.current {
                            self.events.push(ElemEvent {
                                kind: ElemEvents::Selection {
                                    state: SelectionStates::Confirm,
                                },
                                element_key: key,
                                msg: None,
                            });
                        }
                    }
                    SelectOpts::Lock => self.selection.locked = true,
                    SelectOpts::Unlock => self.selection.locked = false,
                }
                return EnvEventStates::Consumed;
            }
        }

        let mut state = EnvEventStates::Free;
        self.entry
            .map(|key| self.elem_env_event(key, event, &mut state));
        state
    }

    pub fn elem_env_event(
        &mut self,
        key: ElementKey,
        event: EnvEvents,
        state: &mut EnvEventStates,
    ) {
        let elem = match self.get_element(key) {
            Some(e) => e,
            None => return,
        };

        if let Some(children) = elem.children.clone() {
            for key in children {
                self.elem_env_event(key, event, state);
            }
        }

        let elem = match self.elements.get(&key) {
            Some(e) => e,
            None => return,
        };

        for listener in &elem.events {
            match (&listener.kind, &state) {
                (ListenerTypes::Force, _) => (),
                (ListenerTypes::Listen, EnvEventStates::Free) => (),
                (ListenerTypes::Peek, EnvEventStates::Free) => (),
                _ => continue,
            }
            match (&listener.event, event) {
                (ElemEventTypes::MouseMove, EnvEvents::CursorMove { .. }) => {
                    if let Some(pos) = self
                        .cursor
                        .current
                        .container_colision(&elem.instance.container)
                    {
                        self.events.push(ElemEvent {
                            kind: ElemEvents::CursorMove {
                                pos,
                                prev_pos: pos.relative_pos(
                                    &elem.instance.container.pos,
                                    elem.instance.container.rotation,
                                ),
                            },
                            element_key: key,
                            msg: listener.msg.clone(),
                        });
                        Self::fix_event_state(state, &listener.kind);
                    }
                }
                (ElemEventTypes::Hover, EnvEvents::CursorMove { .. }) => {
                    let current = self
                        .cursor
                        .current
                        .container_colision(&elem.instance.container);
                    let last = self
                        .cursor
                        .last
                        .container_colision(&elem.instance.container);
                    match (current, last) {
                        (Some(pos), None) => {
                            self.events.push(ElemEvent {
                                kind: ElemEvents::CursorEnter { pos },
                                element_key: key,
                                msg: listener.msg.clone(),
                            });
                            Self::fix_event_state(state, &listener.kind);
                        }
                        (None, Some(prev_pos)) => {
                            self.events.push(ElemEvent {
                                kind: ElemEvents::CursorLeave { prev_pos },
                                element_key: key,
                                msg: listener.msg.clone(),
                            });
                            Self::fix_event_state(state, &listener.kind);
                        }
                        _ => {}
                    }
                }
                (ElemEventTypes::Click, EnvEvents::MouseButton { button, press }) => {
                    if let Some(pos) = self
                        .cursor
                        .current
                        .container_colision(&elem.instance.container)
                    {
                        self.events.push(ElemEvent {
                            kind: ElemEvents::Click { button, press, pos },
                            element_key: key,
                            msg: listener.msg.clone(),
                        });
                        Self::fix_event_state(state, &listener.kind);
                    }
                }
                (ElemEventTypes::Scroll, EnvEvents::Scroll { delta }) => {
                    if let Some(pos) = self
                        .cursor
                        .current
                        .container_colision(&elem.instance.container)
                    {
                        self.events.push(ElemEvent {
                            kind: ElemEvents::Scroll { delta, pos },
                            element_key: key,
                            msg: listener.msg.clone(),
                        });
                        Self::fix_event_state(state, &listener.kind);
                    }
                }
                _ => continue,
            }
        }
    }

    fn fix_event_state(state: &mut EnvEventStates, listener: &ListenerTypes) {
        match listener {
            ListenerTypes::Listen => *state = EnvEventStates::Consumed,
            ListenerTypes::Force => *state = EnvEventStates::Consumed,
            _ => (),
        }
    }

    pub fn foreach_element_mut(
        &mut self,
        cb: &mut impl FnMut(&mut Element<Msg>, ElementKey),
        key: Option<ElementKey>,
    ) {
        let k = match key {
            Some(key) => key,
            None => match self.entry {
                Some(key) => key,
                None => return,
            },
        };
        let e = match self.get_element_mut(k) {
            Some(e) => e,
            None => return,
        };
        cb(e, k);
        let children = match e.children.take() {
            Some(children) => children,
            None => return,
        };
        for child in &children {
            self.foreach_element_mut(cb, Some(*child));
        }
        self.get_element_mut(k).expect("Unexpected :)").children = Some(children);
    }

    pub fn foreach_element(&self, cb: impl Fn(&Element<Msg>, ElementKey), key: Option<ElementKey>) {
        let k = match key {
            Some(key) => key,
            None => match self.entry {
                Some(key) => key,
                None => return,
            },
        };
        let e = match self.get_element(k) {
            Some(e) => e,
            None => return,
        };
        cb(e, k);
        let children = match e.children.clone() {
            Some(children) => children,
            None => return,
        };
        for child in &children {
            self.foreach_element(&cb, Some(*child));
        }
    }

    pub fn prepare_events(&mut self) {
        self.events.reverse();
    }

    pub fn poll_event(&mut self) -> Option<ElemEvent<Msg>> {
        self.events.pop()
    }

    pub fn add_element(&mut self, element: Element<Msg>) -> ElementKey {
        let key = ElementKey(self.next_elem_id);
        self.next_elem_id += 1;
        self.elements.insert(key, element);
        key
    }

    pub fn get_element(&self, k: ElementKey) -> Option<&Element<Msg>> {
        self.elements.get(&k)
    }

    pub fn get_element_mut(&mut self, k: ElementKey) -> Option<&mut Element<Msg>> {
        self.elements.get_mut(&k)
    }

    pub fn set_entry(&mut self, key: ElementKey) {
        self.entry = Some(key)
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Cursor {
    pub current: Vector,
    pub last: Vector,
}

#[derive(Debug, Clone, Default)]
pub struct Selection {
    pub selectables: Vec<ElementKey>,
    pub current: Option<ElementKey>,
    pub locked: bool,
}

impl Selection {
    pub fn next(&mut self) -> Option<ElementKey> {
        self.current = match self.current {
            Some(current) => self
                .selectables
                .iter()
                .skip_while(|k| **k != current)
                .next()
                .cloned(),
            None => self.selectables.first().cloned(),
        };
        self.current
    }
    pub fn prev(&mut self) -> Option<ElementKey> {
        self.current = match self.current {
            Some(current) => self
                .selectables
                .iter()
                .rev()
                .skip_while(|k| **k != current)
                .next()
                .cloned(),
            None => self.selectables.last().cloned(),
        };
        self.current
    }
    pub fn clear(&mut self) {
        self.current = None;
        self.selectables.clear();
    }
    pub fn select_element(&mut self, key: ElementKey) {
        if self.selectables.contains(&key) {
            self.current = Some(key)
        } else {
            self.current = None
        }
    }
}
