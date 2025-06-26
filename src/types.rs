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

/// NOTE - Enum for all possible tile types on the map
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum TileType {
    Empty,      // NOTE - Traversable empty tile
    Obstacle,   // NOTE - Impassable terrain
    Energy,     // NOTE - Energy resource deposit
    Mineral,    // NOTE - Mineral resource deposit
    Scientific, // NOTE - Scientific data point
}

/// NOTE - Enum for robot specialization types
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RobotType {
    Explorer,             // NOTE - General exploration robot
    EnergyCollector,      // NOTE - Energy harvesting robot
    MineralCollector,     // NOTE - Mineral extraction robot
    ScientificCollector,  // NOTE - Scientific data robot
}

/// NOTE - Enum for robot operational modes
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RobotMode {
    Exploring,        // NOTE - Mapping unknown territory
    Collecting,       // NOTE - Gathering resources
    ReturnToStation,  // NOTE - Returning to base
    Idle,             // NOTE - Standby at station
}

/// NOTE - Global constant for map size (square grid)
pub const MAP_SIZE: usize = 20;