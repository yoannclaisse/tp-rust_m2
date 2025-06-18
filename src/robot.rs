use crate::types::{MAP_SIZE, TileType, RobotType, RobotMode};
use crate::map::Map;
use crate::station::{Station, TerrainData};
use rand::prelude::*;
use std::collections::{VecDeque, BinaryHeap, HashMap};
use std::cmp::Ordering;

// Structure de nœud pour l'algorithme A*
#[derive(Clone, Eq, PartialEq)]
struct Node {
    position: (usize, usize),
    g_cost: usize,  // Coût depuis le départ
    f_cost: usize,  // Coût total estimé (g_cost + heuristique)
}

// Implémentation pour le tri dans la file de priorité
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // On inverse pour avoir une min-heap au lieu d'une max-heap
        other.f_cost.cmp(&self.f_cost)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Robot {
    pub x: usize,
    pub y: usize,
    pub energy: f32,
    pub max_energy: f32,
    pub minerals: u32,
    pub scientific_data: u32,
    pub robot_type: RobotType,
    pub mode: RobotMode,
    pub memory: Vec<Vec<TerrainData>>, // Mémoire du robot avec timestamps
    pub path_to_station: VecDeque<(usize, usize)>, // Chemin vers la destination
    pub id: usize,                     // Identifiant unique du robot
    pub home_station_x: usize,         // Coordonnées X de la station d'origine
    pub home_station_y: usize,         // Coordonnées Y de la station d'origine
    pub last_sync_time: u32,           // Dernière synchronisation avec la station
}

impl Robot {
    pub fn new(x: usize, y: usize, robot_type: RobotType) -> Self {
        // Paramètres différents selon le type de robot
        let (max_energy, energy) = match robot_type {
            RobotType::Explorer => (80.0, 80.0),           // Explorateur: endurance moyenne
            RobotType::EnergyCollector => (120.0, 120.0),  // Collecteur d'énergie: grande capacité
            RobotType::MineralCollector => (100.0, 100.0), // Collecteur de minerais: bonne endurance
            RobotType::ScientificCollector => (60.0, 60.0), // Collecteur scientifique: faible endurance
        };
        
        // Initialiser une mémoire vide
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
            x,
            y,
            energy,
            max_energy,
            minerals: 0,
            scientific_data: 0,
            robot_type,
            mode: RobotMode::Exploring, // Commencer directement en mode exploration
            memory,
            path_to_station: VecDeque::new(),
            id: 0, // Sera défini par la station
            home_station_x: x,
            home_station_y: y,
            last_sync_time: 0,
        }
    }
    
    // Constructeur avec mémoire préchargée (pour la création par la station)
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
            memory,
            path_to_station: VecDeque::new(),
            id,
            home_station_x: station_x,
            home_station_y: station_y,
            last_sync_time: 0,
        }
    }
    
    // Caractère pour affichage selon le type de robot
    pub fn get_display_char(&self) -> &str {
        match self.robot_type {
            RobotType::Explorer => "E",
            RobotType::EnergyCollector => "P", // Power collector
            RobotType::MineralCollector => "M",
            RobotType::ScientificCollector => "S",
        }
    }
    
    // Couleur selon le type de robot
    pub fn get_display_color(&self) -> u8 {
        match self.robot_type {
            RobotType::Explorer => 9,          // Rouge vif
            RobotType::EnergyCollector => 10,  // Vert vif
            RobotType::MineralCollector => 13, // Magenta vif
            RobotType::ScientificCollector => 12, // Bleu vif
        }
    }
    
    // Mise à jour de la mémoire (exploration)
    pub fn update_memory(&mut self, map: &Map, station: &Station) {
        let _ = map;
        // Marquer la case actuelle comme explorée avec timestamp
        self.memory[self.y][self.x] = TerrainData {
            explored: true,
            timestamp: station.current_time,
            robot_id: self.id,
            robot_type: self.robot_type,
        };
        
        // Explorer les cases adjacentes (vision)
        let vision_range = match self.robot_type {
            RobotType::Explorer => 3, // L'explorateur voit plus loin
            _ => 2,                   // Les autres types ont une vision standard
        };
        
        for dy in -vision_range..=vision_range {
            for dx in -vision_range..=vision_range {
                let nx = self.x as isize + dx;
                let ny = self.y as isize + dy;
                
                if nx >= 0 && nx < MAP_SIZE as isize && ny >= 0 && ny < MAP_SIZE as isize {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    
                    // Si la case n'est pas encore explorée ou si notre info est plus récente
                    if !self.memory[ny][nx].explored || 
                       self.memory[ny][nx].timestamp < station.current_time {
                        
                        // Mettre à jour avec les connaissances actuelles
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
    
    // Méthode principale de mise à jour
    pub fn update(&mut self, map: &mut Map, station: &mut Station) {
        // Consommer de l'énergie (métabolisme de base)
        self.energy -= 0.1;
        
        // Vérifier si le robot doit retourner à la station
        if self.should_return_to_station(map) {
            self.mode = RobotMode::ReturnToStation;
            self.plan_path_to_station(map);
        }
        
        // Pour les collecteurs, vérifier s'il reste des ressources à collecter
        if self.robot_type != RobotType::Explorer && self.mode == RobotMode::Exploring {
            // Si aucune ressource de son type n'est disponible, retourner à la station
            if self.find_nearest_resource(map).is_none() {
                // S'il n'est pas déjà à la station
                if self.x != self.home_station_x || self.y != self.home_station_y {
                    self.mode = RobotMode::ReturnToStation;
                    self.plan_path_to_station(map);
                }
            }
        }
        
        // Si à la station, recharger, synchroniser et changer de mode
        if self.x == self.home_station_x && self.y == self.home_station_y {
            // Recharger et décharger
            self.energy = self.max_energy;
            station.deposit_resources(self.minerals, self.scientific_data);
            self.minerals = 0;
            self.scientific_data = 0;
            
            // Synchroniser les connaissances avec la station
            if station.current_time > self.last_sync_time {
                station.share_knowledge(self);
                self.last_sync_time = station.current_time;
            }
            
            // Changer de mode après avoir rechargé
            match self.robot_type {
                RobotType::Explorer => {
                    // L'explorateur retourne explorer
                    self.mode = RobotMode::Exploring;
                },
                _ => {
                    // Les collecteurs cherchent des ressources
                    if let Some(resource_pos) = self.find_nearest_resource(map) {
                        self.path_to_station = self.find_path(map, resource_pos);
                        self.mode = RobotMode::Collecting;
                    } else {
                        // Si pas de ressource trouvée, rester à la station en mode Idle
                        self.mode = RobotMode::Idle;
                    }
                }
            }
        }
        
        // Logique de déplacement selon le mode
        match self.mode {
            RobotMode::Idle => {
                // Rester sur place, mais normalement on ne devrait pas rester longtemps en idle
                if self.robot_type == RobotType::Explorer {
                    self.mode = RobotMode::Exploring;
                }
            },
            RobotMode::Exploring => {
                // Si c'est un collecteur, vérifier s'il y a des ressources à proximité
                if self.robot_type != RobotType::Explorer {
                    if let Some(resource_pos) = self.find_nearest_resource(map) {
                        let distance = self.heuristic((self.x, self.y), resource_pos);
                        if distance <= 5 {  // Distance de détection
                            self.path_to_station = self.find_path(map, resource_pos);
                            self.mode = RobotMode::Collecting;
                            return;
                        }
                    }
                }
                
                // Sinon, explorer normalement
                self.explore_move(map);
            },
            RobotMode::Collecting => {
                // Si on est sur la ressource cible, la collecter
                let tile = map.get_tile(self.x, self.y);
                let can_collect = match (self.robot_type, tile) {
                    (RobotType::EnergyCollector, TileType::Energy) => true,
                    (RobotType::MineralCollector, TileType::Mineral) => true,
                    (RobotType::ScientificCollector, TileType::Scientific) => true,
                    _ => false,
                };
                
                if can_collect {
                    self.collect_resources(map);
                } else if !self.path_to_station.is_empty() {
                    // Suivre le chemin vers la ressource
                    let next = self.path_to_station.pop_front().unwrap();
                    self.move_to(next.0, next.1);
                } else {
                    // Si le chemin est vide mais qu'on n'est pas sur la ressource, chercher une autre ressource
                    if let Some(resource_pos) = self.find_nearest_resource(map) {
                        self.path_to_station = self.find_path(map, resource_pos);
                    } else {
                        // Si plus de ressources, retourner à la station
                        self.mode = RobotMode::ReturnToStation;
                        self.plan_path_to_station(map);
                    }
                }
            },
            RobotMode::ReturnToStation => {
                if !self.path_to_station.is_empty() {
                    // Suivre le chemin vers la station
                    let next = self.path_to_station.pop_front().unwrap();
                    self.move_to(next.0, next.1);
                } else {
                    // Si le chemin est vide mais qu'on n'est pas à la station, replanifier
                    if self.x != self.home_station_x || self.y != self.home_station_y {
                        self.plan_path_to_station(map);
                        if !self.path_to_station.is_empty() {
                            let next = self.path_to_station.pop_front().unwrap();
                            self.move_to(next.0, next.1);
                        } else {
                            // Si on ne peut pas générer de chemin, revenir en mode exploration
                            self.mode = RobotMode::Exploring;
                        }
                    } else {
                        // Si on est à la station, passer en mode idle
                        self.mode = RobotMode::Idle;
                    }
                }
            }
        }
        
        // Mettre à jour la mémoire
        self.update_memory(map, station);
    }
    
    // Déplacement d'exploration intelligent
    fn explore_move(&mut self, map: &Map) {
        // Chercher les cases non explorées à proximité
        let mut unexplored_tiles = Vec::new();
        let vision_range = 5; // Portée de détection des cases non explorées
        
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                // Si la case n'est pas explorée
                if !self.memory[y][x].explored {
                    // Calculer la distance avec la position actuelle
                    let distance = self.heuristic((self.x, self.y), (x, y));
                    if distance <= vision_range {
                        unexplored_tiles.push((x, y, distance));
                    }
                }
            }
        }
        
        // Si des cases non explorées sont trouvées, aller vers la plus proche
        if !unexplored_tiles.is_empty() {
            // Trier par distance
            unexplored_tiles.sort_by_key(|&(_, _, dist)| dist);
            
            // Trouver un chemin vers la case non explorée la plus proche
            let target = (unexplored_tiles[0].0, unexplored_tiles[0].1);
            let path = self.find_path(map, target);
            
            if !path.is_empty() {
                let next = path[0];
                self.move_to(next.0, next.1);
                return;
            }
        }
        
        // Si pas de cases non explorées à proximité ou si on ne peut pas y aller,
        // faire un mouvement aléatoire comme avant
        let mut rng = rand::thread_rng();
        let mut possible_moves = Vec::new();
        
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let nx = self.x as isize + dx;
                let ny = self.y as isize + dy;
                
                if nx >= 0 && nx < MAP_SIZE as isize && ny >= 0 && ny < MAP_SIZE as isize 
                   && map.is_valid_position(nx as usize, ny as usize) {
                    possible_moves.push((nx as usize, ny as usize));
                }
            }
        }
        
        if !possible_moves.is_empty() {
            let (nx, ny) = possible_moves[rng.gen_range(0..possible_moves.len())];
            self.move_to(nx, ny);
        }
    }
    
    // Collecte de ressources selon le type de robot
    fn collect_resources(&mut self, map: &mut Map) {
        let tile = map.get_tile(self.x, self.y);
        
        match (self.robot_type, tile) {
            (RobotType::EnergyCollector, TileType::Energy) => {
                if self.energy < self.max_energy {
                    self.energy += 10.0;
                    if self.energy > self.max_energy {
                        self.energy = self.max_energy;
                    }
                    map.consume_resource(self.x, self.y);
                }
            },
            (RobotType::MineralCollector, TileType::Mineral) => {
                self.minerals += 1;
                map.consume_resource(self.x, self.y);
            },
            (RobotType::ScientificCollector, TileType::Scientific) => {
                self.scientific_data += 1;
                map.consume_resource(self.x, self.y);
            },
            _ => {
                // Si pas de ressource à collecter, explorer
                self.explore_move(map);
            }
        }
        
        // Après avoir collecté, vérifier s'il reste des ressources
        if let Some(resource_pos) = self.find_nearest_resource(map) {
            self.path_to_station = self.find_path(map, resource_pos);
        } else {
            // Si plus de ressources, retourner à la station
            self.mode = RobotMode::ReturnToStation;
            self.plan_path_to_station(map);
        }
    }
    
    // Vérifier s'il faut retourner à la station
    fn should_return_to_station(&self, map: &Map) -> bool {
        let _ = map;
        // Retourner si énergie faible
        if self.energy < self.max_energy * 0.3 {
            return true;
        }
        
        // Retourner si inventaire plein (selon le type)
        match self.robot_type {
            RobotType::MineralCollector => self.minerals >= 5,
            RobotType::ScientificCollector => self.scientific_data >= 3,
            _ => false
        }
    }
    
    // Planifier un chemin vers la station
    fn plan_path_to_station(&mut self, map: &Map) {
        let target = (self.home_station_x, self.home_station_y);
        self.path_to_station = self.find_path(map, target);
    }
    
    // Trouver la ressource la plus proche selon le type du robot
    fn find_nearest_resource(&self, map: &Map) -> Option<(usize, usize)> {
        let target_resource = match self.robot_type {
            RobotType::Explorer => None,  // L'explorateur se concentre sur l'exploration
            RobotType::EnergyCollector => Some(TileType::Energy),
            RobotType::MineralCollector => Some(TileType::Mineral),
            RobotType::ScientificCollector => Some(TileType::Scientific),
        };
        
        // Si pas de ressource cible, retourner None
        let target_resource = match target_resource {
            Some(res) => res,
            None => return None,
        };
        
        // Chercher la ressource la plus proche
        let mut nearest = None;
        let mut min_distance = usize::MAX;
        
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
    
    // Algorithme A* pour trouver le chemin optimal
    fn find_path(&self, map: &Map, target: (usize, usize)) -> VecDeque<(usize, usize)> {
        let start = (self.x, self.y);
        
        // Si déjà à destination
        if start == target {
            return VecDeque::new();
        }
        
        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut g_score: HashMap<(usize, usize), usize> = HashMap::new();
        
        // Initialiser les valeurs de départ
        g_score.insert(start, 0);
        open_set.push(Node {
            position: start,
            g_cost: 0,
            f_cost: self.heuristic(start, target),
        });
        
        while let Some(current) = open_set.pop() {
            let current_pos = current.position;
            
            // Si on est arrivé à destination
            if current_pos == target {
                // Reconstruire le chemin
                let mut path = VecDeque::new();
                let mut current = target;
                
                while current != start {
                    path.push_front(current);
                    current = *came_from.get(&current).unwrap();
                }
                
                return path;
            }
            
            // Examiner tous les voisins
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue; // Ignorer la position actuelle
                    }
                    
                    let nx = current_pos.0 as isize + dx;
                    let ny = current_pos.1 as isize + dy;
                    
                    // Vérifier si la position est valide
                    if nx < 0 || nx >= MAP_SIZE as isize || ny < 0 || ny >= MAP_SIZE as isize {
                        continue;
                    }
                    
                    let neighbor = (nx as usize, ny as usize);
                    
                    // Vérifier si c'est un obstacle
                    if !map.is_valid_position(neighbor.0, neighbor.1) {
                        continue;
                    }
                    
                    // Calculer le nouveau coût
                    let tentative_g_score = g_score[&current_pos] + 1;
                    
                    // Si on a trouvé un meilleur chemin
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
        
        // Si on ne trouve pas de chemin, retourner un chemin vide
        VecDeque::new()
    }
    
    // Heuristique pour A* (distance de Manhattan)
    fn heuristic(&self, a: (usize, usize), b: (usize, usize)) -> usize {
        let dx = (a.0 as isize - b.0 as isize).abs() as usize;
        let dy = (a.1 as isize - b.1 as isize).abs() as usize;
        dx + dy
    }
    
    // Déplacement vers une position
    fn move_to(&mut self, x: usize, y: usize) {
        // Calculer la distance
        let dx = (x as isize - self.x as isize).abs();
        let dy = (y as isize - self.y as isize).abs();
        let distance = dx.max(dy) as f32;
        
        // Consommer de l'énergie selon la distance et le type de robot
        let energy_cost = match self.robot_type {
            RobotType::Explorer => 0.3 * distance,
            RobotType::EnergyCollector => 0.4 * distance,
            RobotType::MineralCollector => 0.5 * distance,
            RobotType::ScientificCollector => 0.6 * distance,
        };
        
        self.energy -= energy_cost;
        
        // Mettre à jour la position
        self.x = x;
        self.y = y;
    }
    
    // Calculer le pourcentage de la carte exploré par ce robot
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
}