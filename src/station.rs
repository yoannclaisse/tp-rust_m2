use crate::types::{TileType, RobotType, MAP_SIZE};
use crate::map::Map;
use crate::robot::Robot;

#[derive(Clone)]
pub struct TerrainData {
    pub explored: bool,
    pub timestamp: u32,
    pub robot_id: usize,
    pub robot_type: RobotType,
}

pub struct Station {
    pub energy_reserves: u32,
    pub collected_minerals: u32,
    pub collected_scientific_data: u32,
    pub global_memory: Vec<Vec<TerrainData>>,
    pub conflict_count: usize,
    pub next_robot_id: usize,
    pub current_time: u32,
}

impl Station {
    pub fn new() -> Self {
        let mut global_memory = Vec::with_capacity(MAP_SIZE);
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
            global_memory.push(row);
        }
        
        Self {
            energy_reserves: 100,
            collected_minerals: 0,
            collected_scientific_data: 0,
            global_memory,
            conflict_count: 0,
            next_robot_id: 1,
            current_time: 0,
        }
    }
    
    pub fn tick(&mut self) {
        self.current_time += 1;
    }
    
    pub fn share_knowledge(&mut self, robot: &mut Robot) {
        if robot.x == robot.home_station_x && robot.y == robot.home_station_y {
            let mut conflicts = 0;
            
            // Robot shares knowledge with station
            for y in 0..MAP_SIZE {
                for x in 0..MAP_SIZE {
                    if robot.memory[y][x].explored {
                        if self.global_memory[y][x].explored {
                            // Conflict resolution: use most recent data
                            if robot.memory[y][x].timestamp > self.global_memory[y][x].timestamp {
                                self.global_memory[y][x] = robot.memory[y][x].clone();
                                conflicts += 1;
                            }
                        } else {
                            // No conflict, add robot's knowledge
                            self.global_memory[y][x] = robot.memory[y][x].clone();
                        }
                    }
                }
            }
            
            // Robot receives global knowledge
            for y in 0..MAP_SIZE {
                for x in 0..MAP_SIZE {
                    if self.global_memory[y][x].explored {
                        robot.memory[y][x] = self.global_memory[y][x].clone();
                    }
                }
            }
            
            self.conflict_count += conflicts;
            
            if conflicts > 0 {
                println!("Robot {} synchronized knowledge. Conflicts resolved: {}", robot.id, conflicts);
            }
        }
    }
    
    pub fn deposit_resources(&mut self, minerals: u32, scientific_data: u32) {
        self.collected_minerals += minerals;
        self.collected_scientific_data += scientific_data;
        self.energy_reserves += minerals;
    }
    
    pub fn get_status(&self) -> String {
        let status = match (self.energy_reserves, self.collected_minerals) {
            (e, m) if e < 30 => "Faible en énergie",
            (e, m) if m < 10 => "Faible en minerais",
            (e, m) if e >= 200 && m >= 50 => "Ressources abondantes",
            _ => "Ressources adéquates",
        };
        
        format!("{} | Énergie: {} | Minerais: {} | Données: {} | Conflits: {}", 
                status, self.energy_reserves, self.collected_minerals, 
                self.collected_scientific_data, self.conflict_count)
    }
    
    pub fn get_exploration_percentage(&self) -> f32 {
        let mut explored_count = 0;
        
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                if self.global_memory[y][x].explored {
                    explored_count += 1;
                }
            }
        }
        
        (explored_count as f32 / (MAP_SIZE * MAP_SIZE) as f32) * 100.0
    }
}