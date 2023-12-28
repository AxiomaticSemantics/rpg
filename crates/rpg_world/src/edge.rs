#[derive(Default, Debug)]
pub enum EdgeFlags {
    #[default]
    None = 0x00,
    Open = 0x01,
    Barrier = 0x01 << 1,
    Door = 0x01 << 2,
    Portal = 0x01 << 3,
}

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub enum Edge {
    #[default]
    Top = 0x00,
    Bottom = 0x01,
    Left = 0x02,
    Right = 0x03,
}

impl Edge {
    pub fn opposite(&self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}
