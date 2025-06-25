use crate::types::{TileType, MAP_SIZE};
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
use std::collections::VecDeque;

pub struct Map {
    pub tiles: Vec<Vec<TileType>>,
    pub station_x: usize,
    pub station_y: usize,
}

impl Map {
    pub fn new() -> Self {
        // Générer une seed aléatoire à chaque exécution
        let seed: u32 = rand::thread_rng().r#gen();
        let perlin = Perlin::new(seed);
        let mut tiles = vec![vec![TileType::Empty; MAP_SIZE]; MAP_SIZE];
        
        // Station au centre
        let station_x = MAP_SIZE / 2;
        let station_y = MAP_SIZE / 2;
        
        // Première passe: générer la carte avec du bruit de Perlin
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                let nx = x as f64 / MAP_SIZE as f64;
                let ny = y as f64 / MAP_SIZE as f64;
                let value = perlin.get([nx * 4.0, ny * 4.0]);
                
                tiles[y][x] = if value > 0.5 {
                    TileType::Obstacle
                } else if value > 0.3 {
                    TileType::Energy
                } else if value > 0.1 {
                    TileType::Mineral
                } else if value > 0.0 {
                    TileType::Scientific
                } else {
                    TileType::Empty
                };
            }
        }
        
        // Assurer que la zone autour de la station est libre
        for dy in -2..=2 {
            for dx in -2..=2 {
                let sx = (station_x as isize + dx).clamp(0, MAP_SIZE as isize - 1) as usize;
                let sy = (station_y as isize + dy).clamp(0, MAP_SIZE as isize - 1) as usize;
                tiles[sy][sx] = TileType::Empty;
            }
        }
        
        // Créer la carte
        let mut map = Self {
            tiles,
            station_x,
            station_y,
        };
        
        // Identifier toutes les ressources et assurer leur accessibilité
        let resources = map.find_all_resources();
        for (res_x, res_y) in resources {
            if !map.is_accessible(station_x, station_y, res_x, res_y) {
                map.create_path(station_x, station_y, res_x, res_y);
            }
        }
        
        map
    }
    
    pub fn get_tile(&self, x: usize, y: usize) -> TileType {
        if x >= MAP_SIZE || y >= MAP_SIZE {
            // Traiter les zones hors limites comme des obstacles
            return TileType::Obstacle;
        }
        self.tiles[y][x].clone()
    }
    
    pub fn is_valid_position(&self, x: usize, y: usize) -> bool {
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