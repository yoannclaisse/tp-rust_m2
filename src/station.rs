use crate::types::{TileType, RobotType, MAP_SIZE};
use crate::map::Map;
use crate::robot::Robot;

pub struct Station {
    pub energy_reserves: u32,
    pub collected_minerals: u32,
    pub collected_scientific_data: u32,
}

impl Station {
    pub fn new() -> Self {
        Self {
            energy_reserves: 100,
            collected_minerals: 0,
            collected_scientific_data: 0,
        }
    }
    
    pub fn deposit_resources(&mut self, minerals: u32, scientific_data: u32) {
        self.collected_minerals += minerals;
        self.collected_scientific_data += scientific_data;
        self.energy_reserves += minerals; // Convert minerals to energy
    }
    
    pub fn get_status(&self) -> String {
        let status = match (self.energy_reserves, self.collected_minerals) {
            (e, m) if e < 30 => "Faible en énergie",
            (e, m) if m < 10 => "Faible en minerais",
            (e, m) if e >= 200 && m >= 50 => "Ressources abondantes",
            _ => "Ressources adéquates",
        };
        
        format!("{} | Énergie: {} | Minerais: {} | Données: {}", 
                status, self.energy_reserves, self.collected_minerals, self.collected_scientific_data)
    }
}