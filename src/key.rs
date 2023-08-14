use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Space,

    // Arrows
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    // Modifier keys
    Escape,
    Enter,
    Backspace,
    Tab,
    CapsLock,
    Shift,
    Alt,
    Control,
    Meta,
    ContextMenu,
}

impl Key {
    pub fn is_digit(&self) -> bool {
        matches!(
            self,
            Self::Zero
                | Self::One
                | Self::Two
                | Self::Three
                | Self::Four
                | Self::Five
                | Self::Six
                | Self::Seven
                | Self::Eight
                | Self::Nine
        )
    }

    pub fn is_arrow(&self) -> bool {
        matches!(
            self,
            Self::ArrowUp | Self::ArrowDown | Self::ArrowLeft | Self::ArrowRight
        )
    }

    pub fn is_modifier(&self) -> bool {
        matches!(self, Self::Shift | Self::Control | Self::Alt | Self::Meta)
    }
}

impl From<Key> for u8 {
    fn from(value: Key) -> Self {
        match value {
            Key::Zero => 0,
            Key::One => 1,
            Key::Two => 2,
            Key::Three => 3,
            Key::Four => 4,
            Key::Five => 5,
            Key::Six => 6,
            Key::Seven => 7,
            Key::Eight => 8,
            Key::Nine => 9,
            _ => panic!("Could not convert non-digit Key into u8."),
        }
    }
}

#[derive(Error, Debug)]
#[error("Invalid key value: {0}")]
pub struct KeyParseError(String);

impl TryFrom<String> for Key {
    type Error = KeyParseError;

    /// Converts from the value of KeyboardEvent.key (see MDN docs)
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_ref() {
            "a" | "A" => Ok(Self::A),
            "b" | "B" => Ok(Self::B),
            "c" | "C" => Ok(Self::C),
            "d" | "D" => Ok(Self::D),
            "e" | "E" => Ok(Self::E),
            "f" | "F" => Ok(Self::F),
            "g" | "G" => Ok(Self::G),
            "h" | "H" => Ok(Self::H),
            "i" | "I" => Ok(Self::I),
            "j" | "J" => Ok(Self::J),
            "k" | "K" => Ok(Self::K),
            "l" | "L" => Ok(Self::L),
            "m" | "M" => Ok(Self::M),
            "n" | "N" => Ok(Self::N),
            "o" | "O" => Ok(Self::O),
            "p" | "P" => Ok(Self::P),
            "q" | "Q" => Ok(Self::Q),
            "r" | "R" => Ok(Self::R),
            "s" | "S" => Ok(Self::S),
            "t" | "T" => Ok(Self::T),
            "u" | "U" => Ok(Self::U),
            "v" | "V" => Ok(Self::V),
            "w" | "W" => Ok(Self::W),
            "x" | "X" => Ok(Self::X),
            "y" | "Y" => Ok(Self::Y),
            "z" | "Z" => Ok(Self::Z),
            "0" => Ok(Self::Zero),
            "1" => Ok(Self::One),
            "2" => Ok(Self::Two),
            "3" => Ok(Self::Three),
            "4" => Ok(Self::Four),
            "5" => Ok(Self::Five),
            "6" => Ok(Self::Six),
            "7" => Ok(Self::Seven),
            "8" => Ok(Self::Eight),
            "9" => Ok(Self::Nine),
            " " | "Space" => Ok(Self::Space),
            "ArrowUp" => Ok(Self::ArrowUp),
            "ArrowDown" => Ok(Self::ArrowDown),
            "ArrowLeft" => Ok(Self::ArrowLeft),
            "ArrowRight" => Ok(Self::ArrowRight),
            "Escape" => Ok(Self::Escape),
            "Enter" => Ok(Self::Enter),
            "Backspace" => Ok(Self::Backspace),
            "Tab" => Ok(Self::Tab),
            "CapsLock" => Ok(Self::CapsLock),
            "Shift" => Ok(Self::Shift),
            "Alt" => Ok(Self::Alt),
            "Control" => Ok(Self::Control),
            "Meta" => Ok(Self::Meta),
            "ContextMenu" => Ok(Self::ContextMenu),
            _ => Err(KeyParseError(value)),
        }
    }
}
