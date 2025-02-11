#[derive(Debug,Copy,Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub enum ManaType {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless
}

impl ManaType {
    pub const fn all() -> &'static [Self] {
        const ALL: &'static [ManaType] = &[
            ManaType::White,
            ManaType::Blue,
            ManaType::Black,
            ManaType::Red,
            ManaType::Green,
            ManaType::Colorless
        ];
        ALL
    }
}
