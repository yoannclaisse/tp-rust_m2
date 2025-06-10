use crate::types::{MAP_SIZE, RobotType, RobotMode};
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
    pub robot_type: RobotType,
    pub mode: RobotMode,
    pub memory: Vec<Vec<bool>>,
    pub target: Option<(usize, usize)>,
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
            memory: vec![vec![false; MAP_SIZE]; MAP_SIZE],
            target: None,
        }
    }
    
    pub fn update(&mut self, map: &Map) {
        self.energy -= 0.1;
        self.update_memory();
        
        if self.energy > 10.0 {
            self.intelligent_move(map);
        } else {
            // Return to station when low energy
            self.target = Some((map.station_x, map.station_y));
            self.move_towards_target(map);
        }
    }
    
    fn update_memory(&mut self) {
        self.memory[self.y][self.x] = true;
        
        // Vision range
        for dy in -2..=2 {
            for dx in -2..=2 {
                let nx = self.x as isize + dx;
                let ny = self.y as isize + dy;
                
                if nx >= 0 && nx < MAP_SIZE as isize && ny >= 0 && ny < MAP_SIZE as isize {
                    self.memory[ny as usize][nx as usize] = true;
                }
            }
        }
    }
    
    fn intelligent_move(&mut self, map: &Map) {
        // If at station, recharge and find new target
        if self.x == map.station_x && self.y == map.station_y {
            self.energy = self.max_energy;
            self.target = self.find_exploration_target();
        }
        
        if let Some(target) = self.target {
            if self.x == target.0 && self.y == target.1 {
                self.target = self.find_exploration_target();
            }
            
            if let Some(target) = self.target {
                self.move_towards_target(map);
            }
        } else {
            self.target = self.find_exploration_target();
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
            if let Some(next_pos) = path.front() {
                self.x = next_pos.0;
                self.y = next_pos.1;
                self.energy -= 0.5;
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