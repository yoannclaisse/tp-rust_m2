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
        let exploration_percentage = self.get_exploration_percentage();
        
        // Phase 1: Exploration prioritaire (0-50%)
        if exploration_percentage < 50.0 {
            return RobotType::Explorer;
        }
        
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
        
        // Phase 2: Collecte d'énergie et minerais prioritaire (50-80%)
        if exploration_percentage < 80.0 {
            // Prioriser énergie et minerais
            if energy_count > 0 && (energy_count <= 3 || self.energy_reserves < 100) {
                return RobotType::EnergyCollector;
            }
            if mineral_count > 0 && (mineral_count <= 5 || self.collected_minerals < 30) {
                return RobotType::MineralCollector;
            }
            // Sinon, continuer l'exploration
            return RobotType::Explorer;
        }
        
        // Phase 3: Collecte scientifique (80%+)
        if scientific_count > 0 && self.energy_reserves >= 100 {
            return RobotType::ScientificCollector;
        }
        
        // Si plus de ressources scientifiques, prioriser le reste
        if energy_count > 0 {
            return RobotType::EnergyCollector;
        }
        if mineral_count > 0 {
            return RobotType::MineralCollector;
        }
        
        // Par défaut, créer un explorateur pour finir l'exploration
        RobotType::Explorer
    }
    
    // Système de partage de connaissances façon "git"
    pub fn share_knowledge(&mut self, robot: &mut Robot) {
        // Ne synchroniser que si le robot est à la station ET si ce n'est pas déjà fait récemment
        if robot.x == robot.home_station_x && robot.y == robot.home_station_y {
            let mut conflicts = 0;
            let mut changes_made = false;
            
            // Le robot partage ses connaissances avec la station
            for y in 0..MAP_SIZE {
                for x in 0..MAP_SIZE {
                    if robot.memory[y][x].explored {
                        if self.global_memory[y][x].explored {
                            // CONFLIT: Résolution par timestamp (le plus récent gagne)
                            if robot.memory[y][x].timestamp > self.global_memory[y][x].timestamp {
                                self.global_memory[y][x] = robot.memory[y][x].clone();
                                conflicts += 1;
                                changes_made = true;
                            }
                        } else {
                            // Pas de conflit, ajouter les connaissances du robot
                            self.global_memory[y][x] = robot.memory[y][x].clone();
                            changes_made = true;
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
            
            // Mettre à jour les statistiques de conflits seulement si des changements ont été faits
            if changes_made {
                self.conflict_count += conflicts;
                
                if conflicts > 0 {
                    println!("Robot {} a synchronisé ses connaissances. Conflits résolus: {}", 
                             robot.id, conflicts);
                }
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
        let exploration_pct = self.get_exploration_percentage();
        
        let status = if exploration_pct >= 100.0 && self.are_all_resources_collected_placeholder() {
            "🎉 MISSION TERMINÉE!"
        } else if exploration_pct < 30.0 {
            "🔍 Phase d'exploration initiale"
        } else if exploration_pct < 60.0 {
            "⚡ Collecte d'énergie et minerais"
        } else if exploration_pct < 100.0 {
            "🧪 Collecte scientifique en cours"
        } else {
            "🏁 Finalisation de la mission"
        };
        
        format!("{} | Exploration: {:.1}% | Création robot: {}/{} énergie, {}/{} minerai | Conflits: {}", 
                status,
                exploration_pct,
                self.energy_reserves.min(50), 50,
                self.collected_minerals.min(15), 15,
                self.conflict_count)
    }

    // Fonction temporaire pour éviter les erreurs de compilation
    fn are_all_resources_collected_placeholder(&self) -> bool {
        // Cette fonction sera remplacée par le paramètre map dans les appels réels
        false
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
    
    // NOUVELLES FONCTIONS POUR LA MISSION COMPLÈTE
    
    // Vérifier si toutes les missions sont terminées
    pub fn is_all_missions_complete(&self, map: &Map, robots: &Vec<Robot>) -> bool {
        // 1. Vérifier que la carte est explorée à 100%
        if self.get_exploration_percentage() < 100.0 {
            return false;
        }
        
        // 2. Vérifier qu'il n'y a plus de ressources sur la carte
        if !self.are_all_resources_collected(map) {
            return false;
        }
        
        // 3. Vérifier que tous les robots sont revenus à la base ou en mode approprié
        for robot in robots {
            match robot.robot_type {
                RobotType::Explorer => {
                    // L'explorateur doit être en mode Idle à la station
                    if robot.mode != crate::types::RobotMode::Idle || 
                       robot.x != robot.home_station_x || 
                       robot.y != robot.home_station_y {
                        return false;
                    }
                },
                _ => {
                    // Les collecteurs doivent être en mode Idle à la station (plus de ressources à collecter)
                    if robot.mode != crate::types::RobotMode::Idle || 
                       robot.x != robot.home_station_x || 
                       robot.y != robot.home_station_y {
                        return false;
                    }
                }
            }
        }
        
        true // Toutes les conditions sont remplies
    }
    
    // Vérifier si la mission est terminée (toutes les ressources collectées)
    pub fn is_mission_complete(&self, map: &Map) -> bool {
        // Vérifier qu'il n'y a plus de ressources sur la carte
        self.are_all_resources_collected(map)
    }
    
    // Vérifier que toutes les ressources ont été collectées
    fn are_all_resources_collected(&self, map: &Map) -> bool {
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                match map.get_tile(x, y) {
                    TileType::Energy | TileType::Mineral | TileType::Scientific => {
                        return false; // Il reste encore des ressources
                    },
                    _ => {} // Les autres types ne nous intéressent pas
                }
            }
        }
        true // Aucune ressource trouvée
    }
}