// Fichier principal de la bibliothèque EREEA
// Expose tous les modules pour utilisation externe (par les binaires)

pub mod types;          // Types de base (TileType, RobotType, etc.)
pub mod map;           // Gestion de la carte et génération procédurale
pub mod robot;         // Logique des robots et intelligence artificielle
pub mod display;       // Affichage terminal pour mode local
pub mod station;       // Gestion de la station et coordination
pub mod network;       // Communication réseau et sérialisation

// Ré-exportation des types principaux pour faciliter l'importation
pub use types::*;
pub use map::Map;
pub use robot::Robot;
pub use station::Station;
pub use network::*;