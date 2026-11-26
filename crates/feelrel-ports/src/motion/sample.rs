#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Motion {
    Significant,
    Moving,
    Stationary,
}
