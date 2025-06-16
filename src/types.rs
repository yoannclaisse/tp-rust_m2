use serde::{Serialize, Deserialize};

// Types de tuiles sur la carte (maintenant sérialisables pour le réseau)
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum TileType {
    Empty,        // Case vide
    Obstacle,     // Obstacle infranchissable
    Energy,       // Source d'énergie
    Mineral,      // Gisement de minerais
    Scientific,   // Point d'intérêt scientifique
}

// Types de robots spécialisés (sérialisables pour transmission)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RobotType {
    Explorer,              // Robot explorateur
    EnergyCollector,       // Collecteur d'énergie
    MineralCollector,      // Collecteur de minerais
    ScientificCollector,   // Collecteur de données scientifiques
}

// Modes de comportement des robots (sérialisables pour monitoring)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RobotMode {
    Exploring,        // Mode exploration active
    Collecting,       // Mode collecte de ressources
    ReturnToStation,  // Mode retour à la station
    Idle,             // Mode inactif (en attente)
}

// Constante pour la taille de la carte
pub const MAP_SIZE: usize = 20;