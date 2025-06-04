use crate::types::{MAP_SIZE, RobotType, RobotMode};
use crate::map::Map;
use rand::prelude::*;

pub struct Robot {
    pub x: usize,
    pub y: usize,
    pub energy: f32,
    pub max_energy: f32,
    pub robot_type: RobotType,
    pub mode: RobotMode,
}

impl Robot {
    pub fn new(x: usize, y: usize, robot_type: RobotType) -> Self {
        Self {
            x,
            y,
            energy: 100.0,
            max_energy: 100.0,
            robot_type,
            mode: RobotMode::Exploring,
        }
    }
    
    pub fn update(&mut self, map: &Map) {
        self.energy -= 0.1;
        
        if self.energy > 0.0 {
            self.move_randomly(map);
        }
    }
    
    fn move_randomly(&mut self, map: &Map) {
        let mut rng = rand::thread_rng();
        
        if rng.r#gen::<f32>() > 0.7 {
            let dx = rng.gen_range(-1..=1);
            let dy = rng.gen_range(-1..=1);
            
            let new_x = (self.x as isize + dx).clamp(0, MAP_SIZE as isize - 1) as usize;
            let new_y = (self.y as isize + dy).clamp(0, MAP_SIZE as isize - 1) as usize;
            
            if map.is_valid_position(new_x, new_y) {
                self.x = new_x;
                self.y = new_y;
                self.energy -= 0.5;
            }
        }
    }
}