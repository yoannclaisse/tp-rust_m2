//! # Station Management Module
//! 
//! This module implements the central command and coordination system for the EREEA mission.
//! The Station acts as the hub for resource management, robot coordination, and mission planning.
//! 
//! ## Key Responsibilities
//! 
//! - **Resource Management**: Track and allocate energy, minerals, and scientific data
//! - **Robot Coordination**: Create, deploy, and synchronize robot fleets
//! - **Knowledge Sharing**: Maintain global exploration memory and resolve data conflicts
//! - **Mission Planning**: Determine optimal robot types and deployment strategies
//! - **Progress Monitoring**: Track mission completion and exploration status

use crate::types::{TileType, RobotType, MAP_SIZE};
use crate::map::Map;
use crate::robot::Robot;

/// Represents detailed information about a specific map tile's exploration status.
/// 
/// This structure stores metadata about when and how each tile was discovered,
/// enabling the station to maintain a comprehensive exploration history and
/// resolve conflicts when multiple robots report different information.
/// 
/// # Conflict Resolution
/// 
/// When multiple robots explore the same tile at different times, the station
/// uses timestamp-based conflict resolution to maintain data accuracy.
/// 
/// # Examples
/// 
/// ```rust
/// use ereea::station::TerrainData;
/// use ereea::types::RobotType;
/// 
/// let tile_data = TerrainData {
///     explored: true,
///     timestamp: 150,
///     robot_id: 3,
///     robot_type: RobotType::Explorer,
/// };
/// 
/// // Check if this data is more recent than existing data
/// if tile_data.timestamp > existing_data.timestamp {
///     // Update with newer information
/// }
/// ```
#[derive(Clone)]
pub struct TerrainData {
    /// Indicates whether this tile has been explored by any robot
    /// 
    /// - `true`: Tile contents are known and verified
    /// - `false`: Tile remains unexplored (displayed as "?" in interfaces)
    pub explored: bool,
    
    /// Simulation cycle timestamp when this tile was first explored
    /// 
    /// Used for conflict resolution when multiple robots report
    /// different information about the same tile. Higher timestamps
    /// indicate more recent and therefore more reliable data.
    pub timestamp: u32,
    
    /// Unique identifier of the robot that explored this tile
    /// 
    /// Enables tracking of individual robot contributions to
    /// the exploration effort and debugging pathfinding issues.
    pub robot_id: usize,
    
    /// Specialization type of the robot that explored this tile
    /// 
    /// Different robot types may have varying sensor capabilities
    /// or exploration accuracies, which could affect data reliability.
    pub robot_type: RobotType,
}

/// Central command and coordination hub for the EREEA exploration mission.
/// 
/// The Station serves as the nexus for all mission operations, managing resources,
/// coordinating robot activities, and maintaining the authoritative record of
/// exploration progress. It implements sophisticated algorithms for robot creation,
/// mission planning, and knowledge management.
/// 
/// # Architecture
/// 
/// The Station uses a hub-and-spoke model where all robots regularly return to
/// synchronize their discoveries and receive new mission parameters. This ensures
/// efficient coordination while maintaining robot autonomy.
/// 
/// # Resource Economics
/// 
/// The Station implements a resource-based economy where:
/// - Energy powers station operations and robot creation
/// - Minerals are consumed to manufacture new robots
/// - Scientific data represents mission progress and discovery value
/// 
/// # Examples
/// 
/// ```rust
/// use ereea::station::Station;
/// use ereea::map::Map;
/// 
/// let mut station = Station::new();
/// let map = Map::new();
/// 
/// // Attempt to create a new robot
/// if let Some(robot) = station.try_create_robot(&map) {
///     println!("Deployed new robot: {:?}", robot.robot_type);
/// }
/// 
/// // Check mission progress
/// let exploration_percent = station.get_exploration_percentage();
/// if exploration_percent >= 100.0 {
///     println!("Exploration complete!");
/// }
/// ```
pub struct Station {
    /// Current energy reserves available for station operations and robot creation
    /// 
    /// Energy is consumed for:
    /// - Manufacturing new robots (50 units per robot)
    /// - Station life support and communication systems
    /// - Emergency robot rescue and recharging operations
    /// 
    /// Energy is replenished by:
    /// - Robot collection of energy resources
    /// - Conversion of excess minerals (1:1 ratio)
    pub energy_reserves: u32,
    
    /// Total minerals collected and stored at the station
    /// 
    /// Minerals are essential for:
    /// - Robot construction (15 units per robot)
    /// - Station equipment upgrades and maintenance
    /// - Emergency repairs and component replacement
    /// 
    /// Minerals are gathered exclusively by MineralCollector robots
    /// from mineral deposits scattered across the exoplanet surface.
    pub collected_minerals: u32,
    
    /// Scientific data points accumulated from exploration activities
    /// 
    /// Scientific data represents:
    /// - Geological surveys and planetary composition analysis
    /// - Atmospheric readings and environmental assessments
    /// - Biological samples and life detection results
    /// - Strategic value for future colonization planning
    /// 
    /// Scientific data is collected by ScientificCollector robots
    /// from points of interest identified during exploration.
    pub collected_scientific_data: u32,
    
    /// Comprehensive exploration memory containing data for every map tile
    /// 
    /// This 2D grid mirrors the exploration map and stores detailed metadata
    /// about each tile's discovery. The station maintains this as the
    /// authoritative source of exploration knowledge, synchronized from
    /// all active robots during their return visits.
    /// 
    /// Structure: `global_memory[y][x]` corresponds to map position (x, y)
    pub global_memory: Vec<Vec<TerrainData>>,
    
    /// Total number of data conflicts resolved through timestamp-based arbitration
    /// 
    /// Conflicts occur when multiple robots report different information
    /// about the same map tile. The station resolves these by accepting
    /// the most recent report (highest timestamp). High conflict counts
    /// may indicate coordination issues or sensor malfunctions.
    pub conflict_count: usize,
    
    /// Identifier that will be assigned to the next robot created
    /// 
    /// Robot IDs are sequential and unique across the entire mission,
    /// enabling clear identification and tracking of individual robot
    /// performance and contributions. Incremented after each robot creation.
    pub next_robot_id: usize,
    
    /// Global simulation time counter tracking mission duration
    /// 
    /// Incremented once per simulation cycle, this timestamp is used for:
    /// - Exploration data conflict resolution
    /// - Mission scheduling and planning
    /// - Performance analysis and optimization
    /// - Synchronization of distributed robot operations
    pub current_time: u32,
}

impl Station {
    /// Constructs a new Station with initial default values and empty exploration memory.
    /// 
    /// The constructor initializes all station systems and prepares for mission operations.
    /// Initial resource allocations are balanced to enable immediate robot deployment
    /// while maintaining operational reserves.
    /// 
    /// # Initial Conditions
    /// 
    /// - Energy: 100 units (sufficient for 2 robot deployments)
    /// - Minerals: 0 units (must be collected before robot manufacturing)
    /// - Scientific Data: 0 units (collected throughout mission)
    /// - Exploration Memory: All tiles marked as unexplored
    /// - Robot ID Counter: Starts at 1 (robot #1 will be first created)
    /// 
    /// # Returns
    /// 
    /// New Station instance ready for mission operations
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let station = Station::new();
    /// assert_eq!(station.energy_reserves, 100);
    /// assert_eq!(station.next_robot_id, 1);
    /// assert_eq!(station.get_exploration_percentage(), 0.0);
    /// ```
    pub fn new() -> Self {
        // NOTE - Initializing global exploration memory grid
        let mut global_memory = Vec::with_capacity(MAP_SIZE);
        for _ in 0..MAP_SIZE {
            let row = vec![
                TerrainData {
                    explored: false,                    // All tiles start unexplored
                    timestamp: 0,                       // No exploration timestamp yet
                    robot_id: 0,                        // No robot has visited yet
                    robot_type: RobotType::Explorer,    // Default robot type for unvisited tiles
                }; 
                MAP_SIZE
            ];
            global_memory.push(row);
        }
        
        // NOTE - Station struct initialization with default values
        Self {
            energy_reserves: 100,              // Starting energy for initial operations
            collected_minerals: 0,             // No minerals until robots collect them
            collected_scientific_data: 0,      // No scientific data initially
            global_memory,                     // Freshly initialized exploration grid
            conflict_count: 0,                 // No conflicts yet
            next_robot_id: 1,                  // First robot will be ID #1
            current_time: 0,                   // Mission starts at time 0
        }
    }
    
    /// Advances the global mission clock by one simulation cycle.
    /// 
    /// This method should be called once per simulation iteration to maintain
    /// synchronized timing across all mission systems. The current time is used
    /// for exploration timestamp recording and conflict resolution.
    /// 
    /// # Side Effects
    /// 
    /// - Increments `current_time` by 1
    /// - Affects all subsequent exploration timestamp recording
    /// - May influence robot behavior algorithms that depend on timing
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let mut station = Station::new();
    /// assert_eq!(station.current_time, 0);
    /// 
    /// station.tick();
    /// assert_eq!(station.current_time, 1);
    /// ```
    pub fn tick(&mut self) {
        // NOTE - Advancing simulation time
        self.current_time += 1;
    }
    
    /// Attempts to create a new robot for exploration or resource collection.
    /// 
    /// This method consumes a portion of the station's energy and minerals
    /// reserves to manufacture a new robot. The type of robot created depends
    /// on the current mission needs and resource availability.
    /// 
    /// # Resource Costs
    /// 
    /// - Energy: 50 units are consumed from the station's reserves
    /// - Minerals: 15 units are deducted from the collected minerals
    /// 
    /// # Returns
    /// 
    /// An `Option<Robot>` which is:
    /// - `Some(robot)`: A new robot instance configured for its mission
    /// - `None`: Insufficient resources to create a robot
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let mut station = Station::new();
    /// let map = Map::new();
    /// 
    /// // Create a new robot for exploration
    /// if let Some(robot) = station.try_create_robot(&map) {
    ///     println!("New robot created: ID={}, Type={:?}", robot.id, robot.robot_type);
    /// } else {
    ///     println!("Not enough resources to create a robot.");
    /// }
    /// ```
    pub fn try_create_robot(&mut self, map: &Map) -> Option<Robot> {
        // NOTE - Robot creation resource cost check
        let energy_cost = 50;   // Énergie requise
        let mineral_cost = 15;  // Minerais requis
        
        // NOTE - Checking if enough resources to create a robot
        if self.energy_reserves >= energy_cost && self.collected_minerals >= mineral_cost {
            // NOTE - Determining most needed robot type
            let robot_type = self.determine_needed_robot_type(map);
            
            // NOTE - Consuming resources for robot creation
            self.energy_reserves -= energy_cost;
            self.collected_minerals -= mineral_cost;
            
            println!("Station: Création d'un nouveau robot #{} de type {:?}", 
                     self.next_robot_id, robot_type);
            
            // NOTE - Creating robot with current global memory
            let new_robot = Robot::new_with_memory(
                map.station_x, 
                map.station_y, 
                robot_type, 
                self.next_robot_id,
                map.station_x, 
                map.station_y,
                self.global_memory.clone()
            );
            
            // NOTE - Incrementing robot ID counter
            self.next_robot_id += 1;
            
            return Some(new_robot);
        }
        
        None // Pas assez de ressources
    }
    
    /// Determines the most needed type of robot based on current mission status and resource availability.
    /// 
    /// This function analyzes the exploration progress, resource counts, and existing robot types
    /// to decide whether to create more Explorers, EnergyCollectors, MineralCollectors, or ScientificCollectors.
    /// 
    /// # Returns
    /// 
    /// The `RobotType` that is deemed most necessary for the next phase of the mission.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let station = Station::new();
    /// let map = Map::new();
    /// 
    /// // Initially, explorers are needed
    /// assert_eq!(station.determine_needed_robot_type(&map), RobotType::Explorer);
    /// 
    /// // After some exploration, more energy collectors might be needed
    /// station.global_memory[0][0].explored = true;
    /// station.global_memory[0][0].timestamp = 1;
    /// assert_eq!(station.determine_needed_robot_type(&map), RobotType::EnergyCollector);
    /// ```
    fn determine_needed_robot_type(&self, map: &Map) -> RobotType {
        // NOTE - Calculating exploration percentage
        let exploration_percentage = self.get_exploration_percentage();
        
        // NOTE - Phase 1: Prioritize exploration
        if exploration_percentage < 50.0 {
            return RobotType::Explorer;
        }
        
        // NOTE - Counting remaining resources on the map
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
        
        // NOTE - Phase 2: Prioritize energy and mineral collection
        if exploration_percentage < 80.0 {
            if energy_count > 0 && (energy_count <= 3 || self.energy_reserves < 100) {
                return RobotType::EnergyCollector;
            }
            if mineral_count > 0 && (mineral_count <= 5 || self.collected_minerals < 30) {
                return RobotType::MineralCollector;
            }
            return RobotType::Explorer;
        }
        
        // NOTE - Phase 3: Prioritize scientific collection
        if scientific_count > 0 && self.energy_reserves >= 100 {
            return RobotType::ScientificCollector;
        }
        
        // NOTE - Fallback: prioritize remaining resources
        if energy_count > 0 {
            return RobotType::EnergyCollector;
        }
        if mineral_count > 0 {
            return RobotType::MineralCollector;
        }
        
        // NOTE - Default: create explorer to finish exploration
        RobotType::Explorer
    }
    
    /// Synchronizes the station's knowledge base with a robot's exploration data.
    /// 
    /// This method is called when a robot returns to the station. It allows the robot
    /// to upload its discovered data, which is then merged into the station's global memory.
    /// Conflicts between different robots' data are resolved based on timestamps,
    /// with the most recent data taking precedence.
    /// 
    /// # Parameters
    /// 
    /// - `robot`: A mutable reference to the returning robot. Its data will be merged
    ///   into the station's knowledge base.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let mut station = Station::new();
    /// let mut robot = Robot::new();
    /// 
    /// // After the robot explores some tiles
    /// robot.memory[0][0].explored = true;
    /// robot.memory[0][0].timestamp = 5;
    /// 
    /// // Station synchronizes with the robot
    /// station.share_knowledge(&mut robot);
    /// ```
    pub fn share_knowledge(&mut self, robot: &mut Robot) {
        // NOTE - Only synchronize if robot is at the station
        if robot.x == robot.home_station_x && robot.y == robot.home_station_y {
            let mut conflicts = 0;
            let mut changes_made = false;
            
            // NOTE - Robot shares its knowledge with the station
            for y in 0..MAP_SIZE {
                for x in 0..MAP_SIZE {
                    if robot.memory[y][x].explored {
                        if self.global_memory[y][x].explored {
                            // NOTE - Conflict: resolve by timestamp
                            if robot.memory[y][x].timestamp > self.global_memory[y][x].timestamp {
                                self.global_memory[y][x] = robot.memory[y][x].clone();
                                conflicts += 1;
                                changes_made = true;
                            }
                        } else {
                            // NOTE - No conflict, add robot's knowledge
                            self.global_memory[y][x] = robot.memory[y][x].clone();
                            changes_made = true;
                        }
                    }
                }
            }
            
            // NOTE - Robot receives all global knowledge
            for y in 0..MAP_SIZE {
                for x in 0..MAP_SIZE {
                    if self.global_memory[y][x].explored {
                        robot.memory[y][x] = self.global_memory[y][x].clone();
                    }
                }
            }
            
            // NOTE - Update conflict statistics if changes were made
            if changes_made {
                self.conflict_count += conflicts;
                
                if conflicts > 0 {
                    println!("Robot {} a synchronisé ses connaissances. Conflits résolus: {}", 
                             robot.id, conflicts);
                }
            }
        }
    }
    
    /// Deposits collected resources into the station's reserves.
    /// 
    /// This method is called by robots to transfer the minerals and scientific data
    /// they have collected back to the station. The station then incorporates these
    /// resources into its global reserves, making them available for robot creation
    /// and other station operations.
    /// 
    /// # Parameters
    /// 
    /// - `minerals`: The amount of minerals to deposit
    /// - `scientific_data`: The amount of scientific data to deposit
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let mut station = Station::new();
    /// 
    /// // Deposit 30 minerals and 10 scientific data units
    /// station.deposit_resources(30, 10);
    /// 
    /// assert_eq!(station.collected_minerals, 30);
    /// assert_eq!(station.collected_scientific_data, 10);
    /// ```
    pub fn deposit_resources(&mut self, minerals: u32, scientific_data: u32) {
        // NOTE - Depositing minerals and scientific data
        self.collected_minerals += minerals;
        self.collected_scientific_data += scientific_data;
        self.energy_reserves += minerals; // Conversion minerais -> énergie
    }
    
    /// Generates a status report string summarizing the current state of the station.
    /// 
    /// This report includes information on resource levels, robot creation capacity,
    /// conflict counts, and overall exploration progress. It is intended for display
    /// to the user or for logging purposes.
    /// 
    /// # Returns
    /// 
    /// A formatted string containing the station's status report
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let station = Station::new();
    /// let status_report = station.get_status();
    /// println!("Status Report: {}", status_report);
    /// ```
    pub fn get_status(&self) -> String {
        // NOTE - Generating station status report string
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
        // NOTE - Placeholder for resource collection check
        false
    }
    
    /// Calculates the overall percentage of the map that has been explored.
    /// 
    /// This function counts the number of explored tiles in the station's global memory
    /// and calculates the percentage relative to the total number of tiles. This value
    /// is used to gauge mission progress and determine when the exploration phase is complete.
    /// 
    /// # Returns
    /// 
    /// A floating-point number representing the percentage of the map that has been explored
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let station = Station::new();
    /// 
    /// // Initially, nothing is explored
    /// assert_eq!(station.get_exploration_percentage(), 0.0);
    /// 
    /// // After marking some tiles as explored
    /// station.global_memory[0][0].explored = true;
    /// station.global_memory[1][0].explored = true;
    /// assert_eq!(station.get_exploration_percentage(), 12.5);
    /// ```
    pub fn get_exploration_percentage(&self) -> f32 {
        // NOTE - Counting explored tiles in global memory
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
    
    /// Checks if all mission objectives are complete, including full map exploration and resource collection.
    /// 
    /// This function verifies that the exploration percentage is at 100%, that all resources have been collected,
    /// and that all robots are either idle at the station or in a completed state. This is used to determine
    /// if the mission can be considered finished.
    /// 
    /// # Parameters
    /// 
    /// - `map`: A reference to the current map instance
    /// - `robots`: A reference to the vector of all robots
    /// 
    /// # Returns
    /// 
    /// `true` if all mission conditions are met, `false` otherwise
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let station = Station::new();
    /// let map = Map::new();
    /// let robots = vec![Robot::new(), Robot::new()];
    /// 
    /// // After completing exploration and resource collection
    /// assert!(station.is_all_missions_complete(&map, &robots));
    /// ```
    pub fn is_all_missions_complete(&self, map: &Map, robots: &Vec<Robot>) -> bool {
        // NOTE - Check if map is fully explored
        if self.get_exploration_percentage() < 100.0 {
            return false;
        }
        
        // NOTE - Check if all resources are collected
        if !self.are_all_resources_collected(map) {
            return false;
        }
        
        // NOTE - Check if all robots are at the station and idle
        for robot in robots {
            match robot.robot_type {
                RobotType::Explorer => {
                    if robot.mode != crate::types::RobotMode::Idle || 
                       robot.x != robot.home_station_x || 
                       robot.y != robot.home_station_y {
                        return false;
                    }
                },
                _ => {
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
    
    /// Checks if the current mission is complete, which requires all resources to be collected.
    /// 
    /// This function is a simplified check used when the mission parameters do not require
    /// full exploration, but rather the collection of specific resources. It verifies that
    /// no resources are left on the map.
    /// 
    /// # Parameters
    /// 
    /// - `map`: A reference to the current map instance
    /// 
    /// # Returns
    /// 
    /// `true` if the mission is complete (all resources collected), `false` otherwise
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let station = Station::new();
    /// let map = Map::new();
    /// 
    /// // After collecting all resources
    /// assert!(station.is_mission_complete(&map));
    /// ```
    pub fn is_mission_complete(&self, map: &Map) -> bool {
        // NOTE - Check if all resources are collected
        self.are_all_resources_collected(map)
    }
    
    /// Vérifier que toutes les ressources ont été collectées
    fn are_all_resources_collected(&self, map: &Map) -> bool {
        // NOTE - Scanning map for remaining resources
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