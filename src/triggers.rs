use std::convert::AsRef;

use toml;
use num::NumCast;
use winit::event::VirtualKeyCode;

use super::MouseWheelDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireTrigger {
    Holdable(HoldableTrigger),
    MouseWheelTick(MouseWheelDirection),
}

impl FireTrigger {
    pub fn from_toml(value: &toml::value::Value) -> Result<FireTrigger, String> {
        use toml::value::Value::*;
        use self::FireTrigger::*;
        use self::MouseWheelDirection::*;

        if let Ok(switch_trigger) = HoldableTrigger::from_toml(value) {
            Ok(Holdable(switch_trigger))
        } else {
            match value {
                &String(ref s) => match s.as_ref() {
                    "MouseWheelUp" => Ok(MouseWheelTick(Up)),
                    "MouseWheelDown" => Ok(MouseWheelTick(Down)),
                    _ => Err(format!("Unknown fire trigger: '{}'", s)),
                }
                _ => Err(format!("Fire trigger must be string, got '{}'!", value)),
            }
        }
    }

    pub fn to_toml(&self) -> toml::value::Value {
        use self::FireTrigger::*;
        use super::MouseWheelDirection::*;

        match self {
            &Holdable(trigger) => trigger.to_toml(),
            &MouseWheelTick(Up) => toml::value::Value::String(String::from("MouseWheelUp")),
            &MouseWheelTick(Down) => toml::value::Value::String(String::from("MouseWheelDown")),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum HoldableTrigger {
    ScanCode(u32),
    KeyCode(VirtualKeyCode),
    Button(u32),
}

impl HoldableTrigger {
    pub fn from_toml(value: &toml::value::Value) -> Result<HoldableTrigger, String> {
        use toml::value::Value::*;
        use self::HoldableTrigger::*;

        match value {
            &Integer(i) => match NumCast::from(i) {
                Some(sc) => Ok(ScanCode(sc)),
                None => return Err(format!("Invalid scan code: {}", i)),
            },
            &String(ref s) => {
                match AsRef::<str>::as_ref(s) {
                    // TODO re-add mouse
                    //"MouseLeft" => Ok(Button(0)),
                    //"MouseRight" => Ok(Button(1)),
                    //"MouseMiddle" => Ok(Button(2)),
                    ss => {
                        if ss.starts_with("Button") {
                            match ss[5..].parse() {
                                Ok(number) => Ok(Button(number)),
                                Err(_) => Err(format!("Unknown push button {}", s)),
                            }
                        } else {
                            for &(kc, name) in KEY_CODE_PAIRS {
                                if name == ss {
                                    return Ok(KeyCode(kc));
                                }
                            }
                            Err(format!("Unknown push button {}", s))
                        }
                    }
                }
            }
            _ => Err(format!("Unknown push button {}", *value))
        }
    }

    pub fn to_toml(&self) -> toml::value::Value {
        use self::HoldableTrigger::*;

        match *self {
            ScanCode(sc) => toml::value::Value::Integer(sc as i64),
            KeyCode(kc) => {
                for &(key_code, name) in KEY_CODE_PAIRS {
                    if key_code == kc {
                        return toml::value::Value::String(String::from(name));
                    }
                }
                toml::value::Value::String(String::new()) // should not happen
            },
            // TODO re-add mouse
            //Button(0) => toml::value::Value::String(String::from("MouseLeft")),
            //Button(1) => toml::value::Value::String(String::from("MouseRight")),
            //Button(2) => toml::value::Value::String(String::from("MouseMiddle")),
            Button(number) => toml::value::Value::String(format!("Button{}", number)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueTrigger {
    MouseX,
    MouseY,
    MouseWheel,
    Axis(u32),
}

impl ValueTrigger {
    pub fn from_toml(value: &toml::value::Value) -> Result<ValueTrigger, String> {
        use toml::Value::*;
        use self::ValueTrigger::*;

        match value {
            &Integer(i) => match NumCast::from(i) {
                Some(axis) => Ok(Axis(axis)),
                None => return Err(format!("Invalid axis id: {}", i)),
            },
            &String(ref s) => match s.as_ref() {
                "MouseWheel" => Ok(MouseWheel),
                _ => Err(format!("Unknown axis: '{}'", s)),
            }
            v => Err(format!("'axis' must be integer or string, got '{}'!", v)),
        }
    }

    pub fn to_toml(&self) -> toml::value::Value {
        use self::ValueTrigger::*;

        match *self {
            MouseX => toml::value::Value::String(String::from("MouseX")),
            MouseY => toml::value::Value::String(String::from("MouseY")),
            MouseWheel => toml::value::Value::String(String::from("MouseWheel")),
            Axis(a) => toml::value::Value::Integer(a as i64),
        }
    }
}

const KEY_CODE_PAIRS: &'static [(VirtualKeyCode, &'static str)] = &[
    (VirtualKeyCode::Key1, "1"),
    (VirtualKeyCode::Key2, "2"),
    (VirtualKeyCode::Key3, "3"),
    (VirtualKeyCode::Key4, "4"),
    (VirtualKeyCode::Key5, "5"),
    (VirtualKeyCode::Key6, "6"),
    (VirtualKeyCode::Key7, "7"),
    (VirtualKeyCode::Key8, "8"),
    (VirtualKeyCode::Key9, "9"),
    (VirtualKeyCode::Key0, "0"),
    (VirtualKeyCode::A, "A"),
    (VirtualKeyCode::B, "B"),
    (VirtualKeyCode::C, "C"),
    (VirtualKeyCode::D, "D"),
    (VirtualKeyCode::E, "E"),
    (VirtualKeyCode::F, "F"),
    (VirtualKeyCode::G, "G"),
    (VirtualKeyCode::H, "H"),
    (VirtualKeyCode::I, "I"),
    (VirtualKeyCode::J, "J"),
    (VirtualKeyCode::K, "K"),
    (VirtualKeyCode::L, "L"),
    (VirtualKeyCode::M, "M"),
    (VirtualKeyCode::N, "N"),
    (VirtualKeyCode::O, "O"),
    (VirtualKeyCode::P, "P"),
    (VirtualKeyCode::Q, "Q"),
    (VirtualKeyCode::R, "R"),
    (VirtualKeyCode::S, "S"),
    (VirtualKeyCode::T, "T"),
    (VirtualKeyCode::U, "U"),
    (VirtualKeyCode::V, "V"),
    (VirtualKeyCode::W, "W"),
    (VirtualKeyCode::X, "X"),
    (VirtualKeyCode::Y, "Y"),
    (VirtualKeyCode::Z, "Z"),
    (VirtualKeyCode::Escape, "Escape"),
    (VirtualKeyCode::F1, "F1"),
    (VirtualKeyCode::F2, "F2"),
    (VirtualKeyCode::F3, "F3"),
    (VirtualKeyCode::F4, "F4"),
    (VirtualKeyCode::F5, "F5"),
    (VirtualKeyCode::F6, "F6"),
    (VirtualKeyCode::F7, "F7"),
    (VirtualKeyCode::F8, "F8"),
    (VirtualKeyCode::F9, "F9"),
    (VirtualKeyCode::F10, "F10"),
    (VirtualKeyCode::F11, "F11"),
    (VirtualKeyCode::F12, "F12"),
    (VirtualKeyCode::F13, "F13"),
    (VirtualKeyCode::F14, "F14"),
    (VirtualKeyCode::F15, "F15"),
    (VirtualKeyCode::F16, "F16"),
    (VirtualKeyCode::F17, "F17"),
    (VirtualKeyCode::F18, "F18"),
    (VirtualKeyCode::F19, "F19"),
    (VirtualKeyCode::F20, "F20"),
    (VirtualKeyCode::F21, "F21"),
    (VirtualKeyCode::F22, "F22"),
    (VirtualKeyCode::F23, "F23"),
    (VirtualKeyCode::F24, "F24"),
    (VirtualKeyCode::Snapshot, "Snapshot"),
    (VirtualKeyCode::Scroll, "Scroll"),
    (VirtualKeyCode::Pause, "Pause"),
    (VirtualKeyCode::Insert, "Insert"),
    (VirtualKeyCode::Home, "Home"),
    (VirtualKeyCode::Delete, "Delete"),
    (VirtualKeyCode::End, "End"),
    (VirtualKeyCode::PageDown, "PageDown"),
    (VirtualKeyCode::PageUp, "PageUp"),
    (VirtualKeyCode::Left, "Left"),
    (VirtualKeyCode::Up, "Up"),
    (VirtualKeyCode::Right, "Right"),
    (VirtualKeyCode::Down, "Down"),
    (VirtualKeyCode::Back, "Back"),
    (VirtualKeyCode::Return, "Return"),
    (VirtualKeyCode::Space, "Space"),
    (VirtualKeyCode::Compose, "Compose"),
    (VirtualKeyCode::Caret, "Caret"),
    (VirtualKeyCode::Numlock, "Numlock"),
    (VirtualKeyCode::Numpad0, "Numpad0"),
    (VirtualKeyCode::Numpad1, "Numpad1"),
    (VirtualKeyCode::Numpad2, "Numpad2"),
    (VirtualKeyCode::Numpad3, "Numpad3"),
    (VirtualKeyCode::Numpad4, "Numpad4"),
    (VirtualKeyCode::Numpad5, "Numpad5"),
    (VirtualKeyCode::Numpad6, "Numpad6"),
    (VirtualKeyCode::Numpad7, "Numpad7"),
    (VirtualKeyCode::Numpad8, "Numpad8"),
    (VirtualKeyCode::Numpad9, "Numpad9"),
    (VirtualKeyCode::AbntC1, "AbntC1"),
    (VirtualKeyCode::AbntC2, "AbntC2"),
    (VirtualKeyCode::Add, "Add"),
    (VirtualKeyCode::Apostrophe, "Apostrophe"),
    (VirtualKeyCode::Apps, "Apps"),
    (VirtualKeyCode::At, "At"),
    (VirtualKeyCode::Ax, "Ax"),
    (VirtualKeyCode::Backslash, "Backslash"),
    (VirtualKeyCode::Calculator, "Calculator"),
    (VirtualKeyCode::Capital, "Capital"),
    (VirtualKeyCode::Colon, "Colon"),
    (VirtualKeyCode::Comma, "Comma"),
    (VirtualKeyCode::Convert, "Convert"),
    (VirtualKeyCode::Decimal, "Decimal"),
    (VirtualKeyCode::Divide, "Divide"),
    (VirtualKeyCode::Equals, "Equals"),
    (VirtualKeyCode::Grave, "Grave"),
    (VirtualKeyCode::Kana, "Kana"),
    (VirtualKeyCode::Kanji, "Kanji"),
    (VirtualKeyCode::LAlt, "LAlt"),
    (VirtualKeyCode::LBracket, "LBracket"),
    (VirtualKeyCode::LControl, "LControl"),
    (VirtualKeyCode::LShift, "LShift"),
    (VirtualKeyCode::LWin, "LWin"),
    (VirtualKeyCode::Mail, "Mail"),
    (VirtualKeyCode::MediaSelect, "MediaSelect"),
    (VirtualKeyCode::MediaStop, "MediaStop"),
    (VirtualKeyCode::Minus, "Minus"),
    (VirtualKeyCode::Multiply, "Multiply"),
    (VirtualKeyCode::Mute, "Mute"),
    (VirtualKeyCode::MyComputer, "MyComputer"),
    (VirtualKeyCode::NavigateForward, "NavigateForward"),
    (VirtualKeyCode::NavigateBackward, "NavigateBackward"),
    (VirtualKeyCode::NextTrack, "NextTrack"),
    (VirtualKeyCode::NoConvert, "NoConvert"),
    (VirtualKeyCode::NumpadComma, "NumpadComma"),
    (VirtualKeyCode::NumpadEnter, "NumpadEnter"),
    (VirtualKeyCode::NumpadEquals, "NumpadEquals"),
    (VirtualKeyCode::OEM102, "OEM102"),
    (VirtualKeyCode::Period, "Period"),
    (VirtualKeyCode::PlayPause, "PlayPause"),
    (VirtualKeyCode::Power, "Power"),
    (VirtualKeyCode::PrevTrack, "PrevTrack"),
    (VirtualKeyCode::RAlt, "RAlt"),
    (VirtualKeyCode::RBracket, "RBracket"),
    (VirtualKeyCode::RControl, "RControl"),
    (VirtualKeyCode::RShift, "RShift"),
    (VirtualKeyCode::RWin, "RWin"),
    (VirtualKeyCode::Semicolon, "Semicolon"),
    (VirtualKeyCode::Slash, "Slash"),
    (VirtualKeyCode::Sleep, "Sleep"),
    (VirtualKeyCode::Stop, "Stop"),
    (VirtualKeyCode::Subtract, "Subtract"),
    (VirtualKeyCode::Sysrq, "Sysrq"),
    (VirtualKeyCode::Tab, "Tab"),
    (VirtualKeyCode::Underline, "Underline"),
    (VirtualKeyCode::Unlabeled, "Unlabeled"),
    (VirtualKeyCode::VolumeDown, "VolumeDown"),
    (VirtualKeyCode::VolumeUp, "VolumeUp"),
    (VirtualKeyCode::Wake, "Wake"),
    (VirtualKeyCode::WebBack, "WebBack"),
    (VirtualKeyCode::WebFavorites, "WebFavorites"),
    (VirtualKeyCode::WebForward, "WebForward"),
    (VirtualKeyCode::WebHome, "WebHome"),
    (VirtualKeyCode::WebRefresh, "WebRefresh"),
    (VirtualKeyCode::WebSearch, "WebSearch"),
    (VirtualKeyCode::WebStop, "WebStop"),
    (VirtualKeyCode::Yen, "Yen"),
    (VirtualKeyCode::Copy, "Copy"),
    (VirtualKeyCode::Paste, "Paste"),
    (VirtualKeyCode::Cut, "Cut"),
];