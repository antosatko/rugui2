use rugui2::{colors::Colors, element::Element, events::{ElemEventTypes, EventListener}, styles::{TextWrap, Value}, text::TextRepr, widgets::{SplitOptions, WidgetManager}, Gui};
use rugui2_wgpu::texture::Texture;
use winit::window::CursorIcon;

use crate::{Actions, Msgs, WidgetData};

pub fn init(widgets: &mut WidgetManager<Msgs, Texture, WidgetData, Actions>, gui: &mut Gui<Msgs, Texture>) {
    let mut entry = Element::default();
    let mut left = Element::default();
    let mut right = Element::default();
    editor_window(gui, &mut left);
    editor_window(gui, &mut right);
    let left = gui.add_element(left);
    let right = gui.add_element(right);

    let mut beam = Element::default();
    beam.styles_mut().width.set(Value::Px(3.0));
    beam.styles_mut().color.set(Colors::WHITE);
    beam.events.add(EventListener::new(ElemEventTypes::MouseEnter).with_msg(Msgs::Cursor(CursorIcon::ColResize)));
    let beam = gui.add_element(beam);
    let entry_key = gui.add_element(entry);

    widgets.horizontal_split(gui, entry_key, left, right, &SplitOptions::Dynamic {
        split: None,
        beam,
    }, |_| Actions::None, |_| Actions::None);

    gui.set_entry(entry_key);
}


pub fn editor_window(gui: &mut Gui<Msgs, Texture>, element: &mut Element<Msgs, Texture>){
    element.events.add(EventListener::new(ElemEventTypes::TextInput));
    element.events.add(EventListener::new(ElemEventTypes::MouseEnter).with_msg(Msgs::Cursor(CursorIcon::Text)));
    let styles = element.styles_mut();
    styles.text.set(Some(TextRepr::new_editor("danda")));
    styles.font_size.set(Value::Px(18.0));
    styles.text_wrap.set(TextWrap::Overflow);
}