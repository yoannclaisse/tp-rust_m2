use crate::types::{MAP_SIZE, TileType, RobotType, RobotMode};
use crate::map::Map;
use rand::prelude::*;
use std::collections::{VecDeque, BinaryHeap, HashMap};
use std::cmp::Ordering;

#[derive(Clone, Eq, PartialEq)]
struct Node {
    position: (usize, usize),
    g_cost: usize,
    f_cost: usize,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_cost.cmp(&self.f_cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Robot {
    pub x: usize,
    pub y: usize,
    pub energy: f32,
    pub max_energy: f32,
    pub minerals: u32,
    pub scientific_data: u32,
    pub robot_type: RobotType,
    pub mode: RobotMode,
    pub memory: Vec<Vec<bool>>,
    pub target: Option<(usize, usize)>,
}

impl Robot {
    pub fn new(x: usize, y: usize, robot_type: RobotType) -> Self {
        let (max_energy, energy) = match robot_type {
            RobotType::Explorer => (80.0, 80.0),
            RobotType::EnergyCollector => (120.0, 120.0),
            RobotType::MineralCollector => (100.0, 100.0),
            RobotType::ScientificCollector => (60.0, 60.0),
        };
        
        Self {
            x,
            y,
            energy,
            max_energy,
            minerals: 0,
            scientific_data: 0,
            robot_type,
            mode: RobotMode::Exploring,
            memory: vec![vec![false; MAP_SIZE]; MAP_SIZE],
            target: None,
        }
    }
    
    pub fn get_display_char(&self) -> &str {
        match self.robot_type {
            RobotType::Explorer => "E",
            RobotType::EnergyCollector => "P",
            RobotType::MineralCollector => "M",
            RobotType::ScientificCollector => "S",
        }
    }
    
    pub fn get_display_color(&self) -> u8 {
        match self.robot_type {
            RobotType::Explorer => 9,
            RobotType::EnergyCollector => 10,
            RobotType::MineralCollector => 13,
            RobotType::ScientificCollector => 12,
        }
    }
    
    pub fn update(&mut self, map: &mut Map) {
        self.energy -= 0.1;
        self.update_memory();
        
        if self.should_return_to_station() {
            self.mode = RobotMode::ReturnToStation;
            self.target = Some((map.station_x, map.station_y));
        }
        
        if self.x == map.station_x && self.y == map.station_y {
            self.energy = self.max_energy;
            self.minerals = 0;
            self.scientific_data = 0;
            self.mode = RobotMode::Exploring;
        }
        
        match self.mode {
            RobotMode::Exploring => {
                if let Some(resource_pos) = self.find_nearest_resource(map) {
                    self.target = Some(resource_pos);
                    self.mode = RobotMode::Collecting;
                } else {
                    self.target = self.find_exploration_target();
                }
                
                if let Some(target) = self.target {
                    self.move_towards_target(map);
                }
            },
            RobotMode::Collecting => {
                let tile = map.get_tile(self.x, self.y);
                if self.can_collect(tile) {
                    self.collect_resource(map);
                    self.mode = RobotMode::Exploring;
                } else if let Some(target) = self.target {
                    self.move_towards_target(map);
                }
            },
            RobotMode::ReturnToStation => {
                if let Some(target) = self.target {
                    self.move_towards_target(map);
                }
            },
            RobotMode::Idle => {
                self.mode = RobotMode::Exploring;
            }
        }
    }
    
    fn should_return_to_station(&self) -> bool {
        if self.energy < self.max_energy * 0.3 {
            return true;
        }
        
        match self.robot_type {
            RobotType::MineralCollector => self.minerals >= 5,
            RobotType::ScientificCollector => self.scientific_data >= 3,
            _ => false
        }
    }
    
    fn find_nearest_resource(&self, map: &Map) -> Option<(usize, usize)> {
        let target_resource = match self.robot_type {
            RobotType::Explorer => return None,
            RobotType::EnergyCollector => TileType::Energy,
            RobotType::MineralCollector => TileType::Mineral,
            RobotType::ScientificCollector => TileType::Scientific,
        };
        
        let mut nearest = None;
        let mut min_distance = usize::MAX;
        
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                if map.get_tile(x, y) == target_resource {
                    let distance = self.heuristic((self.x, self.y), (x, y));
                    if distance < min_distance {
                        min_distance = distance;
                        nearest = Some((x, y));
                    }
                }
            }
        }
        
        nearest
    }
    
    fn can_collect(&self, tile: TileType) -> bool {
        match (self.robot_type, tile) {
            (RobotType::EnergyCollector, TileType::Energy) => true,
            (RobotType::MineralCollector, TileType::Mineral) => true,
            (RobotType::ScientificCollector, TileType::Scientific) => true,
            _ => false,
        }
    }
    
    fn collect_resource(&mut self, map: &mut Map) {
        let tile = map.get_tile(self.x, self.y);
        
        match (self.robot_type, tile) {
            (RobotType::EnergyCollector, TileType::Energy) => {
                self.energy = (self.energy + 20.0).min(self.max_energy);
                map.consume_resource(self.x, self.y);
            },
            (RobotType::MineralCollector, TileType::Mineral) => {
                self.minerals += 1;
                map.consume_resource(self.x, self.y);
            },
            (RobotType::ScientificCollector, TileType::Scientific) => {
                self.scientific_data += 1;
                map.consume_resource(self.x, self.y);
            },
            _ => {}
        }
    }
    
    fn update_memory(&mut self) {
        self.memory[self.y][self.x] = true;
        
        let vision_range = match self.robot_type {
            RobotType::Explorer => 3,
            _ => 2,
        };
        
        for dy in -vision_range..=vision_range {
            for dx in -vision_range..=vision_range {
                let nx = self.x as isize + dx;
                let ny = self.y as isize + dy;
                
                if nx >= 0 && nx < MAP_SIZE as isize && ny >= 0 && ny < MAP_SIZE as isize {
                    self.memory[ny as usize][nx as usize] = true;
                }
            }
        }
    }
    
    fn find_exploration_target(&self) -> Option<(usize, usize)> {
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                if !self.memory[y][x] {
                    return Some((x, y));
                }
            }
        }
        None
    }
    
    fn move_towards_target(&mut self, map: &Map) {
        if let Some(target) = self.target {
            let path = self.find_path(map, target);
            if let Some(&next_pos) = path.front() {
                self.x = next_pos.0;
                self.y = next_pos.1;
                
                let energy_cost = match self.robot_type {
                    RobotType::Explorer => 0.3,
                    RobotType::EnergyCollector => 0.4,
                    RobotType::MineralCollector => 0.5,
                    RobotType::ScientificCollector => 0.6,
                };
                
                self.energy -= energy_cost;
            }
        }
    }
    
    fn find_path(&self, map: &Map, target: (usize, usize)) -> VecDeque<(usize, usize)> {
        let start = (self.x, self.y);
        
        if start == target {
            return VecDeque::new();
        }
        
        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut g_score: HashMap<(usize, usize), usize> = HashMap::new();
        
        g_score.insert(start, 0);
        open_set.push(Node {
            position: start,
            g_cost: 0,
            f_cost: self.heuristic(start, target),
        });
        
        while let Some(current) = open_set.pop() {
            let current_pos = current.position;
            
            if current_pos == target {
                let mut path = VecDeque::new();
                let mut current = target;
                
                while current != start {
                    path.push_front(current);
                    current = *came_from.get(&current).unwrap();
                }
                
                return path;
            }
            
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    
                    let nx = current_pos.0 as isize + dx;
                    let ny = current_pos.1 as isize + dy;
                    
                    if nx < 0 || nx >= MAP_SIZE as isize || ny < 0 || ny >= MAP_SIZE as isize {
                        continue;
                    }
                    
                    let neighbor = (nx as usize, ny as usize);
                    
                    if !map.is_valid_position(neighbor.0, neighbor.1) {
                        continue;
                    }
                    
                    let tentative_g_score = g_score[&current_pos] + 1;
                    
                    if !g_score.contains_key(&neighbor) || tentative_g_score < g_score[&neighbor] {
                        came_from.insert(neighbor, current_pos);
                        g_score.insert(neighbor, tentative_g_score);
                        
                        let f_score = tentative_g_score + self.heuristic(neighbor, target);
                        open_set.push(Node {
                            position: neighbor,
                            g_cost: tentative_g_score,
                            f_cost: f_score,
                        });
                    }
                }
            }
        }
        
        VecDeque::new()
    }
    
    fn heuristic(&self, a: (usize, usize), b: (usize, usize)) -> usize {
        let dx = (a.0 as isize - b.0 as isize).abs() as usize;
        let dy = (a.1 as isize - b.1 as isize).abs() as usize;
        dx + dy
    }
}