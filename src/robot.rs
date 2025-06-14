use crate::types::{MAP_SIZE, TileType, RobotType, RobotMode};
use crate::map::Map;
use crate::station::{Station, TerrainData};
use rand::prelude::*;
use std::collections::{VecDeque, BinaryHeap, HashMap};
use std::cmp::Ordering;

// Structure pour l'algorithme A* de recherche de chemin
#[derive(Clone, Eq, PartialEq)]
struct Node {
    position: (usize, usize),  // Position sur la carte
    g_cost: usize,             // Coût depuis le départ
    f_cost: usize,             // Coût total estimé
}

// Implémentation pour la file de priorité (min-heap)
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_cost.cmp(&self.f_cost) // Inversé pour min-heap
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Structure principale d'un robot
pub struct Robot {
    pub x: usize,                           // Position X sur la carte
    pub y: usize,                           // Position Y sur la carte
    pub energy: f32,                        // Énergie actuelle
    pub max_energy: f32,                    // Énergie maximale
    pub minerals: u32,                      // Minerais transportés
    pub scientific_data: u32,               // Données scientifiques collectées
    pub robot_type: RobotType,              // Type de spécialisation
    pub mode: RobotMode,                    // Mode de comportement actuel
    pub memory: Vec<Vec<TerrainData>>,      // Mémoire personnelle du robot
    pub target: Option<(usize, usize)>,     // Destination actuelle
    pub id: usize,                          // Identifiant unique
    pub home_station_x: usize,              // Position X de la station d'origine
    pub home_station_y: usize,              // Position Y de la station d'origine
    pub last_sync_time: u32,                // Dernière synchronisation avec station
}

impl Robot {
    // Constructeur basique d'un robot
    pub fn new(x: usize, y: usize, robot_type: RobotType) -> Self {
        // Paramètres selon le type de robot
        let (max_energy, energy) = match robot_type {
            RobotType::Explorer => (80.0, 80.0),           // Explorateur: endurance moyenne
            RobotType::EnergyCollector => (120.0, 120.0),  // Collecteur énergie: grande autonomie
            RobotType::MineralCollector => (100.0, 100.0), // Collecteur minerais: bonne endurance
            RobotType::ScientificCollector => (60.0, 60.0), // Collecteur science: faible autonomie
        };
        
        // Initialiser une mémoire vierge
        let mut memory = Vec::with_capacity(MAP_SIZE);
        for _ in 0..MAP_SIZE {
            let row = vec![
                TerrainData {
                    explored: false,
                    timestamp: 0,
                    robot_id: 0,
                    robot_type: RobotType::Explorer,
                }; 
                MAP_SIZE
            ];
            memory.push(row);
        }
        
        Self {
            x: x.clamp(0, MAP_SIZE - 1),               // S'assurer que la position est valide
            y: y.clamp(0, MAP_SIZE - 1),
            energy,
            max_energy,
            minerals: 0,
            scientific_data: 0,
            robot_type,
            mode: RobotMode::Exploring,
            memory,
            target: None,
            id: 0,                                     // Sera défini par la station
            home_station_x: x.clamp(0, MAP_SIZE - 1),
            home_station_y: y.clamp(0, MAP_SIZE - 1),
            last_sync_time: 0,
        }
    }
    
    // Constructeur avec mémoire préchargée (pour robots créés par la station)
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
            memory,                                    // Mémoire partagée de la station
            target: None,
            id,
            home_station_x: station_x,
            home_station_y: station_y,
            last_sync_time: 0,
        }
    }
    
    // Retourne le caractère d'affichage selon le type
    pub fn get_display_char(&self) -> &str {
        match self.robot_type {
            RobotType::Explorer => "E",               // E pour Explorateur
            RobotType::EnergyCollector => "P",        // P pour Power (énergie)
            RobotType::MineralCollector => "M",       // M pour Minerais
            RobotType::ScientificCollector => "S",    // S pour Science
        }
    }
    
    // Retourne la couleur d'affichage selon le type
    pub fn get_display_color(&self) -> u8 {
        match self.robot_type {
            RobotType::Explorer => 9,          // Rouge vif
            RobotType::EnergyCollector => 10,  // Vert vif
            RobotType::MineralCollector => 13, // Magenta vif
            RobotType::ScientificCollector => 12, // Bleu vif
        }
    }
    
    // S'assure que le robot reste dans les limites de la carte
    fn clamp_position(&mut self) {
        self.x = self.x.clamp(0, MAP_SIZE - 1);
        self.y = self.y.clamp(0, MAP_SIZE - 1);
    }
    
    // Méthode principale de mise à jour du robot (appelée à chaque cycle)
    pub fn update(&mut self, map: &mut Map, station: &mut Station) {
        // Vérifier les limites de position
        self.clamp_position();
        
        // Consommation d'énergie de base (métabolisme)
        self.energy -= 0.1;
        
        // Mettre à jour la mémoire du robot
        self.update_memory(station);
        
        // Vérifier si le robot doit retourner à la station
        if self.should_return_to_station(map) {
            self.mode = RobotMode::ReturnToStation;
            self.target = Some((self.home_station_x, self.home_station_y));
        }
        
        // Logique spéciale quand le robot est à la station
        if self.x == self.home_station_x && self.y == self.home_station_y {
            // Recharger complètement l'énergie
            self.energy = self.max_energy;
            
            // Déposer toutes les ressources collectées
            station.deposit_resources(self.minerals, self.scientific_data);
            self.minerals = 0;
            self.scientific_data = 0;
            
            // Synchroniser les connaissances avec la station
            if station.current_time > self.last_sync_time {
                station.share_knowledge(self);
                self.last_sync_time = station.current_time;
            }
            
            // Déterminer le prochain mode selon le type de robot
            match self.robot_type {
                RobotType::Explorer => {
                    // L'explorateur retourne immédiatement explorer
                    self.mode = RobotMode::Exploring;
                },
                _ => {
                    // Les collecteurs cherchent des ressources spécifiques
                    if let Some(resource_pos) = self.find_nearest_resource(map) {
                        self.target = Some(resource_pos);
                        self.mode = RobotMode::Collecting;
                    } else {
                        // Si aucune ressource trouvée, passer en mode exploration
                        self.mode = RobotMode::Exploring;
                    }
                }
            }
        }
        
        // Machine à états pour le comportement du robot
        match self.mode {
            RobotMode::Exploring => {
                // Pour les collecteurs, vérifier s'il y a des ressources à proximité
                if self.robot_type != RobotType::Explorer {
                    if let Some(resource_pos) = self.find_nearest_resource(map) {
                        let distance = self.heuristic((self.x, self.y), resource_pos);
                        if distance <= 5 {  // Distance de détection
                            self.target = Some(resource_pos);
                            self.mode = RobotMode::Collecting;
                            self.clamp_position();
                            return;
                        }
                    }
                }
                
                // Sinon, chercher une zone à explorer
                self.target = self.find_exploration_target();
                
                if let Some(target) = self.target {
                    self.move_towards_target(map);
                }
            },
            
            RobotMode::Collecting => {
                // Vérifier si on est sur une ressource collectible
                let tile = map.get_tile(self.x, self.y);
                if self.can_collect(tile) {
                    self.collect_resource(map);
                    // Chercher d'autres ressources après collecte
                    if let Some(resource_pos) = self.find_nearest_resource(map) {
                        self.target = Some(resource_pos);
                    } else {
                        // Plus de ressources disponibles, retourner à la station
                        self.mode = RobotMode::ReturnToStation;
                        self.target = Some((self.home_station_x, self.home_station_y));
                    }
                } else if let Some(target) = self.target {
                    // Continuer vers la cible
                    self.move_towards_target(map);
                }
            },
            
            RobotMode::ReturnToStation => {
                if let Some(target) = self.target {
                    self.move_towards_target(map);
                } else {
                    self.mode = RobotMode::Idle;
                }
            },
            
            RobotMode::Idle => {
                // Seuls les explorateurs sortent automatiquement du mode idle
                if self.robot_type == RobotType::Explorer {
                    self.mode = RobotMode::Exploring;
                }
            }
        }
        
        // Vérification finale des limites après toutes les opérations
        self.clamp_position();
    }
    
    // Met à jour la mémoire du robot avec vision à 360°
    fn update_memory(&mut self, station: &Station) {
        // Marquer la position actuelle comme explorée
        if self.x < MAP_SIZE && self.y < MAP_SIZE {
            self.memory[self.y][self.x] = TerrainData {
                explored: true,
                timestamp: station.current_time,
                robot_id: self.id,
                robot_type: self.robot_type,
            };
        }
        
        // Portée de vision selon le type de robot
        let vision_range = match self.robot_type {
            RobotType::Explorer => 3,  // Explorateurs voient plus loin
            _ => 2,                    // Autres types ont vision standard
        };
        
        // Marquer toutes les cases dans la portée de vision
        for dy in -vision_range..=vision_range {
            for dx in -vision_range..=vision_range {
                let nx = self.x as isize + dx;
                let ny = self.y as isize + dy;
                
                // Vérifier que la position est valide
                if nx >= 0 && nx < MAP_SIZE as isize && ny >= 0 && ny < MAP_SIZE as isize {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    
                    // Mettre à jour seulement si pas encore exploré ou info plus récente
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
    
    // Calcule le pourcentage d'exploration personnel du robot
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
    
    // Détermine si le robot doit retourner à la station
    fn should_return_to_station(&self, map: &Map) -> bool {
        // Retour obligatoire si énergie critique
        if self.energy < self.max_energy * 0.3 {
            return true;
        }
        
        // Retour si inventaire plein (selon le type de robot)
        match self.robot_type {
            RobotType::MineralCollector => self.minerals >= 5,      // 5 minerais max
            RobotType::ScientificCollector => self.scientific_data >= 3, // 3 données max
            _ => false
        }
    }
    
    // Pour les collecteurs, retour automatique si plus de ressources
    fn should_return_no_resources(&self, map: &Map) -> bool {
        if self.robot_type != RobotType::Explorer {
            self.find_nearest_resource(map).is_none()
        } else {
            false
        }
    }
    
    // Trouve la ressource la plus proche selon la spécialisation du robot
    fn find_nearest_resource(&self, map: &Map) -> Option<(usize, usize)> {
        let target_resource = match self.robot_type {
            RobotType::Explorer => return None,  // Les explorateurs ne cherchent pas de ressources
            RobotType::EnergyCollector => TileType::Energy,
            RobotType::MineralCollector => TileType::Mineral,
            RobotType::ScientificCollector => TileType::Scientific,
        };
        
        let mut nearest = None;
        let mut min_distance = usize::MAX;
        
        // Parcourir toute la carte pour trouver la ressource la plus proche
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
    
    // Vérifie si le robot peut collecter une ressource donnée
    fn can_collect(&self, tile: TileType) -> bool {
        match (self.robot_type, tile) {
            (RobotType::EnergyCollector, TileType::Energy) => true,
            (RobotType::MineralCollector, TileType::Mineral) => true,
            (RobotType::ScientificCollector, TileType::Scientific) => true,
            _ => false,
        }
    }
    
    // Collecte une ressource sur la case actuelle
    fn collect_resource(&mut self, map: &mut Map) {
        let tile = map.get_tile(self.x, self.y);
        
        match (self.robot_type, tile) {
            (RobotType::EnergyCollector, TileType::Energy) => {
                // Recharge immédiate d'énergie
                self.energy = (self.energy + 20.0).min(self.max_energy);
                map.consume_resource(self.x, self.y);
            },
            (RobotType::MineralCollector, TileType::Mineral) => {
                // Ajouter minerai à l'inventaire
                self.minerals += 1;
                map.consume_resource(self.x, self.y);
            },
            (RobotType::ScientificCollector, TileType::Scientific) => {
                // Ajouter données scientifiques à l'inventaire
                self.scientific_data += 1;
                map.consume_resource(self.x, self.y);
            },
            _ => {}
        }
    }
    
    // Trouve une zone non explorée à cibler
    fn find_exploration_target(&self) -> Option<(usize, usize)> {
        // Recherche de cases non explorées, en commençant par les plus proches
        for distance in 1..MAP_SIZE {
            for y in 0..MAP_SIZE {
                for x in 0..MAP_SIZE {
                    if !self.memory[y][x].explored {
                        let current_distance = self.heuristic((self.x, self.y), (x, y));
                        if current_distance <= distance {
                            return Some((x, y));
                        }
                    }
                }
            }
        }
        None // Toute la carte a été explorée
    }
    
    // Déplace le robot vers sa cible en utilisant le pathfinding A*
    fn move_towards_target(&mut self, map: &Map) {
        if let Some(target) = self.target {
            let path = self.find_path(map, target);
            if let Some(&next_pos) = path.front() {
                // Vérifier que la prochaine position est valide et dans les limites
                if next_pos.0 < MAP_SIZE && next_pos.1 < MAP_SIZE && 
                   map.is_valid_position(next_pos.0, next_pos.1) {
                    
                    self.x = next_pos.0;
                    self.y = next_pos.1;
                    
                    // Coût énergétique du déplacement selon le type de robot
                    let energy_cost = match self.robot_type {
                        RobotType::Explorer => 0.3,              // Explorateurs plus efficaces
                        RobotType::EnergyCollector => 0.4,       // Collecteurs énergie moyens
                        RobotType::MineralCollector => 0.5,      // Collecteurs minerais lourds
                        RobotType::ScientificCollector => 0.6,   // Collecteurs science fragiles
                    };
                    
                    self.energy -= energy_cost;
                } else {
                    // Si le mouvement est invalide, abandonner la cible
                    self.target = None;
                }
            }
        }
        
        // S'assurer que la position finale est valide
        self.clamp_position();
    }
    
    // Algorithme A* pour trouver le chemin optimal vers une destination
    fn find_path(&self, map: &Map, target: (usize, usize)) -> VecDeque<(usize, usize)> {
        let start = (self.x, self.y);
        
        // Valider que la cible est dans les limites
        if target.0 >= MAP_SIZE || target.1 >= MAP_SIZE {
            return VecDeque::new();
        }
        
        // Si déjà à destination
        if start == target {
            return VecDeque::new();
        }
        
        // Structures pour l'algorithme A*
        let mut open_set = BinaryHeap::new();                                    // File de priorité
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new(); // Chemin parcouru
        let mut g_score: HashMap<(usize, usize), usize> = HashMap::new();       // Coût réel depuis le départ
        
        // Initialiser avec la position de départ
        g_score.insert(start, 0);
        open_set.push(Node {
            position: start,
            g_cost: 0,
            f_cost: self.heuristic(start, target),
        });
        
        // Boucle principale de A*
        while let Some(current) = open_set.pop() {
            let current_pos = current.position;
            
            // Si on a atteint la destination
            if current_pos == target {
                // Reconstruire le chemin en remontant
                let mut path = VecDeque::new();
                let mut current = target;
                
                while current != start {
                    path.push_front(current);
                    if let Some(&prev) = came_from.get(&current) {
                        current = prev;
                    } else {
                        break; // Sécurité contre les boucles infinies
                    }
                }
                
                return path;
            }
            
            // Explorer tous les voisins (8 directions)
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue; // Ignorer la position actuelle
                    }
                    
                    let nx = current_pos.0 as isize + dx;
                    let ny = current_pos.1 as isize + dy;
                    
                    // Vérification stricte des limites
                    if nx < 0 || nx >= MAP_SIZE as isize || ny < 0 || ny >= MAP_SIZE as isize {
                        continue;
                    }
                    
                    let neighbor = (nx as usize, ny as usize);
                    
                    // Vérifier que la case est accessible
                    if !map.is_valid_position(neighbor.0, neighbor.1) {
                        continue;
                    }
                    
                    // Calculer le nouveau coût
                    let tentative_g_score = g_score[&current_pos] + 1;
                    
                    // Si on a trouvé un meilleur chemin vers ce voisin
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
        
        VecDeque::new() // Aucun chemin trouvé
    }
    
    // Heuristique pour A* : distance de Manhattan
    fn heuristic(&self, a: (usize, usize), b: (usize, usize)) -> usize {
        let dx = (a.0 as isize - b.0 as isize).abs() as usize;
        let dy = (a.1 as isize - b.1 as isize).abs() as usize;
        dx + dy // Distance de Manhattan
    }
}