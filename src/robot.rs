//! # Robot AI and Behavior Module
//! 
//! This module implements the autonomous robot system for exoplanet exploration.
//! Each robot is an independent AI agent with specialized capabilities, behavior patterns,
//! and decision-making algorithms optimized for efficient exploration and resource collection.
//! 
//! ## AI Architecture
//! 
//! The robot AI uses a hybrid behavior-based architecture combining:
//! - **Reactive Behaviors**: Immediate responses to environmental conditions
//! - **Deliberative Planning**: Long-term pathfinding and mission planning
//! - **Learning Systems**: Exploration memory and experience-based optimization
//! 
//! ## Specialization System
//! 
//! Different robot types have distinct AI personalities and capabilities:
//! - **Explorers**: Aggressive exploration with extensive sensor range
//! - **Collectors**: Resource-focused behavior with efficiency optimization
//! - **Hybrid Modes**: Dynamic switching between exploration and collection

use crate::types::{MAP_SIZE, TileType, RobotType, RobotMode};
use crate::map::Map;
use crate::station::{Station, TerrainData};
use rand::prelude::*;
use std::collections::{VecDeque, BinaryHeap, HashMap};
use std::cmp::Ordering;

// NOTE - Node structure for A* pathfinding algorithm
#[derive(Clone, Eq, PartialEq)]
struct Node {
    // NOTE - Node position on the map
    position: (usize, usize),
    // NOTE - Cost from start to this node
    g_cost: usize,
    // NOTE - Estimated total cost (g_cost + heuristic)
    f_cost: usize,
}

// NOTE - Implement ordering for priority queue (min-heap for A*)
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // NOTE - Reverse order for min-heap
        other.f_cost.cmp(&self.f_cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// NOTE - Main robot structure with all mission state
pub struct Robot {
    // NOTE - Current X position on the map
    pub x: usize,
    // NOTE - Current Y position on the map
    pub y: usize,
    // NOTE - Current energy level
    pub energy: f32,
    // NOTE - Maximum energy capacity
    pub max_energy: f32,
    // NOTE - Minerals carried (for MineralCollector)
    pub minerals: u32,
    // NOTE - Scientific data carried (for ScientificCollector)
    pub scientific_data: u32,
    // NOTE - Robot specialization type
    pub robot_type: RobotType,
    // NOTE - Current operational mode
    pub mode: RobotMode,
    // NOTE - Local exploration memory (per robot)
    pub memory: Vec<Vec<TerrainData>>,
    // NOTE - Planned path to station (A* waypoints)
    pub path_to_station: VecDeque<(usize, usize)>,
    // NOTE - Unique robot identifier
    pub id: usize,
    // NOTE - Home station X coordinate
    pub home_station_x: usize,
    // NOTE - Home station Y coordinate
    pub home_station_y: usize,
    // NOTE - Last time data was synchronized with station
    pub last_sync_time: u32,
    // NOTE - Prevents duplicate exploration completion logs
    pub exploration_complete_announced: bool,
}

impl Robot {
    /// NOTE - Create a new robot with default configuration
    pub fn new(x: usize, y: usize, robot_type: RobotType) -> Self {
        // NOTE - Set energy based on robot type
        let (max_energy, energy) = match robot_type {
            RobotType::Explorer => (80.0, 80.0),           // Balanced capacity for exploration
            RobotType::EnergyCollector => (120.0, 120.0),  // High capacity for extended missions
            RobotType::MineralCollector => (100.0, 100.0), // Good endurance for mining work
            RobotType::ScientificCollector => (60.0, 60.0), // Limited by instrument power needs
        };
        
        // NOTE - Initialize empty exploration memory
        let mut memory = Vec::with_capacity(MAP_SIZE);
        for _ in 0..MAP_SIZE {
            let row = vec![
                TerrainData {
                    explored: false,                    // No tiles explored yet
                    timestamp: 0,                       // No exploration time recorded
                    robot_id: 0,                        // Placeholder robot ID
                    robot_type: RobotType::Explorer,    // Default type for unexplored tiles
                }; 
                MAP_SIZE
            ];
            memory.push(row);
        }
        
        Self {
            x,
            y,
            energy,
            max_energy,
            minerals: 0,                            // Start with empty mineral storage
            scientific_data: 0,                     // Start with no scientific data
            robot_type,
            mode: RobotMode::Exploring,             // Begin mission in exploration mode
            memory,
            path_to_station: VecDeque::new(),       // No planned path initially
            id: 0,                                  // ID will be assigned by station
            home_station_x: x,                      // Remember starting position as home
            home_station_y: y,
            last_sync_time: 0,                      // No synchronization performed yet
            exploration_complete_announced: false,  // Haven't announced completion
        }
    }
    
    // NOTE - Create robot with preloaded memory (for station deployment)
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
            x,
            y,
            energy,
            max_energy,
            minerals: 0,
            scientific_data: 0,
            robot_type,
            mode: RobotMode::Exploring,
            memory,
            path_to_station: VecDeque::new(),
            id,
            home_station_x: station_x,
            home_station_y: station_y,
            last_sync_time: 0,
            exploration_complete_announced: false,
        }
    }
    
    // NOTE - Get display character for robot type (for UI)
    pub fn get_display_char(&self) -> &str {
        match self.robot_type {
            RobotType::Explorer => "🤖",
            RobotType::EnergyCollector => "🔋",
            RobotType::MineralCollector => "⛏️",
            RobotType::ScientificCollector => "🧪",
        }
    }
    
    // NOTE - Get display color for robot type (for UI)
    pub fn get_display_color(&self) -> u8 {
        match self.robot_type {
            RobotType::Explorer => 9,          // Rouge vif
            RobotType::EnergyCollector => 10,  // Vert vif
            RobotType::MineralCollector => 13, // Magenta vif
            RobotType::ScientificCollector => 12, // Bleu vif
        }
    }
    
    // NOTE - Update robot's local exploration memory (improved version)
    pub fn update_memory(&mut self, map: &Map, station: &Station) {
        let _ = map;
        // NOTE - Mark current tile as explored with timestamp
        self.memory[self.y][self.x] = TerrainData {
            explored: true,
            timestamp: station.current_time,
            robot_id: self.id,
            robot_type: self.robot_type,
        };
        
        // NOTE - Set vision range based on robot type
        let vision_range = match self.robot_type {
            RobotType::Explorer => 4, // Vision étendue pour l'explorateur
            _ => 2,                   // Vision standard pour les autres
        };
        
        for dy in -vision_range..=vision_range {
            for dx in -vision_range..=vision_range {
                let nx = self.x as isize + dx;
                let ny = self.y as isize + dy;
                
                if nx >= 0 && nx < MAP_SIZE as isize && ny >= 0 && ny < MAP_SIZE as isize {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    
                    // Si la case n'est pas encore explorée ou si notre info est plus récente
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
    
    // NOTE - Main update method for robot behavior
    pub fn update(&mut self, map: &mut Map, station: &mut Station) {
        // NOTE - Consume base metabolism energy
        self.energy -= 0.1;
        
        // NOTE - Check if exploration is complete (explorers only)
        if self.robot_type == RobotType::Explorer {
            if self.is_exploration_complete() && !self.exploration_complete_announced {
                println!("🌍 EXPLORATION DE L'EXOPLANÈTE TERMINÉE ! 🌍");
                println!("Robot explorateur #{} a cartographié 100% de la planète.", self.id);
                self.exploration_complete_announced = true;
            }
        }
        
        // NOUVELLE LOGIQUE: Les collecteurs attendent que l'exploration atteigne un seuil minimum
        if self.robot_type != RobotType::Explorer {
            let exploration_percentage = station.get_exploration_percentage();
            
            // Les collecteurs attendent au moins 30% d'exploration avant de commencer
            if exploration_percentage < 30.0 {
                // Rester à la station en mode Idle
                if self.x != self.home_station_x || self.y != self.home_station_y {
                    self.mode = RobotMode::ReturnToStation;
                    self.plan_path_to_station(map);
                } else {
                    self.mode = RobotMode::Idle;
                }
                return;
            }
            
            // À 30-60% d'exploration, seuls les collecteurs d'énergie et de minerais travaillent
            if exploration_percentage < 60.0 && self.robot_type == RobotType::ScientificCollector {
                if self.x != self.home_station_x || self.y != self.home_station_y {
                    self.mode = RobotMode::ReturnToStation;
                    self.plan_path_to_station(map);
                } else {
                    self.mode = RobotMode::Idle;
                }
                return;
            }
        }
        
        // NOTE - Check if robot should return to station
        if self.should_return_to_station(map) {
            self.mode = RobotMode::ReturnToStation;
            self.plan_path_to_station(map);
        }
        
        // NOTE - For collectors, check if resources remain to collect
        if self.robot_type != RobotType::Explorer && self.mode == RobotMode::Exploring {
            // Vérifier d'abord si on peut voir des ressources (exploration suffisante)
            if let Some(_resource_pos) = self.find_nearest_known_resource(map, station) {
                // Il y a des ressources connues, continuer la collecte
            } else {
                // Pas de ressources connues dans les zones explorées
                if self.x != self.home_station_x || self.y != self.home_station_y {
                    self.mode = RobotMode::ReturnToStation;
                    self.plan_path_to_station(map);
                } else {
                    self.mode = RobotMode::Idle;
                    println!("🏁 Robot collecteur #{} : Aucune ressource connue, passage en mode Idle", self.id);
                }
            }
        }
        
        // NOTE - If at station, recharge, sync, and change mode
        if self.x == self.home_station_x && self.y == self.home_station_y {
            // Recharger et décharger
            self.energy = self.max_energy;
            station.deposit_resources(self.minerals, self.scientific_data);
            self.minerals = 0;
            self.scientific_data = 0;
            
            // Synchroniser les connaissances avec la station
            if station.current_time > self.last_sync_time {
                station.share_knowledge(self);
                self.last_sync_time = station.current_time;
            }
            
            // Changer de mode après avoir rechargé
            match self.robot_type {
                RobotType::Explorer => {
                    // Si l'exploration est terminée, rester à la station en mode Idle
                    if self.is_exploration_complete() {
                        self.mode = RobotMode::Idle;
                        if !self.exploration_complete_announced {
                            println!("🏠 Robot explorateur #{} : Mission terminée, retour définitif à la base.", self.id);
                        }
                    } else {
                        // Sinon, retourner explorer
                        self.mode = RobotMode::Exploring;
                    }
                },
                _ => {
                    // Les collecteurs cherchent des ressources
                    if let Some(resource_pos) = self.find_nearest_resource(map) {
                        self.path_to_station = self.find_path(map, resource_pos);
                        self.mode = RobotMode::Collecting;
                    } else {
                        // Si pas de ressource trouvée, rester à la station en mode Idle
                        self.mode = RobotMode::Idle;
                        println!("🏁 Robot collecteur #{} : Aucune ressource trouvée, reste en mode Idle", self.id);
                    }
                }
            }
        }
        
        // NOTE - Logique de déplacement selon le mode
        match self.mode {
            RobotMode::Idle => {
                // Pour les explorateurs : si l'exploration est terminée, rester à la station
                if self.robot_type == RobotType::Explorer && self.is_exploration_complete() {
                    // Ne rien faire, rester à la station
                    return;
                }
                
                // Pour les autres ou si exploration pas terminée, retourner en mode exploration
                if self.robot_type == RobotType::Explorer {
                    self.mode = RobotMode::Exploring;
                }
            },
            RobotMode::Exploring => {
                // Pour les explorateurs : vérifier si l'exploration est terminée
                if self.robot_type == RobotType::Explorer && self.is_exploration_complete() {
                    // Si l'exploration est terminée, retourner à la station et y rester
                    self.mode = RobotMode::ReturnToStation;
                    self.plan_path_to_station(map);
                    return;
                }
                
                // Si c'est un collecteur, vérifier s'il y a des ressources à proximité
                if self.robot_type != RobotType::Explorer {
                    if let Some(resource_pos) = self.find_nearest_resource(map) {
                        let distance = self.heuristic((self.x, self.y), resource_pos);
                        if distance <= 5 {  // Distance de détection
                            self.path_to_station = self.find_path(map, resource_pos);
                            self.mode = RobotMode::Collecting;
                            return;
                        }
                    }
                }
                
                // Sinon, explorer normalement
                self.explore_move(map);
            },
            RobotMode::Collecting => {
                // Si on est sur la ressource cible, la collecter
                let tile = map.get_tile(self.x, self.y);
                let can_collect = match (self.robot_type, tile) {
                    (RobotType::EnergyCollector, TileType::Energy) => true,
                    (RobotType::MineralCollector, TileType::Mineral) => true,
                    (RobotType::ScientificCollector, TileType::Scientific) => true,
                    _ => false,
                };
                
                if can_collect {
                    self.collect_resources(map);
                } else if !self.path_to_station.is_empty() {
                    // Suivre le chemin vers la ressource
                    let next = self.path_to_station.pop_front().unwrap();
                    self.move_to(next.0, next.1);
                } else {
                    // Si le chemin est vide mais qu'on n'est pas sur la ressource, chercher une autre ressource
                    if let Some(resource_pos) = self.find_nearest_resource(map) {
                        self.path_to_station = self.find_path(map, resource_pos);
                    } else {
                        // Si plus de ressources, retourner à la station
                        self.mode = RobotMode::ReturnToStation;
                        self.plan_path_to_station(map);
                    }
                }
            },
            RobotMode::ReturnToStation => {
                if !self.path_to_station.is_empty() {
                    // Suivre le chemin vers la station
                    let next = self.path_to_station.pop_front().unwrap();
                    self.move_to(next.0, next.1);
                } else {
                    // Si le chemin est vide mais qu'on n'est pas à la station, replanifier
                    if self.x != self.home_station_x || self.y != self.home_station_y {
                        self.plan_path_to_station(map);
                        if !self.path_to_station.is_empty() {
                            let next = self.path_to_station.pop_front().unwrap();
                            self.move_to(next.0, next.1);
                        } else {
                            // Si on ne peut pas générer de chemin, revenir en mode exploration
                            self.mode = RobotMode::Exploring;
                        }
                    } else {
                        // Si on est à la station, passer en mode idle
                        self.mode = RobotMode::Idle;
                    }
                }
            }
        }
        
        // NOTE - Mettre à jour la mémoire
        self.update_memory(map, station);
    }
    
    // NOTE - Smart exploration movement (improved version)
    fn explore_move(&mut self, map: &Map) {
        // Pour l'explorateur, utiliser une stratégie plus agressive de recherche de cases non explorées
        if self.robot_type == RobotType::Explorer {
            self.explorer_specific_move(map);
        } else {
            // Logique normale pour les autres types de robots
            self.standard_explore_move(map);
        }
    }
    
    // NOTE - Explorer-specific movement logic
    fn explorer_specific_move(&mut self, map: &Map) {
        // Chercher les cases non explorées sur TOUTE la carte (pas juste à proximité)
        let mut unexplored_tiles = Vec::new();
        
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                // Si la case n'est pas explorée (case "?")
                if !self.memory[y][x].explored {
                    let distance = self.heuristic((self.x, self.y), (x, y));
                    unexplored_tiles.push((x, y, distance));
                }
            }
        }
        
        // Si des cases non explorées sont trouvées
        if !unexplored_tiles.is_empty() {
            // Trier par distance pour aller vers la plus proche
            unexplored_tiles.sort_by_key(|&(_, _, dist)| dist);
            
            // Prendre les 3 plus proches et choisir aléatoirement parmi elles
            // (pour éviter que tous les explorateurs aillent au même endroit)
            let candidates = unexplored_tiles.iter().take(3).collect::<Vec<_>>();
            let mut rng = rand::thread_rng();
            let target_idx = rng.gen_range(0..candidates.len());
            let target = (candidates[target_idx].0, candidates[target_idx].1);
            
            // Utiliser A* pour trouver le chemin optimal vers la case "?"
            let path = self.find_path(map, target);
            
            if !path.is_empty() {
                let next = path[0];
                self.move_to(next.0, next.1);
                return;
            }
        }
        
        // Si aucune case non explorée ou impossible d'y aller, mouvement aléatoire intelligent
        self.intelligent_random_move(map);
    }
    
    // NOTE - Intelligent random move for explorer
    fn intelligent_random_move(&mut self, map: &Map) {
        let mut possible_moves = Vec::new();
        
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let nx = self.x as isize + dx;
                let ny = self.y as isize + dy;
                
                if nx >= 0 && nx < MAP_SIZE as isize && ny >= 0 && ny < MAP_SIZE as isize 
                   && map.is_valid_position(nx as usize, ny as usize) {
                    
                    let new_pos = (nx as usize, ny as usize);
                    
                    // Priorité : cases non visitées récemment ou jamais visitées
                    let priority = if !self.memory[new_pos.1][new_pos.0].explored {
                        100 // Très haute priorité pour les cases "?"
                    } else {
                        // Priorité inversement proportionnelle au timestamp (cases anciennes = priorité plus haute)
                        let age = self.last_sync_time.saturating_sub(self.memory[new_pos.1][new_pos.0].timestamp);
                        age.min(50) // Limiter la priorité
                    };
                    
                    possible_moves.push((new_pos.0, new_pos.1, priority));
                }
            }
        }
        
        if !possible_moves.is_empty() {
            // Choisir une case avec probabilité proportionnelle à la priorité
            possible_moves.sort_by_key(|&(_, _, priority)| std::cmp::Reverse(priority));
            
            // Prendre une des 3 meilleures options avec une probabilité décroissante
            let mut rng = rand::thread_rng();
            let choice = if rng.gen_bool(0.6) && !possible_moves.is_empty() {
                0 // 60% de chance de prendre la meilleure option
            } else if rng.gen_bool(0.3) && possible_moves.len() > 1 {
                1 // 30% de chance de prendre la deuxième
            } else if possible_moves.len() > 2 {
                2 // 10% de chance de prendre la troisième
            } else {
                rng.gen_range(0..possible_moves.len())
            };
            
            let (nx, ny, _) = possible_moves[choice];
            self.move_to(nx, ny);
        }
    }
    
    // NOTE - Standard explore move for other robots
    fn standard_explore_move(&mut self, map: &Map) {
        // Logique originale mais avec une portée réduite pour les non-explorateurs
        let mut unexplored_tiles = Vec::new();
        let vision_range = 3; // Portée réduite pour les collecteurs
        
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                if !self.memory[y][x].explored {
                    let distance = self.heuristic((self.x, self.y), (x, y));
                    if distance <= vision_range {
                        unexplored_tiles.push((x, y, distance));
                    }
                }
            }
        }
        
        if !unexplored_tiles.is_empty() {
            unexplored_tiles.sort_by_key(|&(_, _, dist)| dist);
            let target = (unexplored_tiles[0].0, unexplored_tiles[0].1);
            let path = self.find_path(map, target);
            
            if !path.is_empty() {
                let next = path[0];
                self.move_to(next.0, next.1);
                return;
            }
        }
        
        // Mouvement aléatoire simple pour les collecteurs
        let mut rng = rand::thread_rng();
        let mut possible_moves = Vec::new();
        
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let nx = self.x as isize + dx;
                let ny = self.y as isize + dy;
                
                if nx >= 0 && nx < MAP_SIZE as isize && ny >= 0 && ny < MAP_SIZE as isize 
                   && map.is_valid_position(nx as usize, ny as usize) {
                    possible_moves.push((nx as usize, ny as usize));
                }
            }
        }
        
        if !possible_moves.is_empty() {
            let (nx, ny) = possible_moves[rng.gen_range(0..possible_moves.len())];
            self.move_to(nx, ny);
        }
    }
    
    // NOTE - Find nearest known resource in explored areas
    fn find_nearest_known_resource(&self, map: &Map, station: &Station) -> Option<(usize, usize)> {
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
                // Vérifier que la case est explorée ET contient la ressource recherchée
                if station.global_memory[y][x].explored && map.get_tile(x, y) == target_resource {
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
    
    // NOTE - Collect resources based on robot type
    fn collect_resources(&mut self, map: &mut Map) {
        let tile = map.get_tile(self.x, self.y);
        
        match (self.robot_type, tile) {
            (RobotType::EnergyCollector, TileType::Energy) => {
                if self.energy < self.max_energy {
                    self.energy += 10.0;
                    if self.energy > self.max_energy {
                        self.energy = self.max_energy;
                    }
                    map.consume_resource(self.x, self.y);
                    println!("🔋 Robot #{} a collecté de l'énergie à ({}, {})", self.id, self.x, self.y);
                }
            },
            (RobotType::MineralCollector, TileType::Mineral) => {
                self.minerals += 1;
                map.consume_resource(self.x, self.y);
                println!("⛏️ Robot #{} a collecté un minerai à ({}, {})", self.id, self.x, self.y);
            },
            (RobotType::ScientificCollector, TileType::Scientific) => {
                self.scientific_data += 1;
                map.consume_resource(self.x, self.y);
                println!("🧪 Robot #{} a collecté des données scientifiques à ({}, {})", self.id, self.x, self.y);
            },
            _ => {
                // Si pas de ressource à collecter, explorer
                self.explore_move(map);
            }
        }
        
        // Après avoir collecté, vérifier s'il reste des ressources
        if let Some(resource_pos) = self.find_nearest_resource(map) {
            self.path_to_station = self.find_path(map, resource_pos);
        } else {
            // Si plus de ressources, retourner à la station
            self.mode = RobotMode::ReturnToStation;
            self.plan_path_to_station(map);
        }
    }
    
    // NOTE - Check if robot should return to station
    fn should_return_to_station(&self, map: &Map) -> bool {
        let _ = map;
        
        // Pour les explorateurs : retourner si exploration terminée OU énergie faible
        if self.robot_type == RobotType::Explorer {
            if self.is_exploration_complete() {
                return true;
            }
        }
        
        // Retourner si énergie faible
        if self.energy < self.max_energy * 0.3 {
            return true;
        }
        
        // Retourner si inventaire plein (selon le type)
        match self.robot_type {
            RobotType::MineralCollector => self.minerals >= 5,
            RobotType::ScientificCollector => self.scientific_data >= 3,
            _ => false
        }
    }
    
    // NOTE - Plan path to station using A*
    fn plan_path_to_station(&mut self, map: &Map) {
        let target = (self.home_station_x, self.home_station_y);
        self.path_to_station = self.find_path(map, target);
    }
    
    // NOTE - Find nearest resource for robot type
    fn find_nearest_resource(&self, map: &Map) -> Option<(usize, usize)> {
        let target_resource = match self.robot_type {
            RobotType::Explorer => None,
            RobotType::EnergyCollector => Some(TileType::Energy),
            RobotType::MineralCollector => Some(TileType::Mineral),
            RobotType::ScientificCollector => Some(TileType::Scientific),
        };
        
        let target_resource = match target_resource {
            Some(res) => res,
            None => return None,
        };
        
        let mut nearest = None;
        let mut min_distance = usize::MAX;
        
        // Chercher dans TOUTE la carte (pour compatibilité avec l'ancien code)
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
    
    // NOTE - A* pathfinding algorithm for optimal route
    fn find_path(&self, map: &Map, target: (usize, usize)) -> VecDeque<(usize, usize)> {
        let start = (self.x, self.y);
        
        // Si déjà à destination
        if start == target {
            return VecDeque::new();
        }
        
        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut g_score: HashMap<(usize, usize), usize> = HashMap::new();
        
        // Initialiser les valeurs de départ
        g_score.insert(start, 0);
        open_set.push(Node {
            position: start,
            g_cost: 0,
            f_cost: self.heuristic(start, target),
        });
        
        while let Some(current) = open_set.pop() {
            let current_pos = current.position;
            
            // Si on est arrivé à destination
            if current_pos == target {
                // Reconstruire le chemin
                let mut path = VecDeque::new();
                let mut current = target;
                
                while current != start {
                    path.push_front(current);
                    current = *came_from.get(&current).unwrap();
                }
                
                return path;
            }
            
            // Examiner tous les voisins
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue; // Ignorer la position actuelle
                    }
                    
                    let nx = current_pos.0 as isize + dx;
                    let ny = current_pos.1 as isize + dy;
                    
                    // Vérifier si la position est valide
                    if nx < 0 || nx >= MAP_SIZE as isize || ny < 0 || ny >= MAP_SIZE as isize {
                        continue;
                    }
                    
                    let neighbor = (nx as usize, ny as usize);
                    
                    // Vérifier si c'est un obstacle
                    if !map.is_valid_position(neighbor.0, neighbor.1) {
                        continue;
                    }
                    
                    // Calculer le nouveau coût
                    let tentative_g_score = g_score[&current_pos] + 1;
                    
                    // Si on a trouvé un meilleur chemin
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
        
        // Si on ne trouve pas de chemin, retourner un chemin vide
        VecDeque::new()
    }
    
    // NOTE - Heuristic for A* (Manhattan distance)
    fn heuristic(&self, a: (usize, usize), b: (usize, usize)) -> usize {
        let dx = (a.0 as isize - b.0 as isize).abs() as usize;
        let dy = (a.1 as isize - b.1 as isize).abs() as usize;
        dx + dy
    }
    
    // NOTE - Move robot to a position
    fn move_to(&mut self, x: usize, y: usize) {
        // Calculer la distance
        let dx = (x as isize - self.x as isize).abs();
        let dy = (y as isize - self.y as isize).abs();
        let distance = dx.max(dy) as f32;
        
        // Consommer de l'énergie selon la distance et le type de robot
        let energy_cost = match self.robot_type {
            RobotType::Explorer => 0.3 * distance,
            RobotType::EnergyCollector => 0.4 * distance,
            RobotType::MineralCollector => 0.5 * distance,
            RobotType::ScientificCollector => 0.6 * distance,
        };
        
        self.energy -= energy_cost;
        
        // Mettre à jour la position
        self.x = x;
        self.y = y;
    }
    
    // NOTE - Calculate percentage of map explored by this robot
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
    
    // NOTE - Check if exploration is complete (100%)
    fn is_exploration_complete(&self) -> bool {
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                if !self.memory[y][x].explored {
                    return false; // Il reste des cases non explorées
                }
            }
        }
        true // Toutes les cases sont explorées
    }
}