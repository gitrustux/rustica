// Theme system

pub struct Theme {
    pub primary_color: (u8, u8, u8),
    pub secondary_color: (u8, u8, u8),
    pub background: (u8, u8, u8),
    pub text_color: (u8, u8, u8),
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            primary_color: (100, 150, 255),
            secondary_color: (60, 100, 180),
            background: (30, 30, 35),
            text_color: (220, 220, 220),
        }
    }

    pub fn light() -> Self {
        Self {
            primary_color: (50, 120, 200),
            secondary_color: (40, 100, 170),
            background: (250, 250, 250),
            text_color: (20, 20, 20),
        }
    }
}
