#[derive(Clone, PartialEq)]
pub enum TileType {
    Empty,
    Obstacle,
    Energy,
    Mineral,
    Scientific,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RobotType {
    Explorer,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RobotMode {
    Exploring,
    Idle,
}

pub const MAP_SIZE: usize = 20;