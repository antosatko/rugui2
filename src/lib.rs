use std::{fmt::Debug, num::NonZero, path::PathBuf};

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

pub struct Gui<Msg: Clone = (), Img: Clone + ImageData = ()> {
    elements: Vec<Element<Msg, Img>>,
    viewport: ContainerWrapper,
    size: (u32, u32),
    entry: Option<ElementKey>,
    cursor: Cursor,
    events: Vec<events::ElemEvent<Msg>>,
    pub selection: Selection,
    file_drop_hover: Option<PathBuf>,
}

impl<Msg: Clone, Img: Clone + ImageData> Gui<Msg, Img> {
    pub fn new(size: (NonZero<u32>, NonZero<u32>)) -> Self {
        let size = (size.0.get(), size.1.get());
        Self {
            elements: Vec::new(),
            viewport: ContainerWrapper::new_dirty(&Container {
                pos: Vector::ZERO,
                size: Vector(size.0 as f32, size.1 as f32),
                rotation: 0.0,
            }),
            size,
            entry: None,
            cursor: Cursor::default(),
            events: Vec::new(),
            selection: Selection::default(),
            file_drop_hover: None,
        }
    }

    pub fn resize(&mut self, size: (NonZero<u32>, NonZero<u32>)) {
        let size = (size.0.get(), size.1.get());
        self.size = size;
        let s = Vector(size.0 as f32, size.1 as f32);
        self.viewport.set_size(s);
        self.viewport.set_pos(s * 0.5);
    }

    pub fn update(&mut self, time: f32) {
        let entry = match self.entry {
            Some(e) => e,
            None => return,
        };

        let vp_copy = self.viewport;
        let container = &vp_copy;
        let vp = vp_copy.get();

        self.selection.selectables.clear();
        self.update_element(entry, container, vp, time);

        self.viewport.clean();
    }

    fn update_element(&mut self, key: ElementKey, container: &ContainerWrapper, vp: &Container, time: f32) {
        let element = &mut self.elements[key.0 as usize];

        if element.allow_select {
            self.selection.selectables.push(key);
        }

        let mut element_container = ContainerWrapper::new(&element.instance.container);
        let container_transforms = container.get();

        // --- CONTENT-CONTAINERS ---
        if let Some(image_opt) = element.styles.image.fix_dirty() {
            match image_opt {
                Some(image) => {
                    let size = image.data.get_size();
                    element.instance.image_size = [size.0 as f32, size.1 as f32];

                    element.instance.set_flag(Flags::Image);
                }
                None => {
                    element.instance.image_size = [0.0, 0.0];
                    element.instance.remove_flag(Flags::Image);
                }
            }
        }
        let image = &element.instance.image_size.into();
        // --- CONTENT-CONTAINERS ---

        macro_rules! make_containers {
            () => {
                &Containers {
                    container: container_transforms,
                    vp,
                    this: element_container.get(),
                    image,
                    time,
                }
            };
        }


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
            let containers = make_containers!();

            let mut width = width.calc(containers);
            if let Some(max) = max {
                width = width.min(max.calc(containers));
            }
            if let Some(min) = min {
                width = width.max(min.calc(containers));
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
            let containers = make_containers!();

            let mut height = style.calc(containers);
            if let Some(max) = max {
                height = height.min(max.calc(containers));
            }
            if let Some(min) = min {
                height = height.max(min.calc(containers));
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
            element_container.set_pos(container_transforms.pos);
            let containers = make_containers!();

            let center =
                element
                    .styles
                    .center
                    .get()
                    .calc(containers);
            let align = element.styles.align.get().calc_relative(containers);

            element_container.set_pos(center - align);
        }

        //
        // ROTATION
        // - dependent on position
        //
        if element_container.dirty_pos() || container.dirty_rotation() {
            let elem = element_container.get();
            if container_transforms.rotation != 0.0 && container_transforms.pos != elem.pos {
                let pos = elem
                    .pos
                    .rotate_around(&container_transforms.pos, container_transforms.rotation);
                element_container.set_pos(pos);
            };
        }
        if element.styles.rotation.is_dirty() || container.dirty_rotation() {
            let containers = make_containers!();
            let rot = element.styles.rotation.get().calc(containers);
            element_container.set_rotation(rot);
        }
        // --- TRANSFORMS ---

        let element_container_c = element_container.get();

        macro_rules! make_containers {
            () => {
                &Containers {
                    container: container_transforms,
                    vp,
                    this: element_container_c,
                    image,
                    time,
                }
            };
        }
        let containers = make_containers!();

        // --- TRANSFORM-DEPENDENT ---
        let transform_update = element_container.dirty_size()
            || element_container.dirty_pos()
            || element_container.dirty_rotation();
        if transform_update || element.styles.round.is_dirty() {
            if let Some(rnd) = element.styles.round.get() {
                let size = rnd.size.calc(containers);
                let smooth = rnd
                    .smooth
                    .calc(containers);
                element.instance.round = [size, smooth];
            }
        }
        if transform_update || element.styles.grad_linear.is_dirty() {
            if let Some(grad) = element.styles.grad_linear.fix_dirty_force() {
                let p1 = grad
                    .p1
                    .0
                    .calc(containers);
                let p2 = grad
                    .p2
                    .0
                    .calc(containers);
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
                    .calc_rot(containers);
                let p2 = grad
                    .p2
                    .0
                    .calc_rot(containers);
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
        if let Some(tint) = element.styles.image_tint.fix_dirty() {
            element.instance.image_tint = (*tint).into();
        }
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
                self.update_element(*child, &element_container, vp, time);
            }
            self.elements[key.0 as usize].children = Some(children);
        }
    }

    pub fn env_event(&mut self, event: EnvEvents) -> EnvEventStates {
        match &event {
            EnvEvents::Input { text } => {
                if let Some(key) = self.selection.current {
                    if let Some(e) = self.get_element(key) {
                        if e.allow_text_input {
                            self.events.push(ElemEvent {
                                kind: ElemEvents::TextInput { text: text.clone() },
                                element_key: key,
                                msg: None,
                            });
                        }
                    }
                }
            }
            EnvEvents::KeyPress { .. } => {}
            EnvEvents::MouseButton { .. } => (),
            EnvEvents::CursorMove { pos } => {
                self.cursor.last = self.cursor.current;
                self.cursor.current = *pos;
            }
            EnvEvents::Scroll { .. } => (),
            EnvEvents::FileDrop { path, opt } => match opt {
                FileDropOpts::Drop => self.file_drop_hover = None,
                FileDropOpts::Hover => self.file_drop_hover = path.clone(),
                FileDropOpts::Cancel => self.file_drop_hover = None,
            },
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
                    SelectOpts::SelectKey(selected_key) => {
                        let (prev_key, selected_key) = self.selection.select_element(*selected_key);
                        if let Some(element_key) = selected_key {
                            self.events.push(ElemEvent {
                                kind: ElemEvents::Selection {
                                    state: SelectionStates::Enter,
                                },
                                element_key,
                                msg: None,
                            });
                        }
                        if let Some(element_key) = prev_key {
                            self.events.push(ElemEvent {
                                kind: ElemEvents::Selection {
                                    state: SelectionStates::Leave,
                                },
                                element_key,
                                msg: None,
                            });
                        }
                    }
                    SelectOpts::NoFocus => {
                        if let Some(element_key) = self.selection.current {
                            self.events.push(ElemEvent {
                                kind: ElemEvents::Selection {
                                    state: SelectionStates::Leave,
                                },
                                element_key,
                                msg: None,
                            });
                        }
                        self.selection.current = None;
                    }
                }
                return EnvEventStates::Consumed;
            }
        }

        let mut state = EnvEventStates::Free;
        self.entry
            .map(|key| self.elem_env_event(key, &event, &mut state));
        state
    }

    pub fn elem_env_event(
        &mut self,
        key: ElementKey,
        event: &EnvEvents,
        state: &mut EnvEventStates,
    ) {
        let elem = &self.elements[key.0 as usize];

        if let Some(children) = elem.children.clone() {
            for key in children {
                self.elem_env_event(key, event, state);
            }
        }

        let elem = &self.elements[key.0 as usize];

        for listener in &elem.events {
            match (&listener.kind, &state) {
                (ListenerTypes::Force, _) => (),
                (ListenerTypes::Listen, EnvEventStates::Free) => (),
                (ListenerTypes::Peek, EnvEventStates::Free) => (),
                _ => continue,
            }
            match (&listener.event, &event) {
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
                        let (button, press) = (*button, *press);
                        self.events.push(ElemEvent {
                            kind: ElemEvents::Click { button, press, pos },
                            element_key: key,
                            msg: listener.msg.clone(),
                        });
                        Self::fix_event_state(state, &listener.kind);
                    }
                }
                (ElemEventTypes::FileDrop, EnvEvents::FileDrop { path, opt }) => {
                    match opt {
                        FileDropOpts::Cancel => continue,
                        FileDropOpts::Hover => continue,
                        _ => (),
                    }
                    if let Some(pos) = self
                        .cursor
                        .current
                        .container_colision(&elem.instance.container)
                    {
                        let path = if let Some(path) = path {
                            path.clone()
                        } else {
                            continue;
                        };
                        self.events.push(ElemEvent {
                            kind: ElemEvents::FileDrop { path, pos },
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
                        let delta = *delta;
                        self.events.push(ElemEvent {
                            kind: ElemEvents::Scroll { delta, pos },
                            element_key: key,
                            msg: listener.msg.clone(),
                        });
                        Self::fix_event_state(state, &listener.kind);
                    }
                }
                (
                    ElemEventTypes::KeyPress,
                    EnvEvents::KeyPress {
                        key: keyboard_key,
                        press,
                    },
                ) => {
                    let (keyboard_key, press) = (*keyboard_key, *press);
                    self.events.push(ElemEvent {
                        kind: ElemEvents::KeyPress {
                            press,
                            key: keyboard_key,
                        },
                        element_key: key,
                        msg: listener.msg.clone(),
                    });
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
        cb: &mut impl FnMut(&mut Element<Msg, Img>, ElementKey),
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

    pub fn foreach_element(
        &self,
        cb: impl Fn(&Element<Msg, Img>, ElementKey),
        key: Option<ElementKey>,
    ) {
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

    pub fn first_element(
        &self,
        root: Option<ElementKey>,
        predicate: &impl Fn(&Element<Msg, Img>) -> bool,
    ) -> Option<ElementKey> {
        let root = match root {
            Some(r) => r,
            None => match self.entry {
                Some(e) => e,
                None => return None,
            },
        };

        let elem = &self.elements[root.0 as usize];

        match &elem.children {
            Some(c) => {
                let children = c.clone();
                for c in children {
                    match self.first_element(Some(c), predicate) {
                        Some(k) => return Some(k),
                        None => (),
                    }
                }
            }
            None => (),
        };

        if predicate(elem) {
            return Some(root);
        }
        None
    }

    pub fn prepare_events(&mut self) {
        self.events.reverse();
    }

    pub fn poll_event(&mut self) -> Option<ElemEvent<Msg>> {
        self.events.pop()
    }

    pub fn add_element(&mut self, element: Element<Msg, Img>) -> ElementKey {
        let key = ElementKey(self.elements.len() as u64);
        self.elements.push(element);
        key
    }

    pub fn get_element(&self, k: ElementKey) -> Option<&Element<Msg, Img>> {
        if (k.0 as usize) < self.elements.len() {
            Some(&self.elements[k.0 as usize])
        } else {
            None
        }
    }

    pub fn get_element_mut(&mut self, k: ElementKey) -> Option<&mut Element<Msg, Img>> {
        if (k.0 as usize) < self.elements.len() {
            Some(&mut self.elements[k.0 as usize])
        } else {
            None
        }
    }

    /// # Panic
    /// 
    /// May panic if the element does not exist. This is generally safe, since if an element
    /// does not exist, there is no key for it.
    pub fn get_element_unchecked(&self, k: ElementKey) -> &Element<Msg, Img> {
        &self.elements[k.0 as usize]
    }

    /// # Panic
    /// 
    /// May panic if the element does not exist. This is generally safe, since if an element
    /// does not exist, there is no key for it.
    pub fn get_element_mut_unchecked(&mut self, k: ElementKey) -> &mut Element<Msg, Img> {
        &mut self.elements[k.0 as usize]
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

#[derive(Debug, Clone)]
pub struct Selection {
    pub(crate) selectables: Vec<ElementKey>,
    pub(crate) current: Option<ElementKey>,
    pub locked: bool,
    pub menu_accessibility: bool,
}

impl Default for Selection {
    fn default() -> Self {
        Selection {
            selectables: Vec::new(),
            current: None,
            locked: false,
            menu_accessibility: false,
        }
    }
}

impl Selection {
    pub fn next(&mut self) -> Option<ElementKey> {
        self.current = match self.current {
            Some(current) => self
                .selectables
                .iter()
                .skip_while(|k| **k != current)
                .nth(1)
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
                .nth(1)
                .cloned(),
            None => self.selectables.last().cloned(),
        };
        self.current
    }
    pub fn clear(&mut self) {
        self.current = None;
        self.selectables.clear();
    }
    pub fn select_element(&mut self, key: ElementKey) -> (Option<ElementKey>, Option<ElementKey>) {
        let result = self.current;
        if self.selectables.contains(&key) {
            self.current = Some(key)
        } else {
            self.current = None
        }
        (result, self.current)
    }
    pub fn select_element_unchecked(&mut self, key: ElementKey) {
        self.current = Some(key)
    }
    pub fn current(&self) -> &Option<ElementKey> {
        &self.current
    }
}



#[cfg(test)]
mod tests {
    use std::{
        num::NonZero,
        time::{Duration, Instant},
    };

    use crate::{Element, Gui};

    #[test]
    pub fn benchmark() {
        let mut init_total = Duration::ZERO;
        let mut step_total = Duration::ZERO;

        const ITERATIONS: u32 = 10000;

        for _ in 0..ITERATIONS {
            let mut gui: Gui = Gui::new((NonZero::new(800).unwrap(), NonZero::new(800).unwrap()));

            let mut elem = Element::default();

            let mut children = Vec::new();
            for _ in 0..1000 {
                let elem = Element::default();

                let elem_key = gui.add_element(elem);
                children.push(elem_key);
            }
            elem.children = Some(children);

            let elem_key = gui.add_element(elem);

            gui.set_entry(elem_key);
            init_total += measure_task(|| gui.update(0.0), None).1;
            step_total += measure_task(|| gui.update(0.0), None).1;
        }

        println!("-----------------");
        println!("BENCHMARK END");
        println!("");
        println!("init avg: {:?}", init_total / ITERATIONS);
        println!("step avg: {:?}", step_total / ITERATIONS);

        // results
        // initial
        // init avg: 7.485µs
        // step avg: 3.588µs
        //
        // moved container into own variable
        // init avg: 5.989µs
        // step avg: 2.889µs
        //
        // replaced HashMap<K, E> with Vec<E>
        // init avg: 4.856µs
        // step avg: 1.432µs
        //
        // nothing
        // init avg: 78.916µs
        // step avg: 15.219µs

        panic!("danda")
    }

    fn measure_task<T>(mut task: impl FnMut() -> T, label: Option<&str>) -> (T, Duration) {
        let start = Instant::now();
        let r = task();
        let dur = start.elapsed();
        if let Some(label) = label {
            println!("Task '{label}' took: {:?}", dur);
        }
        (r, dur)
    }
}
