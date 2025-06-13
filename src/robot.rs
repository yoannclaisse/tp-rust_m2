use crate::types::{MAP_SIZE, TileType, RobotType, RobotMode};
use crate::map::Map;
use crate::station::{Station, TerrainData};
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
    pub memory: Vec<Vec<TerrainData>>,
    pub target: Option<(usize, usize)>,
    pub id: usize,
    pub home_station_x: usize,
    pub home_station_y: usize,
    pub last_sync_time: u32,
}

impl Robot {
    pub fn new(x: usize, y: usize, robot_type: RobotType) -> Self {
        let (max_energy, energy) = match robot_type {
            RobotType::Explorer => (80.0, 80.0),
            RobotType::EnergyCollector => (120.0, 120.0),
            RobotType::MineralCollector => (100.0, 100.0),
            RobotType::ScientificCollector => (60.0, 60.0),
        };
        
        let mut memory = Vec::with_capacity(MAP_SIZE);
        for _ in 0..MAP_SIZE {
            let row = vec![
                TerrainData {
                    explored: false,
                    timestamp: 0,
                    robot_id: 0,
                    robot_type: RobotType::Explorer,
                }; 
                MAP_SIZE
            ];
            memory.push(row);
        }
        
        Self {
            x: x.clamp(0, MAP_SIZE - 1),
            y: y.clamp(0, MAP_SIZE - 1),
            energy,
            max_energy,
            minerals: 0,
            scientific_data: 0,
            robot_type,
            mode: RobotMode::Exploring,
            memory,
            target: None,
            id: 0,
            home_station_x: x.clamp(0, MAP_SIZE - 1),
            home_station_y: y.clamp(0, MAP_SIZE - 1),
            last_sync_time: 0,
        }
    }
    
    pub fn new_with_memory(
        x: usize, 
        y: usize, 
        robot_type: RobotType, 
        id: usize,
        station_x: usize,
        station_y: usize,
        memory: Vec<Vec<TerrainData>>
    ) -> Self {
        let (max_energy, energy) = match robot_type {
            RobotType::Explorer => (80.0, 80.0),
            RobotType::EnergyCollector => (120.0, 120.0),
            RobotType::MineralCollector => (100.0, 100.0),
            RobotType::ScientificCollector => (60.0, 60.0),
        };
        
        Self {
            x: x.clamp(0, MAP_SIZE - 1),
            y: y.clamp(0, MAP_SIZE - 1),
            energy,
            max_energy,
            minerals: 0,
            scientific_data: 0,
            robot_type,
            mode: RobotMode::Exploring,
            memory,
            target: None,
            id,
            home_station_x: station_x.clamp(0, MAP_SIZE - 1),
            home_station_y: station_y.clamp(0, MAP_SIZE - 1),
            last_sync_time: 0,
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
    
    // Ensure robot position is always within bounds
    fn clamp_position(&mut self) {
        self.x = self.x.clamp(0, MAP_SIZE - 1);
        self.y = self.y.clamp(0, MAP_SIZE - 1);
    }
    
    pub fn update(&mut self, map: &mut Map, station: &mut Station) {
        // Ensure we're within bounds at start of update
        self.clamp_position();
        
        self.energy -= 0.1;
        self.update_memory(station);
        
        if self.should_return_to_station() {
            self.mode = RobotMode::ReturnToStation;
            self.target = Some((self.home_station_x, self.home_station_y));
        }
        
        if self.x == self.home_station_x && self.y == self.home_station_y {
            // Recharge and deposit resources
            self.energy = self.max_energy;
            station.deposit_resources(self.minerals, self.scientific_data);
            self.minerals = 0;
            self.scientific_data = 0;
            
            // Synchronize knowledge
            if station.current_time > self.last_sync_time {
                station.share_knowledge(self);
                self.last_sync_time = station.current_time;
            }
            
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
                    if let Some(resource_pos) = self.find_nearest_resource(map) {
                        self.target = Some(resource_pos);
                    } else {
                        self.mode = RobotMode::Exploring;
                    }
                } else if let Some(target) = self.target {
                    self.move_towards_target(map);
                }
            },
            RobotMode::ReturnToStation => {
                if let Some(target) = self.target {
                    self.move_towards_target(map);
                } else {
                    self.mode = RobotMode::Idle;
                }
            },
            RobotMode::Idle => {
                if self.robot_type == RobotType::Explorer {
                    self.mode = RobotMode::Exploring;
                }
            }
        }
        
        // Final bounds check after all operations
        self.clamp_position();
    }
    
    fn update_memory(&mut self, station: &Station) {
        // Ensure current position is valid before updating memory
        if self.x < MAP_SIZE && self.y < MAP_SIZE {
            self.memory[self.y][self.x] = TerrainData {
                explored: true,
                timestamp: station.current_time,
                robot_id: self.id,
                robot_type: self.robot_type,
            };
        }
        
        let vision_range = match self.robot_type {
            RobotType::Explorer => 3,
            _ => 2,
        };
        
        for dy in -vision_range..=vision_range {
            for dx in -vision_range..=vision_range {
                let nx = self.x as isize + dx;
                let ny = self.y as isize + dy;
                
                if nx >= 0 && nx < MAP_SIZE as isize && ny >= 0 && ny < MAP_SIZE as isize {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    
                    if !self.memory[ny][nx].explored || 
                       self.memory[ny][nx].timestamp < station.current_time {
                        
                        self.memory[ny][nx] = TerrainData {
                            explored: true,
                            timestamp: station.current_time,
                            robot_id: self.id,
                            robot_type: self.robot_type,
                        };
                    }
                }
            }
        }
    }
    
    pub fn get_exploration_percentage(&self) -> f32 {
        let mut explored_count = 0;
        
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                if self.memory[y][x].explored {
                    explored_count += 1;
                }
            }
        }
        
        (explored_count as f32 / (MAP_SIZE * MAP_SIZE) as f32) * 100.0
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
    
    fn find_exploration_target(&self) -> Option<(usize, usize)> {
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                if !self.memory[y][x].explored {
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
                // Double-check that the next position is valid and within bounds
                if next_pos.0 < MAP_SIZE && next_pos.1 < MAP_SIZE && 
                   map.is_valid_position(next_pos.0, next_pos.1) {
                    
                    self.x = next_pos.0;
                    self.y = next_pos.1;
                    
                    let energy_cost = match self.robot_type {
                        RobotType::Explorer => 0.3,
                        RobotType::EnergyCollector => 0.4,
                        RobotType::MineralCollector => 0.5,
                        RobotType::ScientificCollector => 0.6,
                    };
                    
                    self.energy -= energy_cost;
                } else {
                    // If movement is invalid, clear target and look for new one
                    self.target = None;
                }
            }
        }
        
        // Ensure position is still valid after movement
        self.clamp_position();
    }
    
    fn find_path(&self, map: &Map, target: (usize, usize)) -> VecDeque<(usize, usize)> {
        let start = (self.x, self.y);
        
        // Validate target is within bounds
        if target.0 >= MAP_SIZE || target.1 >= MAP_SIZE {
            return VecDeque::new();
        }
        
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
                    if let Some(&prev) = came_from.get(&current) {
                        current = prev;
                    } else {
                        break;
                    }
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
                    
                    // Strict bounds checking
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