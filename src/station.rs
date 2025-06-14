use crate::types::{TileType, RobotType, MAP_SIZE};
use crate::map::Map;
use crate::robot::Robot;

// Structure pour stocker les données de terrain avec métadonnées
#[derive(Clone)]
pub struct TerrainData {
    pub explored: bool,        // Si la case a été explorée
    pub timestamp: u32,        // Quand la case a été explorée
    pub robot_id: usize,       // Quel robot a exploré cette case
    pub robot_type: RobotType, // Type du robot explorateur
}

// Structure principale de la station spatiale
pub struct Station {
    pub energy_reserves: u32,              // Réserves d'énergie de la station
    pub collected_minerals: u32,           // Minerais collectés
    pub collected_scientific_data: u32,    // Données scientifiques collectées
    pub global_memory: Vec<Vec<TerrainData>>, // Mémoire partagée de toute la carte
    pub conflict_count: usize,             // Nombre de conflits de données résolus
    pub next_robot_id: usize,              // ID du prochain robot à créer
    pub current_time: u32,                 // Horloge globale de la simulation
}

impl Station {
    // Constructeur de la station
    pub fn new() -> Self {
        // Initialiser la mémoire globale avec des cases non explorées
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
    
    // Incrémente l'horloge globale de la simulation
    pub fn tick(&mut self) {
        self.current_time += 1;
    }
    
    // Tente de créer un nouveau robot si les ressources le permettent
    pub fn try_create_robot(&mut self, map: &Map) -> Option<Robot> {
        // Coûts nécessaires pour créer un robot
        let energy_cost = 50;   // Énergie requise
        let mineral_cost = 15;  // Minerais requis
        
        // Vérifier si on a assez de ressources
        if self.energy_reserves >= energy_cost && self.collected_minerals >= mineral_cost {
            // Déterminer le type de robot le plus utile actuellement
            let robot_type = self.determine_needed_robot_type(map);
            
            // Consommer les ressources nécessaires
            self.energy_reserves -= energy_cost;
            self.collected_minerals -= mineral_cost;
            
            println!("Station: Création d'un nouveau robot #{} de type {:?}", 
                     self.next_robot_id, robot_type);
            
            // Créer le robot avec la mémoire globale actuelle
            let new_robot = Robot::new_with_memory(
                map.station_x, 
                map.station_y, 
                robot_type, 
                self.next_robot_id,
                map.station_x, 
                map.station_y,
                self.global_memory.clone()
            );
            
            // Incrémenter l'ID pour le prochain robot
            self.next_robot_id += 1;
            
            return Some(new_robot);
        }
        
        None // Pas assez de ressources
    }
    
    // Analyse la situation actuelle pour déterminer le type de robot le plus utile
    fn determine_needed_robot_type(&self, map: &Map) -> RobotType {
        // Compter les ressources restantes sur la carte
        let mut energy_count = 0;
        let mut mineral_count = 0;
        let mut scientific_count = 0;
        
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                match map.get_tile(x, y) {
                    TileType::Energy => energy_count += 1,
                    TileType::Mineral => mineral_count += 1,
                    TileType::Scientific => scientific_count += 1,
                    _ => {}
                }
            }
        }
        
        // Logique de décision basée sur les besoins actuels
        
        // Si très peu d'énergie disponible ou réserves faibles -> Collecteur d'énergie
        if energy_count > 0 && (energy_count <= 3 || self.energy_reserves < 100) {
            return RobotType::EnergyCollector;
        }
        
        // Si peu de minerais mais assez d'énergie -> Collecteur de minerais
        if mineral_count > 0 && (mineral_count <= 5 || self.collected_minerals < 30) {
            return RobotType::MineralCollector;
        }
        
        // Si points d'intérêt scientifique disponibles et ressources suffisantes
        if scientific_count > 0 && self.energy_reserves >= 100 {
            return RobotType::ScientificCollector;
        }
        
        // Par défaut, créer un explorateur pour découvrir de nouvelles zones
        RobotType::Explorer
    }
    
    // Système de partage de connaissances façon "git"
    pub fn share_knowledge(&mut self, robot: &mut Robot) {
        // Ne synchroniser que si le robot est à la station
        if robot.x == robot.home_station_x && robot.y == robot.home_station_y {
            let mut conflicts = 0;
            
            // Le robot partage ses connaissances avec la station
            for y in 0..MAP_SIZE {
                for x in 0..MAP_SIZE {
                    if robot.memory[y][x].explored {
                        if self.global_memory[y][x].explored {
                            // CONFLIT: Résolution par timestamp (le plus récent gagne)
                            if robot.memory[y][x].timestamp > self.global_memory[y][x].timestamp {
                                self.global_memory[y][x] = robot.memory[y][x].clone();
                                conflicts += 1;
                            }
                        } else {
                            // Pas de conflit, ajouter les connaissances du robot
                            self.global_memory[y][x] = robot.memory[y][x].clone();
                        }
                    }
                }
            }
            
            // Le robot reçoit toutes les connaissances globales
            for y in 0..MAP_SIZE {
                for x in 0..MAP_SIZE {
                    if self.global_memory[y][x].explored {
                        robot.memory[y][x] = self.global_memory[y][x].clone();
                    }
                }
            }
            
            // Mettre à jour les statistiques de conflits
            self.conflict_count += conflicts;
            
            if conflicts > 0 {
                println!("Robot {} a synchronisé ses connaissances. Conflits résolus: {}", 
                         robot.id, conflicts);
            }
        }
    }
    
    // Dépose des ressources collectées par les robots
    pub fn deposit_resources(&mut self, minerals: u32, scientific_data: u32) {
        self.collected_minerals += minerals;
        self.collected_scientific_data += scientific_data;
        self.energy_reserves += minerals; // Conversion minerais -> énergie
    }
    
    // Génère un rapport sur l'état actuel de la station
    pub fn get_status(&self) -> String {
        let status = match (self.energy_reserves, self.collected_minerals) {
            (e, m) if e < 30 => "Faible en énergie",
            (e, m) if m < 10 => "Faible en minerais", 
            (e, m) if e >= 200 && m >= 50 => "Ressources abondantes",
            _ => "Ressources adéquates",
        };
        
        format!("{} | Création robot: {}/{} énergie, {}/{} minerai | Conflits: {}", 
                status, 
                self.energy_reserves.min(50), 50,      // Progression vers énergie requise
                self.collected_minerals.min(15), 15,   // Progression vers minerais requis
                self.conflict_count)
    }
    
    // Calcule le pourcentage d'exploration global
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