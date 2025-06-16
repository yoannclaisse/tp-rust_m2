use std::io::{stdout, Write, Result};
use crossterm::{
    ExecutableCommand,
    terminal::{Clear, ClearType},
    cursor::MoveTo,
    style::{Color, SetForegroundColor},
};
use crate::types::{TileType, MAP_SIZE, RobotType, RobotMode};
use crate::map::Map;
use crate::robot::Robot;
use crate::station::Station;

pub struct Display;

impl Display {
    pub fn render(map: &Map, station: &Station, robots: &Vec<Robot>) -> Result<()> {
        let mut stdout = stdout();
        
        // Effacer l'écran
        stdout.execute(Clear(ClearType::All))?;
        
        // Afficher la carte
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                stdout.execute(MoveTo(x as u16 * 2, y as u16))?;
                
                // Vérifier si un robot est sur cette case
                let robot_here = robots.iter().find(|r| r.x == x && r.y == y);
                
                if x == map.station_x && y == map.station_y {
                    stdout.execute(SetForegroundColor(Color::Yellow))?;
                    print!("[]");
                } else if let Some(robot) = robot_here {
                    // Afficher le robot avec sa couleur
                    stdout.execute(SetForegroundColor(Color::AnsiValue(robot.get_display_color())))?;
                    print!("{}{}",robot.get_display_char(), robot.id);
                } else {
                    // Définir la couleur de base de la tuile
                    let base_color = match map.get_tile(x, y) {
                        TileType::Empty => Color::White,
                        TileType::Obstacle => Color::DarkGrey,
                        TileType::Energy => Color::Green,
                        TileType::Mineral => Color::Magenta,
                        TileType::Scientific => Color::Blue,
                    };
                    
                    // Vérifier si la case est explorée dans la mémoire de la station
                    let is_explored_by_station = station.global_memory[y][x].explored;
                    
                    // Modifier l'affichage en fonction de la connaissance
                    if is_explored_by_station {
                        stdout.execute(SetForegroundColor(base_color))?;
                        
                        match map.get_tile(x, y) {
                            TileType::Empty => print!("· "),
                            TileType::Obstacle => print!("██"),
                            TileType::Energy => print!("♦ "),
                            TileType::Mineral => print!("★ "),
                            TileType::Scientific => print!("○ "),
                        }
                    } else {
                        // Zone non explorée par la station
                        stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                        print!("? ");
                    }
                }
            }
            println!();
        }
        
        // Afficher les informations de la station
        stdout.execute(MoveTo(0, MAP_SIZE as u16 + 1))?;
        stdout.execute(SetForegroundColor(Color::Yellow))?;
        println!("Station: Minerais: {} | Données Scientifiques: {} | Énergie: {}", 
                station.collected_minerals, 
                station.collected_scientific_data,
                station.energy_reserves);
        
        // Ajouter le statut de la station
        stdout.execute(MoveTo(0, MAP_SIZE as u16 + 2))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        println!("Statut: {} | Carte explorée: {:.1}%", 
            station.get_status(),
            station.get_exploration_percentage());
        
        // Afficher les informations de chaque robot
        for (i, robot) in robots.iter().enumerate() {
            stdout.execute(MoveTo(0, MAP_SIZE as u16 + 4 + i as u16))?;
            stdout.execute(SetForegroundColor(Color::AnsiValue(robot.get_display_color())))?;
            
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
            
            println!("Robot #{}: {} | Énergie: {:.1}/{:.1} | Mode: {} | Min: {} | Sci: {} | Exploré: {:.1}%", 
                    robot.id, robot_type, robot.energy, robot.max_energy, mode, 
                    robot.minerals, robot.scientific_data, robot.get_exploration_percentage());
        }
        
        // Afficher la légende
        let legend_y = MAP_SIZE as u16 + 4 + robots.len() as u16 + 1;
        stdout.execute(MoveTo(0, legend_y))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        println!("Légende :");
        
        stdout.execute(MoveTo(0, legend_y + 1))?;
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
        
        stdout.execute(MoveTo(0, legend_y + 2))?;
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
        
        stdout.flush()?;
        Ok(())
    }
}