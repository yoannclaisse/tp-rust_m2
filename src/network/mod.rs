// Module de communication réseau entre la simulation et la Terre
use serde::{Serialize, Deserialize};
use crate::types::{MAP_SIZE, TileType, RobotType, RobotMode};

// Structure pour transmettre les données de la carte via le réseau
#[derive(Serialize, Deserialize, Clone)]
pub struct MapData {
    pub tiles: Vec<Vec<TileType>>,  // Grille 2D des types de tuiles
    pub station_x: usize,           // Position X de la station
    pub station_y: usize,           // Position Y de la station
}

// Structure pour transmettre les données d'un robot via le réseau
#[derive(Serialize, Deserialize, Clone)]
pub struct RobotData {
    pub id: usize,                          // Identifiant unique du robot
    pub x: usize,                           // Position X actuelle
    pub y: usize,                           // Position Y actuelle
    pub energy: f32,                        // Niveau d'énergie actuel
    pub max_energy: f32,                    // Niveau d'énergie maximum
    pub minerals: u32,                      // Quantité de minerais transportés
    pub scientific_data: u32,               // Quantité de données scientifiques
    pub robot_type: RobotType,              // Type de spécialisation
    pub mode: RobotMode,                    // Mode de comportement actuel
    pub exploration_percentage: f32,        // Pourcentage d'exploration personnelle
}

// Structure pour transmettre les données de la station via le réseau
#[derive(Serialize, Deserialize, Clone)]
pub struct StationData {
    pub energy_reserves: u32,               // Réserves d'énergie de la station
    pub collected_minerals: u32,            // Minerais collectés
    pub collected_scientific_data: u32,     // Données scientifiques collectées
    pub exploration_percentage: f32,        // Pourcentage d'exploration globale
    pub conflict_count: usize,              // Nombre de conflits de données résolus
    pub robot_count: usize,                 // Nombre total de robots actifs
    pub status_message: String,             // Message de statut détaillé
    pub mission_complete: bool,             // Mission terminée
}

// Structure pour transmettre l'état d'exploration via le réseau
#[derive(Serialize, Deserialize, Clone)]
pub struct ExplorationData {
    pub explored_tiles: Vec<Vec<bool>>,     // Grille des cases explorées (true/false)
}

// Structure principale contenant l'état complet de la simulation
#[derive(Serialize, Deserialize, Clone)]
pub struct SimulationState {
    pub map_data: MapData,                  // Données de la carte
    pub robots_data: Vec<RobotData>,        // Données de tous les robots
    pub station_data: StationData,          // Données de la station
    pub exploration_data: ExplorationData, // Données d'exploration
    pub iteration: u32,                     // Numéro du cycle de simulation
}

// Configuration réseau
pub const DEFAULT_PORT: u16 = 8080;                    // Port TCP par défaut
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;      // Taille max des messages (1MB)

// Fonction utilitaire : convertir Map vers MapData pour transmission réseau
pub fn create_map_data(map: &crate::map::Map) -> MapData {
    MapData {
        tiles: map.tiles.clone(),           // Copie de la grille des tuiles
        station_x: map.station_x,
        station_y: map.station_y,
    }
}

// Fonction utilitaire : convertir Robot vers RobotData pour transmission réseau
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

// Fonction utilitaire : convertir Station vers StationData pour transmission réseau
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

// Fonction utilitaire : créer les données d'exploration pour transmission réseau
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

// Fonction principale : créer l'état complet de simulation pour transmission
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