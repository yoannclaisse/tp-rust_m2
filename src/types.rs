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
    EnergyCollector,
    MineralCollector,
    ScientificCollector,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RobotMode {
    Exploring,
    Collecting,
    ReturnToStation,
    Idle,
}

pub const MAP_SIZE: usize = 20;