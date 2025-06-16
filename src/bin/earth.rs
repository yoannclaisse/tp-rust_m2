// src/bin/earth.rs
use ereea::types::{TileType, MAP_SIZE, RobotType, RobotMode};
use ereea::network::{SimulationState, DEFAULT_PORT};

use std::io::{stdout, Write};
use crossterm::{
    ExecutableCommand,
    terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType},
    cursor::MoveTo,
    style::{Color, SetForegroundColor},
};
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration du terminal
    enable_raw_mode()?;
    
    println!("Connexion au serveur de simulation...");
    
    // Connexion au serveur de simulation
    let stream = match TcpStream::connect(format!("127.0.0.1:{}", DEFAULT_PORT)).await {
        Ok(stream) => stream,
        Err(e) => {
            disable_raw_mode()?;
            eprintln!("Erreur de connexion au serveur: {}", e);
            eprintln!("Assurez-vous que le serveur de simulation est en cours d'exécution.");
            eprintln!("Démarrez-le avec: cargo run --bin simulation");
            return Err(e.into());
        }
    };
    
    println!("Connexion établie! Réception des données de la planète...");
    
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    
    loop {
        // Lire les données du serveur
        line.clear();
        
        if let Err(e) = reader.read_line(&mut line).await {
            eprintln!("Erreur de lecture: {}", e);
            break;
        }
        
        if line.is_empty() {
            // Connexion fermée par le serveur
            break;
        }
        
        // Désérialiser l'état de la simulation
        let state: SimulationState = match serde_json::from_str(&line) {
            Ok(state) => state,
            Err(e) => {
                eprintln!("Erreur de désérialisation: {}", e);
                continue;
            }
        };
        
        // Afficher l'état
        render_earth_interface(&state)?;
    }
    
    // Restaurer le terminal
    disable_raw_mode()?;
    Ok(())
}

// Fonction pour afficher l'interface Terre
fn render_earth_interface(state: &SimulationState) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    
    // Effacer l'écran
    stdout.execute(Clear(ClearType::All))?;
    
    // Afficher l'en-tête
    stdout.execute(MoveTo(0, 0))?;
    stdout.execute(SetForegroundColor(Color::Cyan))?;
    println!("== CENTRE DE CONTRÔLE TERRE - MISSION EREEA ==");
    stdout.execute(SetForegroundColor(Color::White))?;
    println!("Itération: {} | Exploration: {:.1}% | Robots actifs: {}", 
             state.iteration, 
             state.station_data.exploration_percentage,
             state.station_data.robot_count);
    
    // Afficher la carte
    println!("\n== CARTOGRAPHIE DE L'EXOPLANÈTE ==");
    
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            stdout.execute(MoveTo(x as u16 * 2, y as u16 + 5))?;
            
            // Vérifier si un robot est sur cette case
            let robot_here = state.robots_data.iter().find(|r| r.x == x && r.y == y);
            
            if x == state.map_data.station_x && y == state.map_data.station_y {
                stdout.execute(SetForegroundColor(Color::Yellow))?;
                print!("[]");
            } else if let Some(robot) = robot_here {
                // Afficher le robot avec sa couleur
                let color = match robot.robot_type {
                    RobotType::Explorer => Color::AnsiValue(9),
                    RobotType::EnergyCollector => Color::AnsiValue(10),
                    RobotType::MineralCollector => Color::AnsiValue(13),
                    RobotType::ScientificCollector => Color::AnsiValue(12),
                };
                
                stdout.execute(SetForegroundColor(color))?;
                
                let display_char = match robot.robot_type {
                    RobotType::Explorer => "E",
                    RobotType::EnergyCollector => "P",
                    RobotType::MineralCollector => "M",
                    RobotType::ScientificCollector => "S",
                };
                
                print!("{}{}", display_char, robot.id);
            } else {
                // Définir la couleur de base de la tuile
                if !state.exploration_data.explored_tiles[y][x] {
                    // Zone non explorée
                    stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                    print!("? ");
                } else {
                    // Zone explorée
                    let tile = &state.map_data.tiles[y][x];
                    match tile {
                        TileType::Empty => {
                            stdout.execute(SetForegroundColor(Color::White))?;
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
    
    // Afficher les informations de la station
    let station_y = MAP_SIZE as u16 + 6;
    stdout.execute(MoveTo(0, station_y))?;
    stdout.execute(SetForegroundColor(Color::Yellow))?;
    println!("\n== RAPPORT DE LA STATION ==");
    stdout.execute(SetForegroundColor(Color::White))?;
    println!("Énergie: {} | Minerais: {} | Données scientifiques: {} | Conflits de données: {}", 
             state.station_data.energy_reserves,
             state.station_data.collected_minerals,
             state.station_data.collected_scientific_data,
             state.station_data.conflict_count);
    println!("Statut: {}", state.station_data.status_message);
    
    // Afficher les informations des robots
    stdout.execute(MoveTo(0, station_y + 5))?;
    stdout.execute(SetForegroundColor(Color::Cyan))?;
    println!("\n== STATUT DES ROBOTS ==");
    
    for (i, robot) in state.robots_data.iter().enumerate() {
        let robot_color = match robot.robot_type {
            RobotType::Explorer => Color::AnsiValue(9),
            RobotType::EnergyCollector => Color::AnsiValue(10),
            RobotType::MineralCollector => Color::AnsiValue(13),
            RobotType::ScientificCollector => Color::AnsiValue(12),
        };
        
        stdout.execute(SetForegroundColor(robot_color))?;
        
        let robot_type = match robot.robot_type {
            RobotType::Explorer => "Explorateur",
            RobotType::EnergyCollector => "Collecteur d'énergie",
            RobotType::MineralCollector => "Collecteur de minerais",
            RobotType::ScientificCollector => "Collecteur scientifique",
        };
        
        let mode = match robot.mode {
            RobotMode::Exploring => "Exploration",
            RobotMode::Collecting => "Collecte",
            RobotMode::ReturnToStation => "Retour",
            RobotMode::Idle => "Inactif",
        };
        
        println!("Robot #{}: {} | Pos: ({},{}) | Énergie: {:.1}/{:.1} | Mode: {} | Min: {} | Sci: {} | Exploré: {:.1}%", 
                robot.id, robot_type, robot.x, robot.y, robot.energy, robot.max_energy, 
                mode, robot.minerals, robot.scientific_data, robot.exploration_percentage);
    }
    
    // Afficher la légende
    let legend_y = station_y + 6 + state.robots_data.len() as u16;
    stdout.execute(MoveTo(0, legend_y))?;
    stdout.execute(SetForegroundColor(Color::White))?;
    println!("\n== LÉGENDE ==");
    
    stdout.execute(SetForegroundColor(Color::Yellow))?;
    print!("[] = Station   ");
    
    stdout.execute(SetForegroundColor(Color::AnsiValue(9)))?;
    print!("E# = Explorateur   ");
    
    stdout.execute(SetForegroundColor(Color::AnsiValue(10)))?;
    print!("P# = Collecteur d'énergie   ");
    
    stdout.execute(SetForegroundColor(Color::AnsiValue(13)))?;
    print!("M# = Collecteur de minerais   ");
    
    stdout.execute(SetForegroundColor(Color::AnsiValue(12)))?;
    println!("S# = Collecteur scientifique");
    
    stdout.execute(SetForegroundColor(Color::Green))?;
    print!("♦ = Énergie   ");
    
    stdout.execute(SetForegroundColor(Color::Magenta))?;
    print!("★ = Minerai   ");
    
    stdout.execute(SetForegroundColor(Color::Blue))?;
    print!("○ = Intérêt scientifique   ");
    
    stdout.execute(SetForegroundColor(Color::DarkGrey))?;
    print!("██ = Obstacle   ");
    
    stdout.execute(SetForegroundColor(Color::DarkGrey))?;
    println!("? = Non exploré");
    
    // Instruction pour quitter
    println!("\nAppuyez sur Ctrl+C pour quitter");
    
    stdout.flush()?;
    Ok(())
}