//! # Procedural Map Generation and Management Module
//! 
//! This module handles the creation and management of the exoplanet exploration map.
//! It uses advanced procedural generation techniques to create realistic, explorable
//! terrain with balanced resource distribution and guaranteed accessibility.
//! 
//! ## Generation Algorithm
//! 
//! The map generation uses Perlin noise for natural terrain distribution combined
//! with accessibility algorithms to ensure all resources can be reached by robots.
//! 
//! ## Features
//! 
//! - **Procedural Generation**: Each map is unique using random seeds
//! - **Resource Balance**: Controlled distribution of energy, minerals, and science points
//! - **Accessibility Guarantee**: All resources are reachable from the station
//! - **Obstacle Placement**: Natural-looking terrain barriers and passages

use crate::types::{TileType, MAP_SIZE};
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
use std::collections::VecDeque;

/// Represents the exoplanet exploration map with terrain, resources, and station location.
/// 
/// The Map structure contains the complete game world including terrain types,
/// resource distributions, obstacle placements, and the central station position.
/// It provides methods for tile access, pathfinding validation, and resource management.
/// 
/// # Map Coordinate System
/// 
/// - Origin (0,0) is at the top-left corner
/// - X coordinates increase from left to right (0 to MAP_SIZE-1)  
/// - Y coordinates increase from top to bottom (0 to MAP_SIZE-1)
/// - Station is positioned at the center for optimal robot deployment
/// 
/// # Thread Safety
/// 
/// Map operations are designed to be thread-safe for read access, though
/// write operations (like resource consumption) should be synchronized
/// in multi-threaded environments.
/// 
/// # Examples
/// 
/// ```rust
/// use ereea::map::Map;
/// use ereea::types::TileType;
/// 
/// let map = Map::new();
/// let station_tile = map.get_tile(map.station_x, map.station_y);
/// assert_eq!(station_tile, TileType::Empty); // Station area is always clear
/// 
/// let is_passable = map.is_valid_position(5, 5);
/// // Returns true if robots can move to position (5, 5)
/// ```
pub struct Map {
    /// 2D grid containing the type of each tile on the exploration map
    /// 
    /// Organized as `tiles[y][x]` where:
    /// - `y` is the row index (0 to MAP_SIZE-1)
    /// - `x` is the column index (0 to MAP_SIZE-1)
    /// 
    /// Each tile contains a TileType enum value representing:
    /// - Empty space (traversable)
    /// - Obstacles (impassable barriers)
    /// - Resources (energy, mineral, or scientific deposits)
    pub tiles: Vec<Vec<TileType>>,
    
    /// X coordinate of the central station where robots begin and return
    /// 
    /// The station serves as:
    /// - Robot deployment and manufacturing center
    /// - Resource storage and processing facility  
    /// - Communication hub for mission coordination
    /// - Emergency rescue and repair station
    pub station_x: usize,
    
    /// Y coordinate of the central station
    pub station_y: usize,
}

impl Map {
    /// Generates a new procedural map with balanced terrain and resource distribution.
    /// 
    /// This method creates a complete exoplanet map using advanced procedural generation
    /// techniques. The generation process ensures realistic terrain patterns while
    /// maintaining gameplay balance and accessibility requirements.
    /// 
    /// # Generation Process
    /// 
    /// 1. **Noise-Based Terrain**: Uses Perlin noise for natural terrain distribution
    /// 2. **Resource Placement**: Distributes energy, mineral, and scientific deposits
    /// 3. **Station Clearing**: Ensures station area is obstacle-free
    /// 4. **Accessibility Check**: Verifies all resources can be reached
    /// 5. **Path Creation**: Creates routes to isolated resources if needed
    /// 
    /// # Procedural Parameters
    /// 
    /// - Random seed ensures each map is unique
    /// - Noise frequency controls terrain feature size
    /// - Threshold values determine resource vs. obstacle ratios
    /// - Station is always positioned at the map center
    /// 
    /// # Returns
    /// 
    /// Newly generated Map instance ready for exploration
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let map1 = Map::new();
    /// let map2 = Map::new();
    /// // map1 and map2 will have different terrain due to random seed
    /// 
    /// assert_eq!(map1.station_x, MAP_SIZE / 2);
    /// assert_eq!(map1.station_y, MAP_SIZE / 2);
    /// ```
    pub fn new() -> Self {
        // Generate unique random seed for procedural generation
        // This ensures each game session has a different map layout
        let seed: u32 = rand::thread_rng().r#gen();
        let perlin = Perlin::new(seed);
        
        // Initialize empty map grid
        let mut tiles = vec![vec![TileType::Empty; MAP_SIZE]; MAP_SIZE];
        
        // Calculate station position at map center for optimal robot deployment
        let station_x = MAP_SIZE / 2;
        let station_y = MAP_SIZE / 2;
        
        // First pass: Generate base terrain using Perlin noise
        // Perlin noise creates natural-looking terrain patterns
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                // Normalize coordinates to 0.0-1.0 range for noise function
                let nx = x as f64 / MAP_SIZE as f64;
                let ny = y as f64 / MAP_SIZE as f64;
                
                // Sample Perlin noise with 4x frequency for detailed features
                let value = perlin.get([nx * 4.0, ny * 4.0]);
                
                // Convert noise value to tile type using threshold system
                // Higher thresholds create rarer terrain types
                tiles[y][x] = if value > 0.5 {
                    TileType::Obstacle       // 25% obstacles for navigation challenge
                } else if value > 0.3 {
                    TileType::Energy         // 20% energy deposits
                } else if value > 0.1 {
                    TileType::Mineral        // 20% mineral deposits  
                } else if value > 0.0 {
                    TileType::Scientific     // 10% scientific points
                } else {
                    TileType::Empty          // 25% empty traversable space
                };
            }
        }
        
        // Clear area around station to ensure robot deployment space
        // Station needs obstacle-free zone for robot movement and operations
        for dy in -2..=2 {
            for dx in -2..=2 {
                // Calculate coordinates with boundary clamping
                let sx = (station_x as isize + dx).clamp(0, MAP_SIZE as isize - 1) as usize;
                let sy = (station_y as isize + dy).clamp(0, MAP_SIZE as isize - 1) as usize;
                
                // Force station area to be empty (traversable)
                tiles[sy][sx] = TileType::Empty;
            }
        }
        
        // Create initial map structure
        let mut map = Self {
            tiles,
            station_x,
            station_y,
        };
        
        // Accessibility pass: Ensure all resources can be reached from station
        // This prevents generation of isolated resource deposits
        let resources = map.find_all_resources();
        for (res_x, res_y) in resources {
            // Check if each resource is reachable from station
            if !map.is_accessible(station_x, station_y, res_x, res_y) {
                // Create pathway if resource is isolated
                map.create_path(station_x, station_y, res_x, res_y);
            }
        }
        
        map
    }
    
    /// Retrieves the tile type at the specified coordinates.
    /// 
    /// This method provides safe access to map tiles with bounds checking.
    /// Coordinates outside the map boundaries are treated as obstacles
    /// to prevent robots from attempting to move off the map.
    /// 
    /// # Parameters
    /// 
    /// * `x` - X coordinate (0 to MAP_SIZE-1)
    /// * `y` - Y coordinate (0 to MAP_SIZE-1)
    /// 
    /// # Returns
    /// 
    /// TileType at the specified position, or TileType::Obstacle for out-of-bounds
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let map = Map::new();
    /// 
    /// // Valid coordinates
    /// let tile = map.get_tile(5, 5);
    /// 
    /// // Out-of-bounds coordinates return Obstacle
    /// let invalid = map.get_tile(MAP_SIZE, MAP_SIZE);
    /// assert_eq!(invalid, TileType::Obstacle);
    /// ```
    pub fn get_tile(&self, x: usize, y: usize) -> TileType {
        // Bounds checking: treat off-map areas as impassable obstacles
        if x >= MAP_SIZE || y >= MAP_SIZE {
            return TileType::Obstacle;
        }
        
        // Return actual tile type for valid coordinates
        self.tiles[y][x].clone()
    }
    
    /// Validates whether a position is traversable by robots.
    /// 
    /// This method combines bounds checking with tile type validation
    /// to determine if robots can safely move to the specified position.
    /// Used extensively by pathfinding algorithms and movement validation.
    /// 
    /// # Parameters
    /// 
    /// * `x` - X coordinate to validate
    /// * `y` - Y coordinate to validate
    /// 
    /// # Returns
    /// 
    /// `true` if the position is within bounds and not an obstacle, `false` otherwise
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let map = Map::new();
    /// 
    /// // Check if position is valid for robot movement
    /// if map.is_valid_position(target_x, target_y) {
    ///     // Robot can move to this position
    ///     robot.move_to(target_x, target_y);
    /// }
    /// ```
    pub fn is_valid_position(&self, x: usize, y: usize) -> bool {
        // Must be within map boundaries AND not an obstacle
        x < MAP_SIZE && y < MAP_SIZE && self.tiles[y][x] != TileType::Obstacle
    }
    
    // Consommer une ressource à une position (ne modifie que les ressources)
    pub fn consume_resource(&mut self, x: usize, y: usize) {
        if x < MAP_SIZE && y < MAP_SIZE {
            match self.tiles[y][x] {
                TileType::Energy | TileType::Mineral | TileType::Scientific => {
                    self.tiles[y][x] = TileType::Empty;
                },
                _ => {}
            }
        }
    }
    
    fn find_all_resources(&self) -> Vec<(usize, usize)> {
        let mut resources = Vec::new();
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                match self.tiles[y][x] {
                    TileType::Energy | TileType::Mineral | TileType::Scientific => {
                        resources.push((x, y));
                    },
                    _ => {}
                }
            }
        }
        resources
    }
    
    // Vérifie si une position est accessible depuis une autre (BFS)
    fn is_accessible(&self, start_x: usize, start_y: usize, target_x: usize, target_y: usize) -> bool {
        let mut visited = vec![vec![false; MAP_SIZE]; MAP_SIZE];
        let mut queue = VecDeque::new();
        
         // Point de départ
        queue.push_back((start_x, start_y));
        visited[start_y][start_x] = true;
        
        while let Some((x, y)) = queue.pop_front() {
            // Si on a atteint la cible
            if x == target_x && y == target_y {
                return true;
            }
            
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    
                    // Explorer les voisins
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    
                    if nx >= 0 && nx < MAP_SIZE as isize && ny >= 0 && ny < MAP_SIZE as isize {
                        let nx = nx as usize;
                        let ny = ny as usize;
                        
                        if !visited[ny][nx] && self.tiles[ny][nx] != TileType::Obstacle {
                            visited[ny][nx] = true;
                            queue.push_back((nx, ny));
                        }
                    }
                }
            }
        }
        
        false
    }
    
    // Crée un chemin entre deux points en supprimant les obstacles
    fn create_path(&mut self, start_x: usize, start_y: usize, target_x: usize, target_y: usize) {
        // Utiliser la distance de Manhattan pour créer un chemin approximatif
        let mut current_x = start_x;
        let mut current_y = start_y;
        
        while current_x != target_x || current_y != target_y {
            // Décider de la direction à prendre
            let move_horizontal = rand::thread_rng().gen_bool(0.5);
            
            if move_horizontal && current_x != target_x {
                // Déplacement horizontal
                if current_x < target_x {
                    current_x += 1;
                } else {
                    current_x -= 1;
                }
            } else if current_y != target_y {
                // Déplacement vertical
                if current_y < target_y {
                    current_y += 1;
                } else {
                    current_y -= 1;
                }
            } else if current_x != target_x {
                // Déplacement horizontal forcé
                if current_x < target_x {
                    current_x += 1;
                } else {
                    current_x -= 1;
                }
            }
            
            // Si c'est un obstacle, le transformer en case vide
            if self.tiles[current_y][current_x] == TileType::Obstacle {
                self.tiles[current_y][current_x] = TileType::Empty;
            }
        }
    }
}