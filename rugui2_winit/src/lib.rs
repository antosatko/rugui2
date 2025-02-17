use rugui2::{
    events::{EnvEventStates, Key}, math::Vector, styles::ImageData, text::{MoveCommand, TextRepr}, Gui
};
use winit::{
    dpi::PhysicalPosition,
    event::WindowEvent,
    keyboard::{KeyCode, NamedKey, PhysicalKey},
};

pub struct EventContext {
    pub pressed_ctrl: bool,
    pub pressed_shift: bool,
    #[cfg(feature = "clipboard")]
    pub clipboard: Option<arboard::Clipboard>,
}

impl EventContext {
    pub fn new() -> Self {
        Self {
            pressed_ctrl: false,
            pressed_shift: false,
            #[cfg(feature = "clipboard")]
            clipboard: arboard::Clipboard::new().ok(),
        }
    }
    pub fn event<Msg: Clone, Img: Clone + ImageData>(
        &mut self,
        winit: &WindowEvent,
        gui: &mut Gui<Msg, Img>,
    ) -> EnvEventStates {
        match winit {
            WindowEvent::DroppedFile(path_buf) => {
                gui.env_event(rugui2::events::EnvEvents::FileDrop {
                    path: Some(path_buf.clone()),
                    opt: rugui2::events::FileDropOpts::Drop,
                })
            }
            WindowEvent::HoveredFile(path_buf) => {
                gui.env_event(rugui2::events::EnvEvents::FileDrop {
                    path: Some(path_buf.clone()),
                    opt: rugui2::events::FileDropOpts::Hover,
                })
            }
            WindowEvent::HoveredFileCancelled => {
                gui.env_event(rugui2::events::EnvEvents::FileDrop {
                    path: None,
                    opt: rugui2::events::FileDropOpts::Cancel,
                })
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                let press = event.state == winit::event::ElementState::Pressed;
                if let (true, Some(key)) = (press, gui.selection.current()) {
                    if let Some(text) = gui.get_element_mut_unchecked(*key).styles_mut().text.get_mut() {
                        match (&event.logical_key, self.pressed_ctrl) {
                            (winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp), false) => {
                                return text.move_cursor(MoveCommand{
                                    cmd: rugui2::text::MoveCommands::MoveChar,
                                    direction: rugui2::text::Directions::Up,
                                    hold_select: self.pressed_shift
                                })
                            }
                            (winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown), false) => {
                                return text.move_cursor(MoveCommand{
                                    cmd: rugui2::text::MoveCommands::MoveChar,
                                    direction: rugui2::text::Directions::Down,
                                    hold_select: self.pressed_shift
                                })
                            }
                            (winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft), false) => {
                                return text.move_cursor(MoveCommand{
                                    cmd: rugui2::text::MoveCommands::MoveChar,
                                    direction: rugui2::text::Directions::Left,
                                    hold_select: self.pressed_shift
                                })
                            }
                            (winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight), false) => {
                                return text.move_cursor(MoveCommand{
                                    cmd: rugui2::text::MoveCommands::MoveChar,
                                    direction: rugui2::text::Directions::Right,
                                    hold_select: self.pressed_shift
                                })
                            }
                            (winit::keyboard::Key::Named(winit::keyboard::NamedKey::Home), false) => {
                                return text.move_cursor(MoveCommand{
                                    cmd: rugui2::text::MoveCommands::MoveLine,
                                    direction: rugui2::text::Directions::Left,
                                    hold_select: self.pressed_shift
                                })
                            }
                            (winit::keyboard::Key::Named(winit::keyboard::NamedKey::End), false) => {
                                return text.move_cursor(MoveCommand{
                                    cmd: rugui2::text::MoveCommands::MoveLine,
                                    direction: rugui2::text::Directions::Right,
                                    hold_select: self.pressed_shift
                                })
                            }
                            (winit::keyboard::Key::Named(winit::keyboard::NamedKey::Home), true) => {
                                return text.move_cursor(MoveCommand{
                                    cmd: rugui2::text::MoveCommands::MoveLine,
                                    direction: rugui2::text::Directions::Up,
                                    hold_select: self.pressed_shift
                                })
                            }
                            (winit::keyboard::Key::Named(winit::keyboard::NamedKey::End), true) => {
                                return text.move_cursor(MoveCommand{
                                    cmd: rugui2::text::MoveCommands::MoveLine,
                                    direction: rugui2::text::Directions::Down,
                                    hold_select: self.pressed_shift
                                })
                            }
                            (winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace), _) => {
                                return text.remove()
                            }
                            (winit::keyboard::Key::Named(winit::keyboard::NamedKey::Delete), _) => {
                                return text.delete()
                            }
                            _ => ()
                        }
                        match (&event.physical_key, self.pressed_ctrl) {
                            (winit::keyboard::PhysicalKey::Code(KeyCode::KeyA), true) => {
                                return text.select_all();
                            }
                            _ => ()
                        }
                    }
                }
                if press && !gui.selection.locked {
                    match event.logical_key {
                        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab) => {
                            return gui.env_event(rugui2::events::EnvEvents::Select {
                                opt: rugui2::events::SelectOpts::Next,
                            })
                        }
                        winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter) => {
                            return gui.env_event(rugui2::events::EnvEvents::Select {
                                opt: rugui2::events::SelectOpts::Confirm,
                            })
                        }
                        _ => (),
                    }
                    if gui.selection.menu_accessibility {
                        match event.logical_key {
                            winit::keyboard::Key::Named(NamedKey::ArrowDown) => {
                                return gui.env_event(rugui2::events::EnvEvents::Select {
                                    opt: rugui2::events::SelectOpts::Next,
                                })
                            }
                            winit::keyboard::Key::Named(NamedKey::ArrowRight) => {
                                return gui.env_event(rugui2::events::EnvEvents::Select {
                                    opt: rugui2::events::SelectOpts::Next,
                                })
                            }
                            winit::keyboard::Key::Named(NamedKey::ArrowUp) => {
                                return gui.env_event(rugui2::events::EnvEvents::Select {
                                    opt: rugui2::events::SelectOpts::Prev,
                                })
                            }
                            winit::keyboard::Key::Named(NamedKey::ArrowLeft) => {
                                return gui.env_event(rugui2::events::EnvEvents::Select {
                                    opt: rugui2::events::SelectOpts::Prev,
                                })
                            }
                            winit::keyboard::Key::Named(NamedKey::Escape) => {
                                return gui.env_event(rugui2::events::EnvEvents::Select {
                                    opt: rugui2::events::SelectOpts::NoFocus,
                                })
                            }
                            _ => (),
                        }
                    }
                }
                if let winit::keyboard::Key::Named(key) = &event.logical_key {
                    if let NamedKey::Control = key {
                        self.pressed_ctrl = press;
                    }
                    if let NamedKey::Shift = key {
                        self.pressed_shift = press;
                    }
                    if gui.selection.current().is_some() && !self.pressed_ctrl {
                        if let Some(txt) = &event.text {
                            gui.env_event(rugui2::events::EnvEvents::Input {
                                text: txt.to_string(),
                            });
                        } else if let PhysicalKey::Code(KeyCode::Enter) = event.physical_key {
                            gui.env_event(rugui2::events::EnvEvents::Input {
                                text: String::from("\n"),
                            });
                        }
                    }
                    gui.env_event(rugui2::events::EnvEvents::KeyPress {
                        key: winit_2_rugui_key(key),
                        press,
                    })
                } else if let PhysicalKey::Code(key) = event.physical_key {
                    match (key, press, self.pressed_ctrl) {
                        #[cfg(feature = "clipboard")]
                        (KeyCode::KeyC, true, true) => {
                            if let (Some(ctx), Some(txt)) =
                                (&mut self.clipboard, gui.copy_selection_text())
                            {
                                let _ = ctx.set_text(txt);
                            }
                            gui.env_event(rugui2::events::EnvEvents::Copy)
                        }
                        #[cfg(feature = "clipboard")]
                        (KeyCode::KeyV, true, true) => {
                            if let Some(ctx) = &mut self.clipboard {
                                if let Ok(txt) = ctx.get_text() {
                                    if gui.selection.current().is_some() {
                                        gui.env_event(rugui2::events::EnvEvents::Input {
                                            text: txt,
                                        })
                                    } else {
                                        EnvEventStates::Free
                                    }
                                } else {
                                    EnvEventStates::Free
                                }
                            } else {
                                EnvEventStates::Free
                            }
                        }
                        _ => {
                            if gui.selection.current().is_some() && !self.pressed_ctrl {
                                if let Some(txt) = &event.text {
                                    gui.env_event(rugui2::events::EnvEvents::Input {
                                        text: txt.to_string(),
                                    });
                                } else if let PhysicalKey::Code(KeyCode::Enter) = event.physical_key
                                {
                                    gui.env_event(rugui2::events::EnvEvents::Input {
                                        text: String::from("\n"),
                                    });
                                }
                            }
                            gui.env_event(rugui2::events::EnvEvents::KeyPress {
                                key: winit_physical_to_rugui_key(key),
                                press,
                            })
                        }
                    }
                } else {
                    EnvEventStates::Free
                }
            }
            WindowEvent::ModifiersChanged(_) => EnvEventStates::Free,
            WindowEvent::CursorMoved { position, .. } => {
                gui.env_event(rugui2::events::EnvEvents::CursorMove {
                    pos: Vector(position.x as _, position.y as _),
                })
            }
            WindowEvent::MouseWheel { delta, .. } => {
                gui.env_event(rugui2::events::EnvEvents::Scroll {
                    delta: match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => Vector(*x, *y),
                        winit::event::MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
                            Vector(*x as _, *y as _)
                        }
                    },
                })
            }
            WindowEvent::MouseInput { state, button, .. } => {
                gui.env_event(rugui2::events::EnvEvents::MouseButton {
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
                })
            }
            WindowEvent::PinchGesture { .. } => EnvEventStates::Free,
            WindowEvent::PanGesture { .. } => EnvEventStates::Free,
            WindowEvent::DoubleTapGesture { .. } => EnvEventStates::Free,
            WindowEvent::RotationGesture { .. } => EnvEventStates::Free,
            WindowEvent::TouchpadPressure { .. } => EnvEventStates::Free,
            WindowEvent::Touch(_) => EnvEventStates::Free,
            _ => EnvEventStates::Free,
        }
    }
}

fn winit_2_rugui_key(key: &NamedKey) -> Key {
    match key {
        NamedKey::Alt => Key::Alt,
        NamedKey::AltGraph => Key::AltGraph,
        NamedKey::CapsLock => Key::CapsLock,
        NamedKey::Control => Key::Control,
        NamedKey::Fn => Key::Fn,
        NamedKey::FnLock => Key::FnLock,
        NamedKey::NumLock => Key::NumLock,
        NamedKey::ScrollLock => Key::ScrollLock,
        NamedKey::Shift => Key::Shift,
        NamedKey::Symbol => Key::Symbol,
        NamedKey::SymbolLock => Key::SymbolLock,
        NamedKey::Meta => Key::Meta,
        NamedKey::Hyper => Key::Hyper,
        NamedKey::Super => Key::Super,
        NamedKey::Enter => Key::Enter,
        NamedKey::Tab => Key::Tab,
        NamedKey::Space => Key::Space,
        NamedKey::ArrowDown => Key::ArrowDown,
        NamedKey::ArrowLeft => Key::ArrowLeft,
        NamedKey::ArrowRight => Key::ArrowRight,
        NamedKey::ArrowUp => Key::ArrowUp,
        NamedKey::End => Key::End,
        NamedKey::Home => Key::Home,
        NamedKey::PageDown => Key::PageDown,
        NamedKey::PageUp => Key::PageUp,
        NamedKey::Backspace => Key::Backspace,
        NamedKey::Clear => Key::Clear,
        NamedKey::Copy => Key::Copy,
        NamedKey::CrSel => Key::CrSel,
        NamedKey::Cut => Key::Cut,
        NamedKey::Delete => Key::Delete,
        NamedKey::EraseEof => Key::EraseEof,
        NamedKey::ExSel => Key::ExSel,
        NamedKey::Insert => Key::Insert,
        NamedKey::Paste => Key::Paste,
        NamedKey::Redo => Key::Redo,
        NamedKey::Undo => Key::Undo,
        NamedKey::Accept => Key::Accept,
        NamedKey::Again => Key::Again,
        NamedKey::Attn => Key::Attn,
        NamedKey::Cancel => Key::Cancel,
        NamedKey::ContextMenu => Key::ContextMenu,
        NamedKey::Escape => Key::Escape,
        NamedKey::Execute => Key::Execute,
        NamedKey::Find => Key::Find,
        NamedKey::Help => Key::Help,
        NamedKey::Pause => Key::Pause,
        NamedKey::Play => Key::Play,
        NamedKey::Props => Key::Props,
        NamedKey::Select => Key::Select,
        NamedKey::ZoomIn => Key::ZoomIn,
        NamedKey::ZoomOut => Key::ZoomOut,
        NamedKey::BrightnessDown => Key::BrightnessDown,
        NamedKey::BrightnessUp => Key::BrightnessUp,
        NamedKey::Eject => Key::Eject,
        NamedKey::LogOff => Key::LogOff,
        NamedKey::Power => Key::Power,
        NamedKey::PowerOff => Key::PowerOff,
        NamedKey::PrintScreen => Key::PrintScreen,
        NamedKey::Hibernate => Key::Hibernate,
        NamedKey::Standby => Key::Standby,
        NamedKey::WakeUp => Key::WakeUp,
        NamedKey::AllCandidates => Key::AllCandidates,
        NamedKey::Alphanumeric => Key::Alphanumeric,
        NamedKey::CodeInput => Key::CodeInput,
        NamedKey::Compose => Key::Compose,
        NamedKey::Convert => Key::Convert,
        NamedKey::FinalMode => Key::FinalMode,
        NamedKey::GroupFirst => Key::GroupFirst,
        NamedKey::GroupLast => Key::GroupLast,
        NamedKey::GroupNext => Key::GroupNext,
        NamedKey::GroupPrevious => Key::GroupPrevious,
        NamedKey::ModeChange => Key::ModeChange,
        NamedKey::NextCandidate => Key::NextCandidate,
        NamedKey::NonConvert => Key::NonConvert,
        NamedKey::PreviousCandidate => Key::PreviousCandidate,
        NamedKey::Process => Key::Process,
        NamedKey::SingleCandidate => Key::SingleCandidate,
        NamedKey::HangulMode => Key::HangulMode,
        NamedKey::HanjaMode => Key::HanjaMode,
        NamedKey::JunjaMode => Key::JunjaMode,
        NamedKey::Eisu => Key::Eisu,
        NamedKey::Hankaku => Key::Hankaku,
        NamedKey::Hiragana => Key::Hiragana,
        NamedKey::HiraganaKatakana => Key::HiraganaKatakana,
        NamedKey::KanaMode => Key::KanaMode,
        NamedKey::KanjiMode => Key::KanjiMode,
        NamedKey::Katakana => Key::Katakana,
        NamedKey::Romaji => Key::Romaji,
        NamedKey::Zenkaku => Key::Zenkaku,
        NamedKey::ZenkakuHankaku => Key::ZenkakuHankaku,
        NamedKey::Soft1 => Key::Soft1,
        NamedKey::Soft2 => Key::Soft2,
        NamedKey::Soft3 => Key::Soft3,
        NamedKey::Soft4 => Key::Soft4,
        NamedKey::ChannelDown => Key::ChannelDown,
        NamedKey::ChannelUp => Key::ChannelUp,
        NamedKey::Close => Key::Close,
        NamedKey::MailForward => Key::MailForward,
        NamedKey::MailReply => Key::MailReply,
        NamedKey::MailSend => Key::MailSend,
        NamedKey::MediaClose => Key::MediaClose,
        NamedKey::MediaFastForward => Key::MediaFastForward,
        NamedKey::MediaPause => Key::MediaPause,
        NamedKey::MediaPlay => Key::MediaPlay,
        NamedKey::MediaPlayPause => Key::MediaPlayPause,
        NamedKey::MediaRecord => Key::MediaRecord,
        NamedKey::MediaRewind => Key::MediaRewind,
        NamedKey::MediaStop => Key::MediaStop,
        NamedKey::MediaTrackNext => Key::MediaTrackNext,
        NamedKey::MediaTrackPrevious => Key::MediaTrackPrevious,
        NamedKey::New => Key::New,
        NamedKey::Open => Key::Open,
        NamedKey::Print => Key::Print,
        NamedKey::Save => Key::Save,
        NamedKey::SpellCheck => Key::SpellCheck,
        NamedKey::Key11 => Key::Key11,
        NamedKey::Key12 => Key::Key12,
        NamedKey::AudioBalanceLeft => Key::AudioBalanceLeft,
        NamedKey::AudioBalanceRight => Key::AudioBalanceRight,
        NamedKey::AudioBassBoostDown => Key::AudioBassBoostDown,
        NamedKey::AudioBassBoostToggle => Key::AudioBassBoostToggle,
        NamedKey::AudioBassBoostUp => Key::AudioBassBoostUp,
        NamedKey::AudioFaderFront => Key::AudioFaderFront,
        NamedKey::AudioFaderRear => Key::AudioFaderRear,
        NamedKey::AudioSurroundModeNext => Key::AudioSurroundModeNext,
        NamedKey::AudioTrebleDown => Key::AudioTrebleDown,
        NamedKey::AudioTrebleUp => Key::AudioTrebleUp,
        NamedKey::AudioVolumeDown => Key::AudioVolumeDown,
        NamedKey::AudioVolumeUp => Key::AudioVolumeUp,
        NamedKey::AudioVolumeMute => Key::AudioVolumeMute,
        NamedKey::MicrophoneToggle => Key::MicrophoneToggle,
        NamedKey::MicrophoneVolumeDown => Key::MicrophoneVolumeDown,
        NamedKey::MicrophoneVolumeUp => Key::MicrophoneVolumeUp,
        NamedKey::MicrophoneVolumeMute => Key::MicrophoneVolumeMute,
        NamedKey::SpeechCorrectionList => Key::SpeechCorrectionList,
        NamedKey::SpeechInputToggle => Key::SpeechInputToggle,
        NamedKey::LaunchApplication1 => Key::LaunchApplication1,
        NamedKey::LaunchApplication2 => Key::LaunchApplication2,
        NamedKey::LaunchCalendar => Key::LaunchCalendar,
        NamedKey::LaunchContacts => Key::LaunchContacts,
        NamedKey::LaunchMail => Key::LaunchMail,
        NamedKey::LaunchMediaPlayer => Key::LaunchMediaPlayer,
        NamedKey::LaunchMusicPlayer => Key::LaunchMusicPlayer,
        NamedKey::LaunchPhone => Key::LaunchPhone,
        NamedKey::LaunchScreenSaver => Key::LaunchScreenSaver,
        NamedKey::LaunchSpreadsheet => Key::LaunchSpreadsheet,
        NamedKey::LaunchWebBrowser => Key::LaunchWebBrowser,
        NamedKey::LaunchWebCam => Key::LaunchWebCam,
        NamedKey::LaunchWordProcessor => Key::LaunchWordProcessor,
        NamedKey::BrowserBack => Key::BrowserBack,
        NamedKey::BrowserFavorites => Key::BrowserFavorites,
        NamedKey::BrowserForward => Key::BrowserForward,
        NamedKey::BrowserHome => Key::BrowserHome,
        NamedKey::BrowserRefresh => Key::BrowserRefresh,
        NamedKey::BrowserSearch => Key::BrowserSearch,
        NamedKey::BrowserStop => Key::BrowserStop,
        NamedKey::AppSwitch => Key::AppSwitch,
        NamedKey::Call => Key::Call,
        NamedKey::Camera => Key::Camera,
        NamedKey::CameraFocus => Key::CameraFocus,
        NamedKey::EndCall => Key::EndCall,
        NamedKey::GoBack => Key::GoBack,
        NamedKey::GoHome => Key::GoHome,
        NamedKey::HeadsetHook => Key::HeadsetHook,
        NamedKey::LastNumberRedial => Key::LastNumberRedial,
        NamedKey::Notification => Key::Notification,
        NamedKey::MannerMode => Key::MannerMode,
        NamedKey::VoiceDial => Key::VoiceDial,
        NamedKey::TV => Key::TV,
        NamedKey::TV3DMode => Key::TV3DMode,
        NamedKey::TVAntennaCable => Key::TVAntennaCable,
        NamedKey::TVAudioDescription => Key::TVAudioDescription,
        NamedKey::TVAudioDescriptionMixDown => Key::TVAudioDescriptionMixDown,
        NamedKey::TVAudioDescriptionMixUp => Key::TVAudioDescriptionMixUp,
        NamedKey::TVContentsMenu => Key::TVContentsMenu,
        NamedKey::TVDataService => Key::TVDataService,
        NamedKey::TVInput => Key::TVInput,
        NamedKey::TVInputComponent1 => Key::TVInputComponent1,
        NamedKey::TVInputComponent2 => Key::TVInputComponent2,
        NamedKey::TVInputComposite1 => Key::TVInputComposite1,
        NamedKey::TVInputComposite2 => Key::TVInputComposite2,
        NamedKey::TVInputHDMI1 => Key::TVInputHDMI1,
        NamedKey::TVInputHDMI2 => Key::TVInputHDMI2,
        NamedKey::TVInputHDMI3 => Key::TVInputHDMI3,
        NamedKey::TVInputHDMI4 => Key::TVInputHDMI4,
        NamedKey::TVInputVGA1 => Key::TVInputVGA1,
        NamedKey::TVMediaContext => Key::TVMediaContext,
        NamedKey::TVNetwork => Key::TVNetwork,
        NamedKey::TVNumberEntry => Key::TVNumberEntry,
        NamedKey::TVPower => Key::TVPower,
        NamedKey::TVRadioService => Key::TVRadioService,
        NamedKey::TVSatellite => Key::TVSatellite,
        NamedKey::TVSatelliteBS => Key::TVSatelliteBS,
        NamedKey::TVSatelliteCS => Key::TVSatelliteCS,
        NamedKey::TVSatelliteToggle => Key::TVSatelliteToggle,
        NamedKey::TVTerrestrialAnalog => Key::TVTerrestrialAnalog,
        NamedKey::TVTerrestrialDigital => Key::TVTerrestrialDigital,
        NamedKey::TVTimer => Key::TVTimer,
        NamedKey::AVRInput => Key::AVRInput,
        NamedKey::AVRPower => Key::AVRPower,
        NamedKey::ColorF0Red => Key::ColorF0Red,
        NamedKey::ColorF1Green => Key::ColorF1Green,
        NamedKey::ColorF2Yellow => Key::ColorF2Yellow,
        NamedKey::ColorF3Blue => Key::ColorF3Blue,
        NamedKey::ColorF4Grey => Key::ColorF4Grey,
        NamedKey::ColorF5Brown => Key::ColorF5Brown,
        NamedKey::ClosedCaptionToggle => Key::ClosedCaptionToggle,
        NamedKey::Dimmer => Key::Dimmer,
        NamedKey::DisplaySwap => Key::DisplaySwap,
        NamedKey::DVR => Key::DVR,
        NamedKey::Exit => Key::Exit,
        NamedKey::FavoriteClear0 => Key::FavoriteClear0,
        NamedKey::FavoriteClear1 => Key::FavoriteClear1,
        NamedKey::FavoriteClear2 => Key::FavoriteClear2,
        NamedKey::FavoriteClear3 => Key::FavoriteClear3,
        NamedKey::FavoriteRecall0 => Key::FavoriteRecall0,
        NamedKey::FavoriteRecall1 => Key::FavoriteRecall1,
        NamedKey::FavoriteRecall2 => Key::FavoriteRecall2,
        NamedKey::FavoriteRecall3 => Key::FavoriteRecall3,
        NamedKey::FavoriteStore0 => Key::FavoriteStore0,
        NamedKey::FavoriteStore1 => Key::FavoriteStore1,
        NamedKey::FavoriteStore2 => Key::FavoriteStore2,
        NamedKey::FavoriteStore3 => Key::FavoriteStore3,
        NamedKey::Guide => Key::Guide,
        NamedKey::GuideNextDay => Key::GuideNextDay,
        NamedKey::GuidePreviousDay => Key::GuidePreviousDay,
        NamedKey::Info => Key::Info,
        NamedKey::InstantReplay => Key::InstantReplay,
        NamedKey::Link => Key::Link,
        NamedKey::ListProgram => Key::ListProgram,
        NamedKey::LiveContent => Key::LiveContent,
        NamedKey::Lock => Key::Lock,
        NamedKey::MediaApps => Key::MediaApps,
        NamedKey::MediaAudioTrack => Key::MediaAudioTrack,
        NamedKey::MediaLast => Key::MediaLast,
        NamedKey::MediaSkipBackward => Key::MediaSkipBackward,
        NamedKey::MediaSkipForward => Key::MediaSkipForward,
        NamedKey::MediaStepBackward => Key::MediaStepBackward,
        NamedKey::MediaStepForward => Key::MediaStepForward,
        NamedKey::MediaTopMenu => Key::MediaTopMenu,
        NamedKey::NavigateIn => Key::NavigateIn,
        NamedKey::NavigateNext => Key::NavigateNext,
        NamedKey::NavigateOut => Key::NavigateOut,
        NamedKey::NavigatePrevious => Key::NavigatePrevious,
        NamedKey::NextFavoriteChannel => Key::NextFavoriteChannel,
        NamedKey::NextUserProfile => Key::NextUserProfile,
        NamedKey::OnDemand => Key::OnDemand,
        NamedKey::Pairing => Key::Pairing,
        NamedKey::PinPDown => Key::PinPDown,
        NamedKey::PinPMove => Key::PinPMove,
        NamedKey::PinPToggle => Key::PinPToggle,
        NamedKey::PinPUp => Key::PinPUp,
        NamedKey::PlaySpeedDown => Key::PlaySpeedDown,
        NamedKey::PlaySpeedReset => Key::PlaySpeedReset,
        NamedKey::PlaySpeedUp => Key::PlaySpeedUp,
        NamedKey::RandomToggle => Key::RandomToggle,
        NamedKey::RcLowBattery => Key::RcLowBattery,
        NamedKey::RecordSpeedNext => Key::RecordSpeedNext,
        NamedKey::RfBypass => Key::RfBypass,
        NamedKey::ScanChannelsToggle => Key::ScanChannelsToggle,
        NamedKey::ScreenModeNext => Key::ScreenModeNext,
        NamedKey::Settings => Key::Settings,
        NamedKey::SplitScreenToggle => Key::SplitScreenToggle,
        NamedKey::STBInput => Key::STBInput,
        NamedKey::STBPower => Key::STBPower,
        NamedKey::Subtitle => Key::Subtitle,
        NamedKey::Teletext => Key::Teletext,
        NamedKey::VideoModeNext => Key::VideoModeNext,
        NamedKey::Wink => Key::Wink,
        NamedKey::ZoomToggle => Key::ZoomToggle,
        NamedKey::F1 => Key::F1,
        NamedKey::F2 => Key::F2,
        NamedKey::F3 => Key::F3,
        NamedKey::F4 => Key::F4,
        NamedKey::F5 => Key::F5,
        NamedKey::F6 => Key::F6,
        NamedKey::F7 => Key::F7,
        NamedKey::F8 => Key::F8,
        NamedKey::F9 => Key::F9,
        NamedKey::F10 => Key::F10,
        NamedKey::F11 => Key::F11,
        NamedKey::F12 => Key::F12,
        NamedKey::F13 => Key::F13,
        NamedKey::F14 => Key::F14,
        NamedKey::F15 => Key::F15,
        NamedKey::F16 => Key::F16,
        NamedKey::F17 => Key::F17,
        NamedKey::F18 => Key::F18,
        NamedKey::F19 => Key::F19,
        NamedKey::F20 => Key::F20,
        NamedKey::F21 => Key::F21,
        NamedKey::F22 => Key::F22,
        NamedKey::F23 => Key::F23,
        NamedKey::F24 => Key::F24,
        NamedKey::F25 => Key::F25,
        NamedKey::F26 => Key::F26,
        NamedKey::F27 => Key::F27,
        NamedKey::F28 => Key::F28,
        NamedKey::F29 => Key::F29,
        NamedKey::F30 => Key::F30,
        NamedKey::F31 => Key::F31,
        NamedKey::F32 => Key::F32,
        NamedKey::F33 => Key::F33,
        NamedKey::F34 => Key::F34,
        NamedKey::F35 => Key::F35,
        _ => todo!(),
    }
}

fn winit_physical_to_rugui_key(key: KeyCode) -> Key {
    match key {
        KeyCode::Backquote => Key::Backquote,
        KeyCode::Backslash => Key::Backslash,
        KeyCode::BracketLeft => Key::BracketLeft,
        KeyCode::BracketRight => Key::BracketRight,
        KeyCode::Comma => Key::Comma,
        KeyCode::Digit0 => Key::Digit0,
        KeyCode::Digit1 => Key::Digit1,
        KeyCode::Digit2 => Key::Digit2,
        KeyCode::Digit3 => Key::Digit3,
        KeyCode::Digit4 => Key::Digit4,
        KeyCode::Digit5 => Key::Digit5,
        KeyCode::Digit6 => Key::Digit6,
        KeyCode::Digit7 => Key::Digit7,
        KeyCode::Digit8 => Key::Digit8,
        KeyCode::Digit9 => Key::Digit9,
        KeyCode::Equal => Key::Equal,
        KeyCode::IntlBackslash => Key::IntlBackslash,
        KeyCode::IntlRo => Key::IntlRo,
        KeyCode::IntlYen => Key::IntlYen,
        KeyCode::KeyA => Key::KeyA,
        KeyCode::KeyB => Key::KeyB,
        KeyCode::KeyC => Key::KeyC,
        KeyCode::KeyD => Key::KeyD,
        KeyCode::KeyE => Key::KeyE,
        KeyCode::KeyF => Key::KeyF,
        KeyCode::KeyG => Key::KeyG,
        KeyCode::KeyH => Key::KeyH,
        KeyCode::KeyI => Key::KeyI,
        KeyCode::KeyJ => Key::KeyJ,
        KeyCode::KeyK => Key::KeyK,
        KeyCode::KeyL => Key::KeyL,
        KeyCode::KeyM => Key::KeyM,
        KeyCode::KeyN => Key::KeyN,
        KeyCode::KeyO => Key::KeyO,
        KeyCode::KeyP => Key::KeyP,
        KeyCode::KeyQ => Key::KeyQ,
        KeyCode::KeyR => Key::KeyR,
        KeyCode::KeyS => Key::KeyS,
        KeyCode::KeyT => Key::KeyT,
        KeyCode::KeyU => Key::KeyU,
        KeyCode::KeyV => Key::KeyV,
        KeyCode::KeyW => Key::KeyW,
        KeyCode::KeyX => Key::KeyX,
        KeyCode::KeyY => Key::KeyY,
        KeyCode::KeyZ => Key::KeyZ,
        KeyCode::Minus => Key::Minus,
        KeyCode::Period => Key::Period,
        KeyCode::Quote => Key::Quote,
        KeyCode::Semicolon => Key::Semicolon,
        KeyCode::Slash => Key::Slash,
        KeyCode::AltLeft => Key::AltLeft,
        KeyCode::AltRight => Key::AltRight,
        KeyCode::ControlLeft => Key::ControlLeft,
        KeyCode::ControlRight => Key::ControlRight,
        KeyCode::SuperLeft => Key::SuperLeft,
        KeyCode::SuperRight => Key::SuperRight,
        KeyCode::ShiftLeft => Key::ShiftLeft,
        KeyCode::ShiftRight => Key::ShiftRight,
        KeyCode::Lang1 => Key::Lang1,
        KeyCode::Lang2 => Key::Lang2,
        KeyCode::Lang3 => Key::Lang3,
        KeyCode::Lang4 => Key::Lang4,
        KeyCode::Lang5 => Key::Lang5,
        KeyCode::Numpad0 => Key::Numpad0,
        KeyCode::Numpad1 => Key::Numpad1,
        KeyCode::Numpad2 => Key::Numpad2,
        KeyCode::Numpad3 => Key::Numpad3,
        KeyCode::Numpad4 => Key::Numpad4,
        KeyCode::Numpad5 => Key::Numpad5,
        KeyCode::Numpad6 => Key::Numpad6,
        KeyCode::Numpad7 => Key::Numpad7,
        KeyCode::Numpad8 => Key::Numpad8,
        KeyCode::Numpad9 => Key::Numpad9,
        KeyCode::NumpadAdd => Key::NumpadAdd,
        KeyCode::NumpadBackspace => Key::NumpadBackspace,
        KeyCode::NumpadClear => Key::NumpadClear,
        KeyCode::NumpadClearEntry => Key::NumpadClearEntry,
        KeyCode::NumpadComma => Key::NumpadComma,
        KeyCode::NumpadDecimal => Key::NumpadDecimal,
        KeyCode::NumpadDivide => Key::NumpadDivide,
        KeyCode::NumpadEnter => Key::NumpadEnter,
        KeyCode::NumpadEqual => Key::NumpadEqual,
        KeyCode::NumpadHash => Key::NumpadHash,
        KeyCode::NumpadMemoryAdd => Key::NumpadMemoryAdd,
        KeyCode::NumpadMemoryClear => Key::NumpadMemoryClear,
        KeyCode::NumpadMemoryRecall => Key::NumpadMemoryRecall,
        KeyCode::NumpadMemoryStore => Key::NumpadMemoryStore,
        KeyCode::NumpadMemorySubtract => Key::NumpadMemorySubtract,
        KeyCode::NumpadMultiply => Key::NumpadMultiply,
        KeyCode::NumpadParenLeft => Key::NumpadParenLeft,
        KeyCode::NumpadParenRight => Key::NumpadParenRight,
        KeyCode::NumpadStar => Key::NumpadStar,
        KeyCode::NumpadSubtract => Key::NumpadSubtract,
        KeyCode::LaunchApp1 => Key::LaunchApp1,
        KeyCode::LaunchApp2 => Key::LaunchApp2,
        KeyCode::MediaSelect => Key::MediaSelect,
        KeyCode::Sleep => Key::Sleep,
        KeyCode::Turbo => Key::Turbo,
        KeyCode::Abort => Key::Abort,
        KeyCode::Resume => Key::Resume,
        KeyCode::Suspend => Key::Suspend,
        KeyCode::LaunchMail => Key::LaunchMail,
        KeyCode::MediaPlayPause => Key::MediaPlayPause,
        KeyCode::MediaStop => Key::MediaStop,
        KeyCode::MediaTrackNext => Key::MediaTrackNext,
        KeyCode::MediaTrackPrevious => Key::MediaTrackPrevious,
        KeyCode::Power => Key::Power,
        KeyCode::AudioVolumeDown => Key::AudioVolumeDown,
        KeyCode::AudioVolumeMute => Key::AudioVolumeMute,
        KeyCode::AudioVolumeUp => Key::AudioVolumeUp,
        KeyCode::WakeUp => Key::WakeUp,
        KeyCode::Meta => Key::Meta,
        KeyCode::Hyper => Key::Hyper,
        KeyCode::Again => Key::Again,
        KeyCode::Copy => Key::Copy,
        KeyCode::Cut => Key::Cut,
        KeyCode::Find => Key::Find,
        KeyCode::Open => Key::Open,
        KeyCode::Paste => Key::Paste,
        KeyCode::Props => Key::Props,
        KeyCode::Select => Key::Select,
        KeyCode::Undo => Key::Undo,
        KeyCode::Hiragana => Key::Hiragana,
        KeyCode::Katakana => Key::Katakana,
        KeyCode::F1 => Key::F1,
        KeyCode::F2 => Key::F2,
        KeyCode::F3 => Key::F3,
        KeyCode::F4 => Key::F4,
        KeyCode::F5 => Key::F5,
        KeyCode::F6 => Key::F6,
        KeyCode::F7 => Key::F7,
        KeyCode::F8 => Key::F8,
        KeyCode::F9 => Key::F9,
        KeyCode::F10 => Key::F10,
        KeyCode::F11 => Key::F11,
        KeyCode::F12 => Key::F12,
        KeyCode::F13 => Key::F13,
        KeyCode::F14 => Key::F14,
        KeyCode::F15 => Key::F15,
        KeyCode::F16 => Key::F16,
        KeyCode::F17 => Key::F17,
        KeyCode::F18 => Key::F18,
        KeyCode::F19 => Key::F19,
        KeyCode::F20 => Key::F20,
        KeyCode::F21 => Key::F21,
        KeyCode::F22 => Key::F22,
        KeyCode::F23 => Key::F23,
        KeyCode::F24 => Key::F24,
        KeyCode::F25 => Key::F25,
        KeyCode::F26 => Key::F26,
        KeyCode::F27 => Key::F27,
        KeyCode::F28 => Key::F28,
        KeyCode::F29 => Key::F29,
        KeyCode::F30 => Key::F30,
        KeyCode::F31 => Key::F31,
        KeyCode::F32 => Key::F32,
        KeyCode::F33 => Key::F33,
        KeyCode::F34 => Key::F34,
        KeyCode::F35 => Key::F35,
        KeyCode::Escape => Key::Escape,
        KeyCode::Fn => Key::Fn,
        KeyCode::FnLock => Key::FnLock,
        KeyCode::PrintScreen => Key::PrintScreen,
        KeyCode::ScrollLock => Key::ScrollLock,
        KeyCode::Pause => Key::Pause,
        KeyCode::BrowserBack => Key::BrowserBack,
        KeyCode::BrowserFavorites => Key::BrowserFavorites,
        KeyCode::BrowserForward => Key::BrowserForward,
        KeyCode::BrowserHome => Key::BrowserHome,
        KeyCode::BrowserRefresh => Key::BrowserRefresh,
        KeyCode::BrowserSearch => Key::BrowserSearch,
        KeyCode::BrowserStop => Key::BrowserStop,
        KeyCode::Eject => Key::Eject,
        KeyCode::NonConvert => Key::NonConvert,
        KeyCode::Delete => Key::Delete,
        KeyCode::End => Key::End,
        KeyCode::Help => Key::Help,
        KeyCode::Home => Key::Home,
        KeyCode::Insert => Key::Insert,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::ArrowDown => Key::ArrowDown,
        KeyCode::ArrowLeft => Key::ArrowLeft,
        KeyCode::ArrowRight => Key::ArrowRight,
        KeyCode::ArrowUp => Key::ArrowUp,
        KeyCode::NumLock => Key::NumLock,
        KeyCode::Space => Key::Space,
        KeyCode::Tab => Key::Tab,
        KeyCode::Convert => Key::Convert,
        KeyCode::KanaMode => Key::KanaMode,
        KeyCode::Enter => Key::Enter,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::CapsLock => Key::CapsLock,
        KeyCode::ContextMenu => Key::ContextMenu,
        _ => todo!(),
    }
}
