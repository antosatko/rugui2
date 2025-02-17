use std::mem::MaybeUninit;

use colors::Colors;
use element::{Element, ElementKey};
use events::EventListener;
use rugui2::Gui;
use rugui2::*;
use rugui2_wgpu::texture::Texture;
use styles::{Container, Gradient, Image, Portion, Position, Value, Values};
use text::{Font, FontIdx, TextRepr};
use widgets::{OnEvent, WidgetControlFlow, WidgetManager};
use winit::window::CursorIcon;

use crate::{Msgs, Sides};

#[derive(Clone, Debug, Copy)]
pub enum ElementNames {
    Start,
    Orange,
    Close,
    Resume,
    ExitGame,
    Settings,
}

#[derive(Clone, Debug, Copy)]
pub enum Pages {
    Menu,
    Ingame,
    IngamePause,
    IngameEnd(Sides),
    Settings,
}

pub struct GuiManager {
    pub widgets: WidgetManager<Msgs, Texture, (), Actions>,
    pub page: Pages,
    pub menu: ElementKey,
    pub ingame: ElementKey,
    pub ingame_pause: ElementKey,
    pub ingame_end: ElementKey,
    pub ingame_end_overlay: ElementKey,
    pub settings: ElementKey,
    pub noto_font: FontIdx,
    pub mono_font: FontIdx,
}

#[derive(Clone, Debug, Copy)]
pub enum Actions {
    ChangePage(Pages),
    Button(ElementNames),
    Cursor(CursorIcon),
    None,
}

impl GuiManager {
    pub fn new(gui: &mut Gui<Msgs, Texture>, game_tex: Texture, imag: Texture) -> Self {
        let mono = gui
            .text_ctx
            .add_font(Font::from_bytes(include_bytes!("SpaceMono-Regular.ttf"), 0).unwrap());
        gui.text_ctx.add_font(Font::from_bytes(include_bytes!("NotoEmoji-Regular.ttf"), 0).unwrap());
        let noto = gui.text_ctx.add_font(Font::from_bytes(include_bytes!("NotoSans-Medium.ttf"), 0).unwrap());
        let widgets = WidgetManager::new(&gui, Msgs::Widgets);
        let menu = new(gui, |gui, container| {
            container
                .styles_mut()
                .text
                .set(Some(TextRepr::new_editor(include_str!(
                    "../../../rugui2_wgpu/src/shaders/glyph.wgsl"
                ))));
            container.events.add(EventListener::new(events::ElemEventTypes::TextInput));
            container.styles_mut().text_wrap.set(styles::TextWrap::Wrap);
            container.styles_mut().font_size.set(Value::Px(14.0));
            //container.styles_mut().text_align.set(styles::TextAlign::Center);
            let mut children = Vec::new();
            children.push(
                widgets
                    .rows_builder(3)
                    .modify_height(|v| Value::Mul(Box::new((v, Value::Px(0.7)))))
                    .build(
                        |e, _| {
                            let styles = e.styles_mut();
                            styles.width.set(Value::Value(
                                Container::Container,
                                Values::Width,
                                Portion::Mul(0.4),
                            ));
                            styles.height.set(Value::Value(
                                Container::Container,
                                Values::Height,
                                Portion::Mul(0.65),
                            ));
                            styles.origin.get_mut().height =
                                Value::Value(Container::This, Values::Height, Portion::Full);
                            styles.position.get_mut().height =
                                Value::Value(Container::Container, Values::Height, Portion::Full);
                        },
                        |i, e, gui| {
                            let styles = e.styles_mut();
                            match i {
                                0 => {
                                    styles.color.set(Colors::GREEN);
                                    styles.text.set(Some(text::TextRepr::new_label("Start!")));
                                    styles.font_size.set(Value::Value(
                                        Container::This,
                                        Values::Height,
                                        Portion::Mul(0.95),
                                    ));
                                    styles.font_color.set(Colors::RED);
                                    buttonify(e, gui, &widgets, |_| {
                                        Actions::Button(ElementNames::Start)
                                    });
                                }
                                1 => {
                                    styles.color.set(Colors::ORANGE);
                                    styles.font_size.set(Value::Value(
                                        Container::This,
                                        Values::Height,
                                        Portion::Mul(0.95),
                                    ));
                                    styles.text.set(Some(text::TextRepr::new_label("Danda :3")));
                                    buttonify(e, gui, &widgets, |_| {
                                        Actions::Button(ElementNames::Orange)
                                    });
                                }
                                _ => {
                                    styles.color.set(Colors::RED);
                                    styles.font_size.set(Value::Value(
                                        Container::This,
                                        Values::Height,
                                        Portion::Mul(0.95),
                                    ));
                                    styles.text.set(Some(text::TextRepr::new_label("Leaving?")));
                                    buttonify(e, gui, &widgets, |_| {
                                        Actions::Button(ElementNames::Close)
                                    });
                                }
                            }

                            WidgetControlFlow::Done
                        },
                        gui,
                    ),
            );
            container.children = Some(children)
        });
        let ingame = new(gui, |_, container| {
            let styles = container.styles_mut();

            styles.image.set(Some(Image {
                data: game_tex.clone(),
            }));
            container
                .events
                .add(EventListener::new(events::ElemEventTypes::KeyPress).with_msg(Msgs::Ingame));
        });

        let ingame_pause = new(gui, |gui, e| {
            let overlay = widgets
                .rows_builder(2)
                .modify_height(|v| Value::Mul(Box::new((v, Value::Px(0.4)))))
                .build(
                    |container, _| {
                        let styles = container.styles_mut();
                        styles.width.set(Value::Value(
                            Container::Container,
                            Values::Width,
                            Portion::Mul(0.8),
                        ));
                        styles.height.set(Value::Value(
                            Container::Container,
                            Values::Height,
                            Portion::Mul(0.8),
                        ));
                        styles.color.set(Colors::WHITE.with_alpha(0.1));
                    },
                    |i, e, gui| {
                        let styles = e.styles_mut();
                        styles.width.set(Value::Value(
                            Container::Container,
                            Values::Width,
                            Portion::Half,
                        ));
                        styles.origin.set(Position {
                            container: Container::This,
                            width: Value::Value(Container::This, Values::Width, Portion::Half),
                            height: Value::Value(Container::This, Values::Height, Portion::Zero)
                        });
                        match i {
                            0 => {
                                styles.color.set(Colors::GREEN);
                                buttonify(e, gui, &widgets, |_| {
                                    Actions::Button(ElementNames::Resume)
                                });
                            }
                            _ => {
                                styles.color.set(Colors::RED);
                                buttonify(e, gui, &widgets, |_| {
                                    Actions::Button(ElementNames::ExitGame)
                                });
                            }
                        }
                        WidgetControlFlow::Done
                    },
                    gui,
                );
            e.children = Some(vec![ingame, overlay]);
        });
        let mut ingame_end_overlay = unsafe {
            #[allow(invalid_value)]
            MaybeUninit::uninit().assume_init()
        };
        let ingame_end = new(gui, |gui, e| {
            let overlay = new(gui, |_, e| {
                let styles = e.styles_mut();
                styles.grad_linear.set(Some(Gradient {
                    p1: (
                        Position {
                            container: Container::This,
                            height: Value::Px(0.0),
                            width: Value::Px(0.0),
                        },
                        Colors::GREEN.with_alpha(0.3),
                    ),
                    p2: (
                        Position {
                            container: Container::This,
                            height: Value::Px(0.0),
                            width: Value::Value(Container::This, Values::Width, Portion::Full),
                        },
                        Colors::RED.with_alpha(0.3),
                    ),
                }));
            });
            ingame_end_overlay = overlay;

            let button = new(gui, |gui, e| {
                let styles = e.styles_mut();
                styles.color.set(Colors::RED);
                styles.width.set(Value::Value(
                    Container::Container,
                    Values::Width,
                    Portion::Half,
                ));
                styles.height.set(Value::Value(
                    Container::Container,
                    Values::Height,
                    Portion::Mul(0.1),
                ));
                styles.position.get_mut().height =
                    Value::Value(Container::Container, Values::Height, Portion::Mul(0.8));

                buttonify(e, gui, &widgets, |_| {
                    Actions::Button(ElementNames::ExitGame)
                });
            });

            e.children = Some(vec![ingame, overlay, button]);
        });

        let settings = new(gui, |gui, e| {
            e.styles_mut().image.set(Some(Image { data: imag }));
            e.events
                .add(EventListener::new(events::ElemEventTypes::KeyPress).with_msg(Msgs::Settings));
            let b = new(gui, |gui, e| {
                let styles = e.styles_mut();

                let p = styles.position.get_mut();
                p.width = Value::Value(Container::Container, Values::Width, Portion::Mul(0.95));
                p.height = Value::Value(Container::Container, Values::Height, Portion::Mul(0.95));
                let o = styles.origin.get_mut();
                o.width = Value::Value(Container::This, Values::Width, Portion::Full);
                o.height = Value::Value(Container::This, Values::Height, Portion::Full);
                styles.color.set(Colors::ORANGE);
                styles.width.set(Value::Value(
                    Container::Container,
                    Values::Width,
                    Portion::Mul(0.3),
                ));
                styles.height.set(Value::Value(
                    Container::Container,
                    Values::Height,
                    Portion::Mul(0.1),
                ));
                styles.text.set(Some(TextRepr::new_label("Handsome")));
                styles
                    .font_size
                    .set(Value::Value(Container::This, Values::Height, Portion::Full));
                buttonify(e, gui, &widgets, |_| {
                    Actions::Button(ElementNames::ExitGame)
                });
            });
            e.children = Some(vec![b])
        });

        gui.set_entry(menu);
        gui.selection.select_element_unchecked(menu);
        Self {
            widgets,
            menu,
            ingame,
            ingame_pause,
            ingame_end,
            ingame_end_overlay,
            settings,
            page: Pages::Menu,
            noto_font: noto,
            mono_font: mono,
        }
    }

    pub fn get_page_key(&self, page: Pages) -> ElementKey {
        match page {
            Pages::Menu => self.menu,
            Pages::Ingame => self.ingame,
            Pages::IngamePause => self.ingame_pause,
            Pages::IngameEnd(_) => self.ingame_end,
            Pages::Settings => self.settings,
        }
    }

    pub fn change_page(&mut self, gui: &mut Gui<Msgs, Texture>, page: Pages) {
        match page {
            Pages::Ingame => {
                gui.selection.menu_accessibility = false;
            }
            Pages::Menu => {
                gui.selection.menu_accessibility = true;
            }
            Pages::IngamePause => {
                gui.selection.menu_accessibility = true;
            }
            Pages::IngameEnd(side) => {
                gui.selection.menu_accessibility = true;
                let e = gui.get_element_mut_unchecked(self.ingame_end_overlay);
                let grad = e.styles_mut().grad_linear.get_mut().as_mut().unwrap();
                match side {
                    Sides::Left => {
                        grad.p1.1 = Colors::GREEN.with_alpha(0.3);
                        grad.p2.1 = Colors::RED.with_alpha(0.3);
                    }
                    Sides::Right => {
                        grad.p1.1 = Colors::RED.with_alpha(0.3);
                        grad.p2.1 = Colors::GREEN.with_alpha(0.3);
                    }
                }
            }
            Pages::Settings => {}
        }
        gui.set_entry(self.get_page_key(page));
        self.page = page;
    }
}

fn new(
    gui: &mut Gui<Msgs, Texture>,
    cb: impl FnOnce(&mut Gui<Msgs, Texture>, &mut Element<Msgs, Texture>),
) -> ElementKey {
    let mut e = Element::default();

    cb(gui, &mut e);

    gui.add_element(e)
}

fn buttonify(
    e: &mut Element<Msgs, Texture>,
    gui: &mut Gui<Msgs, Texture>,
    widgets: &WidgetManager<Msgs, Texture, (), Actions>,
    confirm: OnEvent<Msgs, Texture, (), Actions>,
) {
    widgets.button(
        e,
        confirm,
        |args| {
            let child = args.element().child(0).unwrap();
            let e = args.gui.get_element_mut_unchecked(*child);
            let styles = e.styles_mut();
            styles.color.set(Colors::BLACK.with_alpha(0.5));

            match args.mouse_based {
                true => Actions::Cursor(CursorIcon::Pointer),
                false => Actions::None,
            }
        },
        |args| {
            let child = args.element().child(0).unwrap();
            let e = args.gui.get_element_mut_unchecked(*child);
            e.styles_mut().color.set(Colors::TRANSPARENT);

            match args.mouse_based {
                true => Actions::Cursor(CursorIcon::Default),
                false => Actions::None,
            }
        },
    );
    let styles = e.styles_mut();
    styles.text_wrap.set(styles::TextWrap::Overflow);
    styles.text_align.set(styles::TextAlign::Center);
    styles.round.set(Some(Value::Value(
        Container::This,
        Values::Min,
        Portion::Half,
    )));
    styles.shadow.set(Some(Value::Value(
        Container::This,
        Values::Min,
        Portion::Mul(0.1),
    )));
    styles.shadow_alpha.set(0.1);

    e.children = Some(vec![new(gui, |_, e| {
        let styles = e.styles_mut();
        styles.round.set(Some(Value::Value(
            Container::This,
            Values::Min,
            Portion::Half,
        )));
        styles.shadow.set(Some(Value::Value(
            Container::This,
            Values::Min,
            Portion::Mul(0.1),
        )));
        styles.shadow_alpha.set(0.1);
        styles.grad_linear.set(Some(Gradient {
            p1: (
                Position {
                    container: Container::This,
                    height: Value::Px(0.0),
                    width: Value::Px(0.0),
                },
                Colors::WHITE.with_alpha(0.4),
            ),
            p2: (
                Position {
                    container: Container::This,
                    height: Value::Value(Container::This, Values::Height, Portion::Half),
                    width: Value::Value(Container::This, Values::Width, Portion::Mul(0.1)),
                },
                Colors::TRANSPARENT,
            ),
        }));
    })]);
}
