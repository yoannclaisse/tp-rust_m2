//! # EREEA Types Module
//! 
//! This module defines all the core data types used throughout the EREEA (Exploration Robotique 
//! d'Exoplanètes Autonome) simulation system. These types represent the fundamental building 
//! blocks of the exoplanet exploration simulation.
//! 
//! ## Key Components
//! 
//! - **TileType**: Represents different terrain and resource types on the exploration map
//! - **RobotType**: Defines the specialization categories for exploration robots
//! - **RobotMode**: Describes the current behavioral state of robots
//! - **MAP_SIZE**: Global constant defining the dimensions of the exploration grid
//! 
//! All types are serializable for network transmission between simulation server and Earth control.

use serde::{Serialize, Deserialize};

/// Represents the different types of terrain tiles found on the exoplanet surface.
/// 
/// Each tile type has specific gameplay implications:
/// - Empty tiles are traversable but provide no resources
/// - Obstacles block robot movement and pathfinding
/// - Resource tiles (Energy, Mineral, Scientific) can be collected by specialized robots
/// 
/// # Serialization
/// 
/// This enum is serializable to enable network transmission of map data between
/// the simulation server and Earth monitoring stations.
/// 
/// # Examples
/// 
/// ```rust
/// use ereea::types::TileType;
/// 
/// let resource = TileType::Energy;
/// let traversable = matches!(resource, TileType::Empty | TileType::Energy | TileType::Mineral | TileType::Scientific);
/// assert_eq!(traversable, true);
/// ```
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum TileType {
    /// Empty traversable space with no special properties or resources
    /// Robots can move through these tiles freely without energy penalties
    Empty,
    
    /// Impassable terrain that blocks robot movement and pathfinding algorithms
    /// Represents rocky outcrops, crevasses, or other geological hazards
    Obstacle,
    
    /// Energy resource deposits that can be harvested by EnergyCollector robots
    /// Provides renewable power for station operations and robot recharging
    Energy,
    
    /// Mineral resource veins that can be extracted by MineralCollector robots  
    /// Essential for manufacturing new robots and upgrading station capabilities
    Mineral,
    
    /// Points of scientific interest containing valuable research data
    /// Can be analyzed by ScientificCollector robots to advance mission objectives
    Scientific,
}

/// Defines the specialized roles and capabilities of exploration robots.
/// 
/// Each robot type has unique characteristics:
/// - Different energy capacities and consumption rates
/// - Specialized equipment for specific resource types
/// - Varied movement speeds and operational ranges
/// - Distinct behavioral algorithms and priorities
/// 
/// # Design Philosophy
/// 
/// The specialization system encourages strategic deployment of different robot types
/// based on mission phase, available resources, and exploration priorities.
/// 
/// # Examples
/// 
/// ```rust
/// use ereea::types::RobotType;
/// 
/// let scout = RobotType::Explorer;
/// let harvester = RobotType::EnergyCollector;
/// 
/// // Different robots have different energy capacities
/// let energy_capacity = match scout {
///     RobotType::Explorer => 80.0,
///     RobotType::EnergyCollector => 120.0,
///     _ => 60.0,
/// };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RobotType {
    /// General-purpose exploration robot optimized for mapping and reconnaissance
    /// 
    /// **Capabilities:**
    /// - Extended sensor range for detecting unexplored areas
    /// - Moderate energy capacity (80 units)
    /// - Balanced movement speed
    /// - Advanced pathfinding algorithms for efficient exploration
    Explorer,
    
    /// Specialized robot designed for harvesting and transporting energy resources
    /// 
    /// **Capabilities:**
    /// - High energy capacity (120 units) for extended operations
    /// - Efficient energy collection and storage systems
    /// - Can recharge itself from energy deposits
    /// - Optimized for energy deposit detection and extraction
    EnergyCollector,
    
    /// Heavy-duty robot equipped for mineral extraction and processing
    /// 
    /// **Capabilities:**
    /// - Robust construction for harsh mining conditions
    /// - Good energy capacity (100 units)
    /// - Advanced drilling and extraction equipment
    /// - Can carry multiple mineral units simultaneously
    MineralCollector,
    
    /// Precision instrument platform for scientific data collection and analysis
    /// 
    /// **Capabilities:**
    /// - Sensitive scientific instruments and sensors
    /// - Lower energy capacity (60 units) due to instrument power requirements
    /// - Specialized data processing and storage systems
    /// - High-precision movement for delicate operations
    ScientificCollector,
}

/// Represents the current operational mode and behavioral state of a robot.
/// 
/// The mode system controls robot decision-making and determines which behaviors
/// and algorithms are active. Modes can change automatically based on conditions
/// like energy levels, inventory status, or mission objectives.
/// 
/// # State Transitions
/// 
/// Typical mode transitions:
/// - Exploring → Collecting (when target resource found)
/// - Collecting → ReturnToStation (when inventory full or energy low)
/// - ReturnToStation → Idle (when arrived at station)
/// - Idle → Exploring (after resupply and mission planning)
/// 
/// # Examples
/// 
/// ```rust
/// use ereea::types::RobotMode;
/// 
/// let mut robot_mode = RobotMode::Exploring;
/// 
/// // Simulate discovering a resource
/// if resource_detected {
///     robot_mode = RobotMode::Collecting;
/// }
/// 
/// // Check if should return to base
/// if energy_low || inventory_full {
///     robot_mode = RobotMode::ReturnToStation;
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RobotMode {
    /// Active exploration mode - robot is mapping unknown territory
    /// 
    /// **Behaviors:**
    /// - Seeks unexplored map tiles marked with "?"
    /// - Uses intelligent pathfinding to maximize coverage
    /// - Updates local and station memory with discoveries
    /// - Automatically detects and evaluates resources
    Exploring,
    
    /// Resource collection mode - robot is gathering specific materials
    /// 
    /// **Behaviors:**
    /// - Moves toward known resource deposits matching robot specialization
    /// - Extracts resources when positioned on appropriate tiles
    /// - Manages inventory capacity and load balancing
    /// - Continues collecting until inventory full or energy depleted
    Collecting,
    
    /// Return-to-base mode - robot is navigating back to the station
    /// 
    /// **Behaviors:**
    /// - Uses A* pathfinding algorithm for optimal route to station
    /// - Prioritizes energy conservation during return journey
    /// - Avoids unnecessary detours and resource collection
    /// - Automatically docks and transfers resources upon arrival
    ReturnToStation,
    
    /// Inactive mode - robot is stationed and awaiting new orders
    /// 
    /// **Behaviors:**
    /// - Remains at station for energy recharging and maintenance
    /// - Synchronizes exploration data with central station memory
    /// - Awaits resource availability or new mission objectives
    /// - Minimal energy consumption for standby operations
    Idle,
}

/// Global constant defining the dimensions of the exploration map grid.
/// 
/// The map is square with MAP_SIZE × MAP_SIZE tiles. This size represents a balance between:
/// - Computational performance for pathfinding and simulation
/// - Visual display capabilities in terminal interfaces
/// - Mission complexity and exploration time requirements
/// 
/// # Usage Considerations
/// 
/// - All coordinate systems use 0-based indexing from (0,0) to (MAP_SIZE-1, MAP_SIZE-1)
/// - Memory allocation for map-based data structures uses MAP_SIZE² elements
/// - Pathfinding algorithms scale with O(MAP_SIZE²) complexity
/// - Network transmission includes MAP_SIZE² tile data per update
/// 
/// # Examples
/// 
/// ```rust
/// use ereea::types::MAP_SIZE;
/// 
/// // Create a 2D vector for map data
/// let mut map_grid = vec![vec![false; MAP_SIZE]; MAP_SIZE];
/// 
/// // Validate coordinates
/// fn is_valid_position(x: usize, y: usize) -> bool {
///     x < MAP_SIZE && y < MAP_SIZE
/// }
/// 
/// // Calculate total map area
/// let total_tiles = MAP_SIZE * MAP_SIZE; // 400 tiles for 20x20 map
/// ```
pub const MAP_SIZE: usize = 20;