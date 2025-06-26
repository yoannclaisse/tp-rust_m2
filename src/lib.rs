// NOTE - Fichier principal de la bibliothèque EREEA
// NOTE - Expose tous les modules pour utilisation externe (par les binaires)

pub mod types;          // NOTE - Types de base (TileType, RobotType, etc.)
pub mod map;           // NOTE - Gestion de la carte et génération procédurale
pub mod robot;         // NOTE - Logique des robots et intelligence artificielle
pub mod display;       // NOTE - Affichage terminal pour mode local
pub mod station;       // NOTE - Gestion de la station et coordination
pub mod network;       // NOTE - Communication réseau et sérialisation

// NOTE - Ré-exportation des types principaux pour faciliter l'importation
pub use types::*;
pub use map::Map;
pub use robot::Robot;
pub use station::Station;
pub use network::*;