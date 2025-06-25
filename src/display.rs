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

        // Dessiner une bordure autour de la carte
        let map_top = 0;
        let map_left = 0;
        let map_width = MAP_SIZE as u16 * 2;
        let _map_height = MAP_SIZE as u16;

        // Bordure supérieure
        stdout.execute(MoveTo(map_left, map_top))?;
        stdout.execute(SetForegroundColor(Color::DarkGrey))?;
        print!("╔");
        for _ in 0..map_width { print!("═"); }
        println!("╗");

        // Affichage de la carte avec bordures latérales
        for y in 0..MAP_SIZE {
            stdout.execute(MoveTo(map_left, map_top + 1 + y as u16))?;
            print!("║");
            for x in 0..MAP_SIZE {
                // Vérifier si un robot est sur cette case
                let robot_here = robots.iter().find(|r| r.x == x && r.y == y);
                
                if x == map.station_x && y == map.station_y {
                    stdout.execute(SetForegroundColor(Color::Yellow))?;
                    print!("[]");
                } else if let Some(robot) = robot_here {
                    stdout.execute(SetForegroundColor(Color::AnsiValue(robot.get_display_color())))?;
                    print!("{}{}",robot.get_display_char(), robot.id);
                } else {
                    let base_color = match map.get_tile(x, y) {
                        TileType::Empty => Color::White,
                        TileType::Obstacle => Color::DarkGrey,
                        TileType::Energy => Color::Green,
                        TileType::Mineral => Color::Magenta,
                        TileType::Scientific => Color::Blue,
                    };
                    let is_explored_by_station = station.global_memory[y][x].explored;
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
                        stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                        print!("? ");
                    }
                }
            }
            stdout.execute(SetForegroundColor(Color::DarkGrey))?;
            println!("║");
        }

        // Bordure inférieure
        stdout.execute(MoveTo(map_left, map_top + 1 + MAP_SIZE as u16))?;
        print!("╚");
        for _ in 0..map_width { print!("═"); }
        println!("╝");

        // Afficher les informations de la station
        let info_y = map_top + 2 + MAP_SIZE as u16;
        stdout.execute(MoveTo(0, info_y))?;
        stdout.execute(SetForegroundColor(Color::Yellow))?;
        println!("== RAPPORT DE LA STATION ==");
        stdout.execute(SetForegroundColor(Color::White))?;
        println!(
            "Énergie: {} | Minerais: {} | Données scientifiques: {} | Conflits de données: {}", 
            station.energy_reserves,
            station.collected_minerals,
            station.collected_scientific_data,
            station.conflict_count
        );
        println!("Statut: {}", station.get_status());

        // Afficher les informations de chaque robot
        let robots_y = info_y + 4;
        stdout.execute(MoveTo(0, robots_y))?;
        stdout.execute(SetForegroundColor(Color::Cyan))?;
        println!("== STATUT DES ROBOTS ==");
        stdout.execute(SetForegroundColor(Color::White))?;
        for robot in robots {
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
            println!(
                "Robot #{}: {:<22} | Pos: ({:>2},{:>2}) | Énergie: {:>5.1}/{:<5.1} | Mode: {:<10} | Min: {:>2} | Sci: {:>2} | Exploré: {:>5.1}%",
                robot.id, robot_type, robot.x, robot.y, robot.energy, robot.max_energy, 
                mode, robot.minerals, robot.scientific_data, robot.get_exploration_percentage()
            );
        }

        // Afficher la légende
        let legend_y = robots_y + 2 + robots.len() as u16;
        stdout.execute(MoveTo(0, legend_y))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        println!("Légende :");
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

        stdout.flush()?;
        Ok(())
    }

    pub fn render_mission_complete(map: &Map, station: &Station, robots: &Vec<Robot>) -> Result<()> {
        // D'abord afficher la carte normale
        Self::render(map, station, robots)?;
        
        let mut stdout = stdout();
        
        // Calculer la position centrale pour le message
        let center_x = MAP_SIZE as u16;
        let center_y = (MAP_SIZE / 2) as u16;
        
        // Créer un cadre pour le message
        let message_lines = vec![
            "╔════════════════════════════════════╗",
            "║                                    ║",
            "║        🎉 MISSION COMPLETE! 🎉     ║",
            "║                                    ║",
            "║     Exoplanète entièrement         ║",
            "║       explorée et exploitée!       ║",
            "║                                    ║",
            "║   Toutes les ressources récoltées  ║",
            "║     Tous les robots à la base      ║",
            "║                                    ║",
            "║        Félicitations! 🚀           ║",
            "║                                    ║",
            "╚════════════════════════════════════╝",
        ];
        
        // Afficher le message au centre de l'écran
        for (i, line) in message_lines.iter().enumerate() {
            stdout.execute(MoveTo(center_x, center_y + i as u16 - 6))?;
            stdout.execute(SetForegroundColor(Color::Yellow))?;
            print!("{}", line);
        }
        
        // Afficher les statistiques finales
        stdout.execute(MoveTo(center_x, center_y + 8))?;
        stdout.execute(SetForegroundColor(Color::Green))?;
        println!("📊 STATISTIQUES FINALES:");
        
        stdout.execute(MoveTo(center_x, center_y + 9))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        println!("• Carte explorée: 100%");
        
        stdout.execute(MoveTo(center_x, center_y + 10))?;
        println!("• Minerais collectés: {}", station.collected_minerals);
        
        stdout.execute(MoveTo(center_x, center_y + 11))?;
        println!("• Données scientifiques: {}", station.collected_scientific_data);
        
        stdout.execute(MoveTo(center_x, center_y + 12))?;
        println!("• Robots déployés: {}", robots.len());
        
        stdout.execute(MoveTo(center_x, center_y + 13))?;
        println!("• Conflits résolus: {}", station.conflict_count);
        
        stdout.execute(MoveTo(center_x, center_y + 15))?;
        stdout.execute(SetForegroundColor(Color::Red))?;
        println!("Appuyez sur Ctrl+C pour quitter...");
        
        stdout.flush()?;
        Ok(())
    }
}