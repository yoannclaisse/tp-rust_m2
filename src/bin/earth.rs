// src/bin/earth.rs
use ereea::types::{TileType, MAP_SIZE, RobotType, RobotMode};
use ereea::network::{SimulationState, DEFAULT_PORT};

use std::io::{stdout, Write};
use std::collections::VecDeque;
use crossterm::{
    ExecutableCommand,
    terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType},
    cursor::MoveTo,
    style::{Color, SetForegroundColor},
};
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, BufReader};

// Structure pour tracker l'état de l'affichage
struct DisplayState {
    initialized: bool,
    log_messages: VecDeque<String>,
    max_log_lines: usize,
}

impl DisplayState {
    fn new() -> Self {
        Self {
            initialized: false,
            log_messages: VecDeque::new(),
            max_log_lines: 8,
        }
    }
    
    fn add_log(&mut self, message: String) {
        self.log_messages.push_back(message);
        if self.log_messages.len() > self.max_log_lines {
            self.log_messages.pop_front();
        }
    }
}

// Positions fixes pour l'interface - Layout réorganisé
const HEADER_Y: u16 = 0;
const STATUS_Y: u16 = 3;
const MAP_START_Y: u16 = 5;
const MAP_LEFT: u16 = 2;
const STATION_INFO_Y: u16 = MAP_START_Y + MAP_SIZE as u16 + 4;
const ROBOTS_INFO_Y: u16 = STATION_INFO_Y + 4;
const LOGS_Y: u16 = ROBOTS_INFO_Y + 8;  // Logs after robots
const LEGEND_Y: u16 = LOGS_Y + 12;      // Legend at the bottom

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration du terminal
    enable_raw_mode()?;
    
    // Initialiser l'affichage
    let mut stdout = stdout();
    stdout.execute(Clear(ClearType::All))?;
    
    // Connexion au serveur de simulation
    let stream = match TcpStream::connect(format!("127.0.0.1:{}", DEFAULT_PORT)).await {
        Ok(stream) => stream,
        Err(e) => {
            disable_raw_mode()?;
            eprintln!("❌ Erreur de connexion au serveur: {}", e);
            eprintln!("💡 Assurez-vous que le serveur de simulation est en cours d'exécution.");
            eprintln!("🚀 Démarrez-le avec: cargo run --bin simulation");
            return Err(e.into());
        }
    };
    
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let mut display_state = DisplayState::new();
    
    // Message initial
    display_state.add_log("🌍 Connexion établie avec la station EREEA".to_string());
    display_state.add_log("📡 Réception des données de simulation...".to_string());
    
    loop {
        // Lire les données du serveur
        line.clear();
        
        if let Err(_) = reader.read_line(&mut line).await {
            display_state.add_log("❌ Connexion perdue avec la station".to_string());
            break;
        }
        
        if line.is_empty() {
            display_state.add_log("📡 Fin de transmission".to_string());
            break;
        }
        
        // Désérialiser l'état de la simulation
        let state: SimulationState = match serde_json::from_str(&line) {
            Ok(state) => state,
            Err(_) => {
                display_state.add_log("⚠️ Données corrompues reçues".to_string());
                continue;
            }
        };
        
        // Ajouter des logs basés sur l'état de la simulation
        if state.iteration % 50 == 0 {
            display_state.add_log(format!("📊 Cycle {} - Exploration: {:.1}%", 
                                        state.iteration, 
                                        state.station_data.exploration_percentage));
        }
        
        // Vérifier si un nouveau robot a été créé
        if state.robots_data.len() > 4 && state.iteration % 50 == 1 {
            display_state.add_log(format!("🤖 Nouveau robot déployé - Flotte: {} robots", 
                                        state.robots_data.len()));
        }
        
        // Vérifier si la mission est proche de la fin
        if state.station_data.exploration_percentage > 90.0 {
            display_state.add_log("🎯 Mission proche de l'achèvement!".to_string());
        }
        
        // Afficher l'état avec la nouvelle interface
        render_interface(&state, &mut display_state)?;
    }
    
    // Restaurer le terminal
    disable_raw_mode()?;
    Ok(())
}

// Fonction principale de rendu avec mise à jour dynamique complète
fn render_interface(state: &SimulationState, display_state: &mut DisplayState) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    
    // Initialiser la structure fixe une seule fois
    if !display_state.initialized {
        initialize_fixed_layout(&mut stdout)?;
        display_state.initialized = true;
    }
    
    // Mettre à jour TOUT le contenu dynamique
    update_all_dynamic_content(state, display_state, &mut stdout)?;
    
    stdout.flush()?;
    Ok(())
}

// Initialisation de la structure fixe (une seule fois)
fn initialize_fixed_layout(stdout: &mut std::io::Stdout) -> Result<(), Box<dyn std::error::Error>> {
    // En-tête fixe
    stdout.execute(MoveTo(0, HEADER_Y))?;
    stdout.execute(SetForegroundColor(Color::Cyan))?;
    print!("╔══════════════════════════════════════════════════════════════════════════════╗");
    stdout.execute(MoveTo(0, HEADER_Y + 1))?;
    print!("║            🌍 CENTRE DE CONTRÔLE TERRE - MISSION EREEA 🚀                   ║");
    stdout.execute(MoveTo(0, HEADER_Y + 2))?;
    print!("╚══════════════════════════════════════════════════════════════════════════════╝");
    
    // Titre de la carte
    stdout.execute(MoveTo(MAP_LEFT, MAP_START_Y))?;
    stdout.execute(SetForegroundColor(Color::Yellow))?;
    print!("🗺️  CARTE DE L'EXOPLANÈTE");
    
    // Bordures de la carte
    let map_width = MAP_SIZE as u16 * 2;
    stdout.execute(MoveTo(MAP_LEFT, MAP_START_Y + 1))?;
    stdout.execute(SetForegroundColor(Color::DarkGrey))?;
    print!("╔");
    for _ in 0..map_width { print!("═"); }
    print!("╗");
    
    // Lignes de la carte avec bordures
    for y in 0..MAP_SIZE {
        stdout.execute(MoveTo(MAP_LEFT, MAP_START_Y + 2 + y as u16))?;
        print!("║");
        for _ in 0..map_width { print!(" "); }
        print!("║");
    }
    
    // Bordure inférieure
    stdout.execute(MoveTo(MAP_LEFT, MAP_START_Y + 2 + MAP_SIZE as u16))?;
    print!("╚");
    for _ in 0..map_width { print!("═"); }
    print!("╝");
    
    // Section Station (structure fixe)
    stdout.execute(MoveTo(0, STATION_INFO_Y))?;
    stdout.execute(SetForegroundColor(Color::Yellow))?;
    print!("╔══════════════════════════════════════════════════════════════════════════════╗");
    stdout.execute(MoveTo(0, STATION_INFO_Y + 1))?;
    print!("║                          📡 RAPPORT DE LA STATION                           ║");
    stdout.execute(MoveTo(0, STATION_INFO_Y + 2))?;
    print!("╚══════════════════════════════════════════════════════════════════════════════╝");
    
    // Section Robots (structure fixe)
    stdout.execute(MoveTo(0, ROBOTS_INFO_Y))?;
    stdout.execute(SetForegroundColor(Color::Cyan))?;
    print!("╔══════════════════════════════════════════════════════════════════════════════╗");
    stdout.execute(MoveTo(0, ROBOTS_INFO_Y + 1))?;
    print!("║                            🤖 STATUT DES ROBOTS                             ║");
    stdout.execute(MoveTo(0, ROBOTS_INFO_Y + 2))?;
    print!("╚══════════════════════════════════════════════════════════════════════════════╝");
    
    // Section Logs (nouvelle section)
    stdout.execute(MoveTo(0, LOGS_Y))?;
    stdout.execute(SetForegroundColor(Color::Green))?;
    print!("╔══════════════════════════════════════════════════════════════════════════════╗");
    stdout.execute(MoveTo(0, LOGS_Y + 1))?;
    print!("║                           📋 JOURNAL DE MISSION                             ║");
    stdout.execute(MoveTo(0, LOGS_Y + 2))?;
    print!("╚══════════════════════════════════════════════════════════════════════════════╝");
    
    // Lignes vides pour les logs
    for i in 0..8 {
        stdout.execute(MoveTo(0, LOGS_Y + 3 + i))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        print!("{:<80}", "");
    }
    
    // Légende (structure fixe)
    stdout.execute(MoveTo(0, LEGEND_Y))?;
    stdout.execute(SetForegroundColor(Color::White))?;
    print!("╔══════════════════════════════════════════════════════════════════════════════╗");
    stdout.execute(MoveTo(0, LEGEND_Y + 1))?;
    print!("║                                 📋 LÉGENDE                                  ║");
    stdout.execute(MoveTo(0, LEGEND_Y + 2))?;
    print!("╚══════════════════════════════════════════════════════════════════════════════╝");
    
    // Contenu de la légende (fixe)
    stdout.execute(MoveTo(0, LEGEND_Y + 3))?;
    stdout.execute(SetForegroundColor(Color::Yellow))?;
    print!("🏠 [] = Station     ");
    stdout.execute(SetForegroundColor(Color::AnsiValue(9)))?;
    print!("🔍 E# = Explorateur     ");
    stdout.execute(SetForegroundColor(Color::AnsiValue(10)))?;
    print!("⚡ P# = Énergie     ");
    stdout.execute(SetForegroundColor(Color::AnsiValue(13)))?;
    print!("⛏️  M# = Minerais");
    
    stdout.execute(MoveTo(0, LEGEND_Y + 4))?;
    stdout.execute(SetForegroundColor(Color::AnsiValue(12)))?;
    print!("🧪 S# = Scientifique     ");
    stdout.execute(SetForegroundColor(Color::Green))?;
    print!("♦ = Énergie     ");
    stdout.execute(SetForegroundColor(Color::Magenta))?;
    print!("★ = Minerai     ");
    stdout.execute(SetForegroundColor(Color::Blue))?;
    print!("○ = Science     ");
    stdout.execute(SetForegroundColor(Color::DarkGrey))?;
    print!("? = Inexploré");
    
    stdout.execute(MoveTo(0, LEGEND_Y + 5))?;
    stdout.execute(SetForegroundColor(Color::Red))?;
    print!("🚨 Ctrl+C pour quitter la mission");
    
    Ok(())
}

// Mise à jour de TOUT le contenu dynamique
fn update_all_dynamic_content(state: &SimulationState, display_state: &mut DisplayState, stdout: &mut std::io::Stdout) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Mettre à jour les informations de statut en haut
    stdout.execute(MoveTo(0, STATUS_Y))?;
    stdout.execute(SetForegroundColor(Color::White))?;
    print!("📊 Cycle: {:>4} | 🌍 Exploration: {:>5.1}% | 🤖 Robots: {:>2} | 🔋 Énergie: {:>3} | ⛏️  Minerais: {:>3} | 🧪 Science: {:>3}        ",
           state.iteration,
           state.station_data.exploration_percentage,
           state.station_data.robot_count,
           state.station_data.energy_reserves,
           state.station_data.collected_minerals,
           state.station_data.collected_scientific_data);
    
    // 2. Mettre à jour TOUTE la carte
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            stdout.execute(MoveTo(MAP_LEFT + 1 + (x as u16 * 2), MAP_START_Y + 2 + y as u16))?;
            
            // Vérifier si un robot est sur cette case
            let robot_here = state.robots_data.iter().find(|r| r.x == x && r.y == y);
            
            if x == state.map_data.station_x && y == state.map_data.station_y {
                stdout.execute(SetForegroundColor(Color::Yellow))?;
                print!("[]");
            } else if let Some(robot) = robot_here {
                // Afficher le robot
                let robot_color = match robot.robot_type {
                    RobotType::Explorer => Color::AnsiValue(9),
                    RobotType::EnergyCollector => Color::AnsiValue(10),
                    RobotType::MineralCollector => Color::AnsiValue(13),
                    RobotType::ScientificCollector => Color::AnsiValue(12),
                };
                
                stdout.execute(SetForegroundColor(robot_color))?;
                
                let display_char = match robot.robot_type {
                    RobotType::Explorer => "E",
                    RobotType::EnergyCollector => "P",
                    RobotType::MineralCollector => "M",
                    RobotType::ScientificCollector => "S",
                };
                
                print!("{}{}", display_char, robot.id);
            } else {
                // Afficher le terrain
                if !state.exploration_data.explored_tiles[y][x] {
                    stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                    print!("? ");
                } else {
                    match &state.map_data.tiles[y][x] {
                        TileType::Empty => {
                            stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                            print!("· ");
                        },
                        TileType::Obstacle => {
                            stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                            print!("██");
                        },
                        TileType::Energy => {
                            stdout.execute(SetForegroundColor(Color::Green))?;
                            print!("♦ ");
                        },
                        TileType::Mineral => {
                            stdout.execute(SetForegroundColor(Color::Magenta))?;
                            print!("★ ");
                        },
                        TileType::Scientific => {
                            stdout.execute(SetForegroundColor(Color::Blue))?;
                            print!("○ ");
                        },
                    }
                }
            }
        }
    }
    
    // 3. Mettre à jour les informations de la station
    stdout.execute(MoveTo(0, STATION_INFO_Y + 3))?;
    stdout.execute(SetForegroundColor(Color::White))?;
    print!("📊 🔋 Énergie: {:>3} | ⛏️  Minerais: {:>3} | 🧪 Science: {:>3} | ⚔️  Conflits: {:>3}                          ",
           state.station_data.energy_reserves,
           state.station_data.collected_minerals,
           state.station_data.collected_scientific_data,
           state.station_data.conflict_count);
    
    // 4. Mettre à jour les informations des robots (limité à 5 robots pour économiser l'espace)
    for i in 0..5 {
        stdout.execute(MoveTo(0, ROBOTS_INFO_Y + 3 + i as u16))?;
        
        if i < state.robots_data.len() {
            let robot = &state.robots_data[i];
            
            let robot_color = match robot.robot_type {
                RobotType::Explorer => Color::AnsiValue(9),
                RobotType::EnergyCollector => Color::AnsiValue(10),
                RobotType::MineralCollector => Color::AnsiValue(13),
                RobotType::ScientificCollector => Color::AnsiValue(12),
            };
            
            stdout.execute(SetForegroundColor(robot_color))?;
            
            let robot_type_str = match robot.robot_type {
                RobotType::Explorer => "🔍 Explorateur",
                RobotType::EnergyCollector => "⚡ Énergie",
                RobotType::MineralCollector => "⛏️  Minerais",
                RobotType::ScientificCollector => "🧪 Science",
            };
            
            let mode_str = match robot.mode {
                RobotMode::Exploring => "🚶 Exploration",
                RobotMode::Collecting => "📦 Collecte",
                RobotMode::ReturnToStation => "🏠 Retour",
                RobotMode::Idle => "😴 Repos",
            };
            
            print!("Robot #{:>2}: {:<12} | 📍({:>2},{:>2}) | 🔋{:>5.1}/{:<5.1} | {} | Min:{:>2} Sci:{:>2} | 📊{:>5.1}%            ",
                   robot.id,
                   robot_type_str,
                   robot.x, robot.y,
                   robot.energy, robot.max_energy,
                   mode_str,
                   robot.minerals,
                   robot.scientific_data,
                   robot.exploration_percentage);
        } else {
            // Effacer les lignes des robots qui n'existent plus
            stdout.execute(SetForegroundColor(Color::White))?;
            print!("{:<90}", "");
        }
    }
    
    // 5. Afficher les logs dans la section dédiée
    for (i, log_line) in display_state.log_messages.iter().enumerate() {
        stdout.execute(MoveTo(0, LOGS_Y + 3 + i as u16))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        print!("{:<80}", log_line);
    }
    
    // Effacer les lignes de logs non utilisées
    for i in display_state.log_messages.len()..display_state.max_log_lines {
        stdout.execute(MoveTo(0, LOGS_Y + 3 + i as u16))?;
        print!("{:<80}", "");
    }
    
    Ok(())
}