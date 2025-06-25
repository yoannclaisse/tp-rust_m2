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

/// A* pathfinding algorithm node for optimal route calculation.
/// 
/// This structure represents a single position in the A* search space,
/// containing both actual movement cost and heuristic estimates for
/// efficient pathfinding across the exploration map.
/// 
/// # Algorithm Details
/// 
/// The A* algorithm uses `f_cost = g_cost + h_cost` where:
/// - `g_cost`: Actual movement cost from start to this position
/// - `h_cost`: Heuristic estimate from this position to goal (Manhattan distance)
/// - `f_cost`: Total estimated cost of path through this position
/// 
/// # Examples
/// 
/// ```rust
/// let node = Node {
///     position: (5, 3),
///     g_cost: 8,      // 8 steps from start
///     f_cost: 15,     // 8 + 7 (estimated 7 steps to goal)
/// };
/// ```
#[derive(Clone, Eq, PartialEq)]
struct Node {
    /// Coordinates (x, y) of this node on the exploration map
    position: (usize, usize),
    
    /// Actual movement cost from the pathfinding start position to this node
    /// Represents the confirmed minimum cost to reach this position
    g_cost: usize,
    
    /// Total estimated cost (g_cost + heuristic) for a path through this node
    /// Used by A* algorithm to prioritize exploration of promising nodes
    f_cost: usize,
}

// Implementation for priority queue ordering in A* algorithm
// Rust's BinaryHeap is a max-heap, but A* needs a min-heap (lowest f_cost first)
impl Ord for Node {
    /// Compares nodes by f_cost for priority queue ordering.
    /// 
    /// Returns reverse ordering to convert BinaryHeap max-heap into min-heap behavior.
    /// Lower f_cost values will be processed first by the A* algorithm.
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse comparison: other.cmp(self) creates min-heap from max-heap
        other.f_cost.cmp(&self.f_cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Autonomous exploration robot with specialized AI behavior and capabilities.
/// 
/// Each robot is an independent agent capable of exploration, resource collection,
/// pathfinding, and coordination with the central station. The robot's behavior
/// is determined by its specialization type and current operational mode.
/// 
/// # AI Behavior System
/// 
/// The robot AI operates through a state machine with the following modes:
/// - **Exploring**: Actively seeks unexplored map areas using intelligent search
/// - **Collecting**: Focuses on gathering resources matching robot specialization  
/// - **ReturnToStation**: Navigates back to base for resupply and data synchronization
/// - **Idle**: Standby mode while awaiting new missions or resource availability
/// 
/// # Memory System
/// 
/// Each robot maintains local exploration memory that is periodically synchronized
/// with the central station. This enables:
/// - Independent operation during communication blackouts
/// - Conflict detection and resolution for overlapping exploration
/// - Distributed knowledge sharing across the robot fleet
/// 
/// # Examples
/// 
/// ```rust
/// use ereea::robot::Robot;
/// use ereea::types::RobotType;
/// 
/// // Create a specialized exploration robot
/// let mut explorer = Robot::new(10, 10, RobotType::Explorer);
/// assert_eq!(explorer.robot_type, RobotType::Explorer);
/// assert_eq!(explorer.energy, 80.0); // Explorer energy capacity
/// 
/// // Robot can move and update its state
/// explorer.update(&mut map, &mut station);
/// ```
pub struct Robot {
    /// Current X coordinate position on the exploration map (0 to MAP_SIZE-1)
    pub x: usize,
    
    /// Current Y coordinate position on the exploration map (0 to MAP_SIZE-1)  
    pub y: usize,
    
    /// Current energy level (0.0 = completely depleted, max_energy = fully charged)
    /// 
    /// Energy is consumed by:
    /// - Movement (varies by robot type and distance)
    /// - Sensor operations and environmental scanning
    /// - Resource collection and processing activities
    /// - Communication with other robots and station
    pub energy: f32,
    
    /// Maximum energy capacity for this robot type
    /// 
    /// Different robot types have varying energy capacities:
    /// - Explorer: 80.0 (balanced for extended exploration)
    /// - EnergyCollector: 120.0 (high capacity for long missions)
    /// - MineralCollector: 100.0 (good endurance for mining operations)
    /// - ScientificCollector: 60.0 (limited by instrument power requirements)
    pub max_energy: f32,
    
    /// Number of mineral units currently carried by the robot
    /// 
    /// Only MineralCollector robots can carry minerals. Storage capacity
    /// is limited and affects movement speed when heavily loaded.
    /// Minerals must be deposited at the station before collection can continue.
    pub minerals: u32,
    
    /// Number of scientific data units currently stored by the robot
    /// 
    /// Only ScientificCollector robots can gather scientific data.
    /// Data represents analyzed samples, readings, and observations
    /// that contribute to mission scientific objectives.
    pub scientific_data: u32,
    
    /// Specialization category determining robot capabilities and behavior patterns
    /// 
    /// This field affects:
    /// - Available actions and resource collection abilities
    /// - Energy capacity and consumption rates  
    /// - AI behavior patterns and decision algorithms
    /// - Visual representation in user interfaces
    pub robot_type: RobotType,
    
    /// Current operational mode controlling robot behavior and decision-making
    /// 
    /// The mode determines which AI algorithms are active and how the robot
    /// responds to environmental conditions and mission objectives.
    /// Modes can change automatically based on energy, inventory, and mission status.
    pub mode: RobotMode,
    
    /// Local exploration memory containing discovered map information
    /// 
    /// Each robot maintains its own exploration memory that may differ from
    /// other robots and the central station. Memory is synchronized during
    /// station visits, enabling conflict detection and knowledge sharing.
    /// 
    /// Structure: `memory[y][x]` corresponds to map position (x, y)
    pub memory: Vec<Vec<TerrainData>>,
    
    /// Planned movement path as a sequence of coordinate waypoints
    /// 
    /// Generated by A* pathfinding algorithm for optimal navigation to
    /// target destinations. The robot follows this path step-by-step,
    /// consuming waypoints as it moves. Empty when no planned movement.
    pub path_to_station: VecDeque<(usize, usize)>,
    
    /// Unique identifier for this robot across the entire mission
    /// 
    /// Robot IDs are assigned sequentially by the station and used for:
    /// - Tracking individual robot performance and contributions
    /// - Conflict resolution in exploration memory synchronization
    /// - User interface display and mission reporting
    pub id: usize,
    
    /// X coordinate of the robot's home station for return navigation
    pub home_station_x: usize,
    
    /// Y coordinate of the robot's home station for return navigation  
    pub home_station_y: usize,
    
    /// Timestamp of the robot's last data synchronization with the station
    /// 
    /// Used to prevent redundant synchronization operations and optimize
    /// communication efficiency. Updated whenever the robot exchanges
    /// exploration data with the central station memory.
    pub last_sync_time: u32,
    
    /// Flag preventing duplicate exploration completion announcements
    /// 
    /// Set to true when the robot has announced completion of its exploration
    /// objectives. Prevents spamming the mission log with repeated messages
    /// about the same achievement.
    pub exploration_complete_announced: bool,
}

impl Robot {
    /// Creates a new robot with default configuration at the specified position.
    /// 
    /// This constructor initializes a robot with type-appropriate energy capacity
    /// and creates empty exploration memory. The robot starts in Exploring mode
    /// and is ready for immediate deployment.
    /// 
    /// # Parameters
    /// 
    /// * `x` - Initial X coordinate position on the map
    /// * `y` - Initial Y coordinate position on the map  
    /// * `robot_type` - Specialization determining capabilities and behavior
    /// 
    /// # Returns
    /// 
    /// Newly created Robot instance ready for mission deployment
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let explorer = Robot::new(5, 7, RobotType::Explorer);
    /// assert_eq!(explorer.x, 5);
    /// assert_eq!(explorer.y, 7);
    /// assert_eq!(explorer.mode, RobotMode::Exploring);
    /// ```
    pub fn new(x: usize, y: usize, robot_type: RobotType) -> Self {
        // Configure energy capacity based on robot specialization
        // Different types have different energy profiles optimized for their roles
        let (max_energy, energy) = match robot_type {
            RobotType::Explorer => (80.0, 80.0),           // Balanced capacity for exploration
            RobotType::EnergyCollector => (120.0, 120.0),  // High capacity for extended missions
            RobotType::MineralCollector => (100.0, 100.0), // Good endurance for mining work
            RobotType::ScientificCollector => (60.0, 60.0), // Limited by instrument power needs
        };
        
        // Initialize empty exploration memory grid
        // All tiles start as unexplored from this robot's perspective
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
    
    // Constructeur avec mémoire préchargée (pour la création par la station)
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
    
    // Caractère pour affichage selon le type de robot
    pub fn get_display_char(&self) -> &str {
        match self.robot_type {
            RobotType::Explorer => "🤖",
            RobotType::EnergyCollector => "🔋",
            RobotType::MineralCollector => "⛏️",
            RobotType::ScientificCollector => "🧪",
        }
    }
    
    // Couleur selon le type de robot
    pub fn get_display_color(&self) -> u8 {
        match self.robot_type {
            RobotType::Explorer => 9,          // Rouge vif
            RobotType::EnergyCollector => 10,  // Vert vif
            RobotType::MineralCollector => 13, // Magenta vif
            RobotType::ScientificCollector => 12, // Bleu vif
        }
    }
    
    // Mise à jour de la mémoire (exploration) - VERSION AMÉLIORÉE
    pub fn update_memory(&mut self, map: &Map, station: &Station) {
        let _ = map;
        // Marquer la case actuelle comme explorée avec timestamp
        self.memory[self.y][self.x] = TerrainData {
            explored: true,
            timestamp: station.current_time,
            robot_id: self.id,
            robot_type: self.robot_type,
        };
        
        // L'explorateur a une vision encore plus étendue pour détecter les cases "?"
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
    
    // Méthode principale de mise à jour
    pub fn update(&mut self, map: &mut Map, station: &mut Station) {
        // Consommer de l'énergie (métabolisme de base)
        self.energy -= 0.1;
        
        // Vérifier si l'exploration est terminée (pour les explorateurs uniquement)
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
        
        // Vérifier si le robot doit retourner à la station
        if self.should_return_to_station(map) {
            self.mode = RobotMode::ReturnToStation;
            self.plan_path_to_station(map);
        }
        
        // Pour les collecteurs, vérifier s'il reste des ressources à collecter
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
        
        // Si à la station, recharger, synchroniser et changer de mode
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
        
        // Logique de déplacement selon le mode
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
        
        // Mettre à jour la mémoire
        self.update_memory(map, station);
    }
    
    // Déplacement d'exploration intelligent - VERSION AMÉLIORÉE
    fn explore_move(&mut self, map: &Map) {
        // Pour l'explorateur, utiliser une stratégie plus agressive de recherche de cases non explorées
        if self.robot_type == RobotType::Explorer {
            self.explorer_specific_move(map);
        } else {
            // Logique normale pour les autres types de robots
            self.standard_explore_move(map);
        }
    }
    
    // Nouvelle fonction spécifique pour l'explorateur
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
    
    // Mouvement aléatoire plus intelligent pour l'explorateur
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
    
    // Fonction explore_move originale renommée pour les autres robots
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
    
    // NOUVELLE FONCTION: Trouve la ressource la plus proche dans les zones EXPLORÉES
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
    
    // Collecte de ressources selon le type de robot
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
    
    // Vérifier s'il faut retourner à la station
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
    
    // Planifier un chemin vers la station
    fn plan_path_to_station(&mut self, map: &Map) {
        let target = (self.home_station_x, self.home_station_y);
        self.path_to_station = self.find_path(map, target);
    }
    
    // Trouver la ressource la plus proche selon le type du robot
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
    
    // Algorithme A* pour trouver le chemin optimal
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
    
    // Heuristique pour A* (distance de Manhattan)
    fn heuristic(&self, a: (usize, usize), b: (usize, usize)) -> usize {
        let dx = (a.0 as isize - b.0 as isize).abs() as usize;
        let dy = (a.1 as isize - b.1 as isize).abs() as usize;
        dx + dy
    }
    
    // Déplacement vers une position
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
    
    // Calculer le pourcentage de la carte exploré par ce robot
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
    
    // Vérifier si l'exploration est terminée (100%)
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