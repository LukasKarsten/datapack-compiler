#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

impl ChatColor {
    pub fn from_string(s: &str) -> Option<Self> {
        let mut normalized;
        let mut s = s;
        if s.contains(|chr: char| !chr.is_ascii_lowercase()) {
            normalized = s.to_ascii_lowercase();
            normalized.retain(|chr| chr.is_ascii_lowercase());
            s = &normalized;
        }
        match s {
            "black" => Some(Self::Black),
            "darkblue" => Some(Self::DarkBlue),
            "darkgreen" => Some(Self::DarkGreen),
            "darkaqua" => Some(Self::DarkAqua),
            "darkred" => Some(Self::DarkRed),
            "darkpurple" => Some(Self::DarkPurple),
            "gold" => Some(Self::Gold),
            "gray" => Some(Self::Gray),
            "darkgray" => Some(Self::DarkGray),
            "blue" => Some(Self::Blue),
            "green" => Some(Self::Green),
            "aqua" => Some(Self::Aqua),
            "red" => Some(Self::Red),
            "lightpurple" => Some(Self::LightPurple),
            "yellow" => Some(Self::Yellow),
            "white" => Some(Self::White),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Black => "black",
            Self::DarkBlue => "dark_blue",
            Self::DarkGreen => "dark_green",
            Self::DarkAqua => "dark_aqua",
            Self::DarkRed => "dark_red",
            Self::DarkPurple => "dark_purple",
            Self::Gold => "gold",
            Self::Gray => "gray",
            Self::DarkGray => "dark_gray",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Aqua => "aqua",
            Self::Red => "red",
            Self::LightPurple => "light_purple",
            Self::Yellow => "yellow",
            Self::White => "white",
        }
    }
}
