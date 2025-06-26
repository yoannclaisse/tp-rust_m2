//! # Network Communication Protocol Module
//! 
//! This module implements the communication protocol between the EREEA simulation server
//! and Earth-based monitoring stations. It provides data serialization, transmission
//! structures, and utility functions for real-time mission monitoring.
//! 
//! ## Protocol Architecture
//! 
//! The protocol uses JSON-based serialization over TCP connections for:
//! - Real-time simulation state transmission
//! - Cross-platform compatibility  
//! - Human-readable debugging and monitoring
//! - Efficient bandwidth utilization through delta compression
//! 
//! ## Data Structures
//! 
//! All network types are serializable and contain complete mission state information:
//! - Map data with terrain and resource distributions
//! - Individual robot status and performance metrics
//! - Station operational data and resource management
//! - Exploration progress and discovery tracking

// Module imports for internal types and serialization
use serde::{Serialize, Deserialize};
use crate::types::{MAP_SIZE, TileType, RobotType, RobotMode};

/// Network-serializable representation of the exploration map data.
/// 
/// This structure contains all information necessary to reconstruct the
/// exploration map on remote monitoring systems. It includes terrain layout,
/// resource distributions, and station positioning data.
/// 
/// # Serialization Format
/// 
/// The map data is serialized as JSON with nested arrays representing
/// the 2D tile grid. This format is human-readable and cross-platform
/// compatible while maintaining reasonable bandwidth efficiency.
/// 
/// # Examples
/// 
/// ```rust
/// use ereea::network::MapData;
/// use ereea::types::TileType;
/// 
/// let map_data = MapData {
///     tiles: vec![vec![TileType::Empty; MAP_SIZE]; MAP_SIZE],
///     station_x: 10,
///     station_y: 10,
/// };
/// 
/// // Serialize for network transmission
/// let json = serde_json::to_string(&map_data)?;
/// ```
#[derive(Serialize, Deserialize, Clone)]
pub struct MapData {
    /// Complete 2D grid of tile types representing the exploration map
    /// 
    /// Structure: `tiles[y][x]` corresponds to map position (x, y)
    /// Contains all terrain types, resources, and obstacles as they
    /// currently exist on the map (resources may be consumed over time)
    pub tiles: Vec<Vec<TileType>>,
    
    /// X coordinate of the central station facility
    /// 
    /// Represents the hub location where robots are manufactured,
    /// resources are stored, and mission coordination occurs.
    /// Used by monitoring systems to highlight the station position.
    pub station_x: usize,
    
    /// Y coordinate of the central station facility
    pub station_y: usize,
}

/// Network-serializable representation of individual robot status and performance.
/// 
/// This structure contains comprehensive information about a single robot's
/// current state, including position, energy, inventory, and operational mode.
/// Transmitted for each active robot to enable detailed fleet monitoring.
/// 
/// # Performance Tracking
/// 
/// The robot data enables Earth-based analysts to:
/// - Monitor individual robot health and energy status
/// - Track exploration contributions and resource collection efficiency
/// - Identify robots requiring assistance or optimization
/// - Plan fleet deployment and specialization strategies
/// 
/// # Examples
/// 
/// ```rust
/// use ereea::network::RobotData;
/// use ereea::types::{RobotType, RobotMode};
/// 
/// let robot_status = RobotData {
///     id: 3,
///     x: 15, y: 8,
///     energy: 45.5, max_energy: 80.0,
///     minerals: 2, scientific_data: 1,
///     robot_type: RobotType::Explorer,
///     mode: RobotMode::Exploring,
///     exploration_percentage: 25.3,
/// };
/// ```
#[derive(Serialize, Deserialize, Clone)]
pub struct RobotData {
    /// Unique identifier for this robot across the entire mission
    /// 
    /// Robot IDs are sequential and permanent, allowing long-term
    /// performance tracking and historical analysis of individual
    /// robot contributions to the mission success.
    pub id: usize,
    
    /// Current X coordinate position on the exploration map
    pub x: usize,
    
    /// Current Y coordinate position on the exploration map
    pub y: usize,
    
    /// Current energy level (0.0 = depleted, max_energy = fully charged)
    /// 
    /// Critical for monitoring robot health and predicting when
    /// robots will need to return to station for recharging.
    /// Low energy levels may indicate maintenance needs or inefficient operations.
    pub energy: f32,
    
    /// Maximum energy capacity for this robot type
    /// 
    /// Different robot specializations have varying energy capacities
    /// optimized for their operational requirements and mission profiles.
    pub max_energy: f32,
    
    /// Number of mineral units currently carried by the robot
    /// 
    /// Only meaningful for MineralCollector robots. High values indicate
    /// successful mining operations but may slow robot movement speed.
    /// Zero for non-mining robot types.
    pub minerals: u32,
    
    /// Number of scientific data units currently stored by the robot
    /// 
    /// Only meaningful for ScientificCollector robots. Represents
    /// completed analysis of points of scientific interest and contributes
    /// to overall mission scientific objectives.
    pub scientific_data: u32,
    
    /// Robot specialization type determining capabilities and behavior
    /// 
    /// Used by monitoring systems to:
    /// - Apply appropriate color coding and visual representation
    /// - Understand expected behavior patterns and performance metrics
    /// - Plan fleet composition and deployment strategies
    pub robot_type: RobotType,
    
    /// Current operational mode controlling robot behavior
    /// 
    /// Indicates the robot's current activity and decision-making state:
    /// - Exploring: Actively mapping unknown territory
    /// - Collecting: Gathering resources matching specialization
    /// - ReturnToStation: Navigating back to base for resupply
    /// - Idle: Standby mode awaiting new missions or resources
    pub mode: RobotMode,
    
    /// Percentage of the map this robot has personally explored
    /// 
    /// Individual exploration metric enabling assessment of robot
    /// contribution to overall mission progress. High values indicate
    /// effective exploration patterns and pathfinding algorithms.
    pub exploration_percentage: f32,
}

/// Network-serializable representation of central station status and operations.
/// 
/// This structure contains comprehensive information about the mission's central
/// command facility, including resource stockpiles, operational metrics, and
/// mission progress indicators. Essential for mission planning and resource allocation.
/// 
/// # Mission Control Integration
/// 
/// Station data enables Earth-based mission control to:
/// - Monitor resource availability for continued operations
/// - Plan robot deployment based on current capabilities
/// - Track overall mission progress toward completion
/// - Identify operational inefficiencies or optimization opportunities
/// 
/// # Examples
/// 
/// ```rust
/// use ereea::network::StationData;
/// 
/// let station_status = StationData {
///     energy_reserves: 150,
///     collected_minerals: 45,
///     collected_scientific_data: 12,
///     exploration_percentage: 67.5,
///     conflict_count: 3,
///     robot_count: 6,
///     status_message: "Phase 2: Resource Collection".to_string(),
///     mission_complete: false,
/// };
/// ```
#[derive(Serialize, Deserialize, Clone)]
pub struct StationData {
    /// Current energy reserves available for station operations
    /// 
    /// Energy is consumed for:
    /// - Manufacturing new robots (50 units per robot)
    /// - Station life support and communication systems
    /// - Emergency operations and robot rescue missions
    /// 
    /// Low energy reserves may limit operational capabilities.
    pub energy_reserves: u32,
    
    /// Total mineral units collected and stored at the station
    /// 
    /// Minerals are essential for:
    /// - Robot construction (15 units per robot)
    /// - Station equipment upgrades and maintenance
    /// - Advanced manufacturing and fabrication operations
    /// 
    /// Mineral stockpiles enable expanded robot deployment.
    pub collected_minerals: u32,
    
    /// Total scientific data points accumulated from exploration
    /// 
    /// Scientific data represents:
    /// - Completed analysis of geological samples
    /// - Environmental surveys and atmospheric readings
    /// - Biological detection and life-form investigations
    /// - Strategic assessments for future colonization
    /// 
    /// High scientific data values indicate mission success.
    pub collected_scientific_data: u32,
    
    /// Percentage of the exoplanet map that has been explored
    /// 
    /// Global exploration metric combining discoveries from all robots.
    /// 100% exploration indicates complete planetary mapping and
    /// readiness for colonization planning phases.
    pub exploration_percentage: f32,
    
    /// Number of data conflicts resolved through timestamp arbitration
    /// 
    /// Conflicts occur when multiple robots report different information
    /// about the same map location. High conflict counts may indicate:
    /// - Coordination issues requiring algorithm optimization
    /// - Environmental hazards affecting sensor accuracy
    /// - Communication delays or synchronization problems
    pub conflict_count: usize,
    
    /// Total number of robots currently active in the mission
    /// 
    /// Includes all deployed robots regardless of current operational status.
    /// Growing robot counts indicate successful resource management and
    /// expanding operational capabilities.
    pub robot_count: usize,
    
    /// Human-readable status message describing current mission phase
    /// 
    /// Provides contextual information about current operations:
    /// - "Phase 1: Initial Exploration" (0-30% exploration)
    /// - "Phase 2: Resource Collection" (30-80% exploration)  
    /// - "Phase 3: Scientific Analysis" (80-100% exploration)
    /// - "Mission Complete" (all objectives achieved)
    pub status_message: String,
    
    /// Boolean flag indicating whether all mission objectives are complete
    /// 
    /// True when:
    /// - 100% exploration has been achieved
    /// - All available resources have been collected
    /// - All robots have returned safely to the station
    /// - Mission is ready for termination and data analysis
    pub mission_complete: bool,
}

/// Network-serializable representation of explored tiles.
/// Used to transmit which tiles have been explored by the station.
#[derive(Serialize, Deserialize, Clone)]
pub struct ExplorationData {
    /// 2D grid: true if tile has been explored, false otherwise.
    pub explored_tiles: Vec<Vec<bool>>,
}

/// Complete simulation state for network transmission.
/// Bundles all relevant data for a single simulation tick.
#[derive(Serialize, Deserialize, Clone)]
pub struct SimulationState {
    pub map_data: MapData,
    pub robots_data: Vec<RobotData>,
    pub station_data: StationData,
    pub exploration_data: ExplorationData,
    pub iteration: u32,
}

/// Global network configuration constants for reliable communication.
/// 
/// These constants define the communication parameters used throughout
/// the EREEA network protocol to ensure consistent and reliable data
/// transmission between simulation and monitoring systems.

/// Default TCP port for EREEA simulation server communication
/// 
/// Port 8080 is chosen for:
/// - Common availability on most systems
/// - Easy firewall configuration
/// - Compatibility with development environments
/// - Standard practice for application servers
/// 
/// Clients should connect to `localhost:8080` when running locally
pub const DEFAULT_PORT: u16 = 8080;

/// Maximum allowed size for network message transmission (1 megabyte)
/// 
/// This limit prevents:
/// - Memory exhaustion from malformed or excessive data
/// - Network congestion from oversized transmissions
/// - Buffer overflow vulnerabilities in client applications
/// - Performance degradation from inefficient serialization
/// 
/// Current simulation data typically uses 10-50KB per transmission
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

// Fonction utilitaire : convertir Map vers MapData pour transmission réseau
pub fn create_map_data(map: &crate::map::Map) -> MapData {
    MapData {
        tiles: map.tiles.clone(),           // Copie de la grille des tuiles
        station_x: map.station_x,
        station_y: map.station_y,
    }
}

/// Fonction utilitaire : convertir Robot vers RobotData pour transmission réseau
pub fn create_robot_data(robot: &crate::robot::Robot) -> RobotData {
    RobotData {
        id: robot.id,
        x: robot.x,
        y: robot.y,
        energy: robot.energy,
        max_energy: robot.max_energy,
        minerals: robot.minerals,
        scientific_data: robot.scientific_data,
        robot_type: robot.robot_type,
        mode: robot.mode,
        exploration_percentage: robot.get_exploration_percentage(),
    }
}

/// Fonction utilitaire : convertir Station vers StationData pour transmission réseau
pub fn create_station_data(station: &crate::station::Station, map: &crate::map::Map) -> StationData {
    StationData {
        energy_reserves: station.energy_reserves,
        collected_minerals: station.collected_minerals,
        collected_scientific_data: station.collected_scientific_data,
        exploration_percentage: station.get_exploration_percentage(),
        conflict_count: station.conflict_count,
        robot_count: station.next_robot_id - 1,    // Estimation du nombre de robots
        status_message: station.get_status(),
        mission_complete: station.is_mission_complete(map),
    }
}

/// Fonction utilitaire : créer les données d'exploration pour transmission réseau
pub fn create_exploration_data(station: &crate::station::Station) -> ExplorationData {
    let mut explored_tiles = vec![vec![false; MAP_SIZE]; MAP_SIZE];
    
    // Convertir la mémoire complexe de la station en simple grille booléenne
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            explored_tiles[y][x] = station.global_memory[y][x].explored;
        }
    }
    
    ExplorationData {
        explored_tiles,
    }
}

/// Fonction principale : créer l'état complet de simulation pour transmission
pub fn create_simulation_state(
    map: &crate::map::Map, 
    station: &crate::station::Station, 
    robots: &Vec<crate::robot::Robot>, 
    iteration: u32
) -> SimulationState {
    // Convertir les données de la carte
    let map_data = create_map_data(map);
    
    // Convertir les données de tous les robots
    let mut robots_data = Vec::with_capacity(robots.len());
    for robot in robots {
        robots_data.push(create_robot_data(robot));
    }
    
    // Convertir les données de la station (avec la référence à map)
    let station_data = create_station_data(station, map);
    
    // Convertir les données d'exploration
    let exploration_data = create_exploration_data(station);
    
    // Assembler l'état complet
    SimulationState {
        map_data,
        robots_data,
        station_data,
        exploration_data,
        iteration,
    }
}