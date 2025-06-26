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
        
        // NOTE - Clear the screen
        stdout.execute(Clear(ClearType::All))?;

        // NOTE - Draw border around the map
        let map_top = 0;
        let map_left = 0;
        let map_width = MAP_SIZE as u16 * 2;

        // NOTE - Draw top border
        stdout.execute(MoveTo(map_left, map_top))?;
        stdout.execute(SetForegroundColor(Color::DarkGrey))?;
        print!("╔");
        for _ in 0..map_width { print!("═"); }
        println!("╗");

        // NOTE - Draw map rows with side borders
        for y in 0..MAP_SIZE {
            stdout.execute(MoveTo(map_left, map_top + 1 + y as u16))?;
            print!("║");
            for x in 0..MAP_SIZE {
                // NOTE - Check if a robot is on this tile
                let robot_here = robots.iter().find(|r| r.x == x && r.y == y);
                
                if x == map.station_x && y == map.station_y {
                    // NOTE - Draw station
                    stdout.execute(SetForegroundColor(Color::Yellow))?;
                    print!("🏠");
                } else if let Some(robot) = robot_here {
                    // NOTE - Draw robot
                    stdout.execute(SetForegroundColor(Color::AnsiValue(robot.get_display_color())))?;
                    print!("{}", robot.get_display_char());
                } else {
                    // NOTE - Draw terrain/resource or unexplored
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
                            TileType::Obstacle => print!("🧱"),
                            TileType::Energy => print!("💎"),
                            TileType::Mineral => print!("⭐"),
                            TileType::Scientific => print!("🔬"),
                        }
                    } else {
                        stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                        print!("❓");
                    }
                }
            }
            stdout.execute(SetForegroundColor(Color::DarkGrey))?;
            println!("║");
        }

        // NOTE - Draw bottom border
        stdout.execute(MoveTo(map_left, map_top + 1 + MAP_SIZE as u16))?;
        print!("╚");
        for _ in 0..map_width { print!("═"); }
        println!("╝");

        // NOTE - Display station information
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

        // NOTE - Display robot information
        let robots_y = info_y + 4;
        stdout.execute(MoveTo(0, robots_y))?;
        stdout.execute(SetForegroundColor(Color::Cyan))?;
        println!("== STATUT DES ROBOTS ==");
        stdout.execute(SetForegroundColor(Color::White))?;
        for robot in robots {
            stdout.execute(SetForegroundColor(Color::AnsiValue(robot.get_display_color())))?;
            let robot_type = match robot.robot_type {
                RobotType::Explorer => "🤖 Explorateur",
                RobotType::EnergyCollector => "🔋 Collecteur d'énergie",
                RobotType::MineralCollector => "⛏️  Collecteur de minerais",
                RobotType::ScientificCollector => "🧪 Collecteur scientifique",
            };
            let mode = match robot.mode {
                RobotMode::Exploring => "Exploration",
                RobotMode::Collecting => "Collecte",
                RobotMode::ReturnToStation => "Retour",
                RobotMode::Idle => "Inactif",
            };
            println!(
                "Robot #{}: {:<25} | Pos: ({:>2},{:>2}) | Énergie: {:>5.1}/{:<5.1} | Mode: {:<10} | Min: {:>2} | Sci: {:>2} | Exploré: {:>5.1}%",
                robot.id, robot_type, robot.x, robot.y, robot.energy, robot.max_energy, 
                mode, robot.minerals, robot.scientific_data, robot.get_exploration_percentage()
            );
        }

        // NOTE - Display legend with emojis
        let legend_y = robots_y + 2 + robots.len() as u16;
        stdout.execute(MoveTo(0, legend_y))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        println!("Légende :");
        stdout.execute(SetForegroundColor(Color::Yellow))?;
        print!("🏠 = Station   ");
        stdout.execute(SetForegroundColor(Color::AnsiValue(9)))?;
        print!("🤖 = Explorateur   ");
        stdout.execute(SetForegroundColor(Color::AnsiValue(10)))?;
        print!("🔋 = Collecteur d'énergie   ");
        stdout.execute(SetForegroundColor(Color::AnsiValue(13)))?;
        print!("⛏️ = Collecteur de minerais   ");
        stdout.execute(SetForegroundColor(Color::AnsiValue(12)))?;
        println!("🧪 = Collecteur scientifique");
        stdout.execute(SetForegroundColor(Color::Green))?;
        print!("💎 = Énergie   ");
        stdout.execute(SetForegroundColor(Color::Magenta))?;
        print!("⭐ = Minerai   ");
        stdout.execute(SetForegroundColor(Color::Blue))?;
        print!("🔬 = Intérêt scientifique   ");
        stdout.execute(SetForegroundColor(Color::DarkGrey))?;
        print!("🧱 = Obstacle   ");
        stdout.execute(SetForegroundColor(Color::DarkGrey))?;
        println!("❓ = Non exploré");

        stdout.flush()?;
        Ok(())
    }

    pub fn render_mission_complete(_map: &Map, station: &Station, robots: &Vec<Robot>) -> Result<()> {
        let mut stdout = stdout();
        
        // NOTE - Clear the screen for mission complete
        stdout.execute(Clear(ClearType::All))?;
        
        // NOTE - Centered mission complete message
        let center_x = 5;
        let center_y = 3;
        
        // NOTE - Draw mission complete box
        let message_lines = vec![
            "╔══════════════════════════════════════════════════════════════════╗",
            "║                                                                  ║",
            "║      🎉🚀 MISSION EREEA ACCOMPLIE AVEC SUCCÈS! 🚀🎉           ║",
            "║                                                                  ║",
            "║            🌍 EXOPLANÈTE ENTIÈREMENT EXPLORÉE 🌍               ║",
            "║                                                                  ║",
            "║                   ✅ OBJECTIFS ATTEINTS ✅                       ║",
            "║                                                                  ║",
            "║             🔍 Exploration complète: 100%                        ║",
            "║             💎 Toutes les ressources collectées                  ║",
            "║             🤖 Tous les robots rapatriés                         ║",
            "║             🏠 Retour sécurisé à la station                      ║",
            "║                                                                  ║",
            "║                      🏆 FÉLICITATIONS! 🏆                       ║",
            "║                                                                  ║",
            "║        L'humanité peut désormais coloniser cette                 ║",
            "║           exoplanète en toute sécurité!                          ║",
            "║                                                                  ║",
            "║                    🌟 MISSION RÉUSSIE 🌟                        ║",
            "║                                                                  ║",
            "╚══════════════════════════════════════════════════════════════════╝",
        ];
        
        // NOTE - Print mission complete message
        for (i, line) in message_lines.iter().enumerate() {
            stdout.execute(MoveTo(center_x, center_y + i as u16))?;
            stdout.execute(SetForegroundColor(Color::Yellow))?;
            print!("{}", line);
        }
        
        // NOTE - Print final statistics
        stdout.execute(MoveTo(center_x + 5, center_y + message_lines.len() as u16 + 2))?;
        stdout.execute(SetForegroundColor(Color::Cyan))?;
        println!("🎯 STATISTIQUES DE LA MISSION:");
        
        stdout.execute(MoveTo(center_x + 8, center_y + message_lines.len() as u16 + 4))?;
        stdout.execute(SetForegroundColor(Color::Green))?;
        println!("📊 Exoplanète cartographiée à 100%");
        
        stdout.execute(MoveTo(center_x + 8, center_y + message_lines.len() as u16 + 5))?;
        println!("💎 Minerais collectés: {}", station.collected_minerals);
        
        stdout.execute(MoveTo(center_x + 8, center_y + message_lines.len() as u16 + 6))?;
        println!("🧪 Données scientifiques: {}", station.collected_scientific_data);
        
        stdout.execute(MoveTo(center_x + 8, center_y + message_lines.len() as u16 + 7))?;
        println!("🤖 Robots déployés: {}", robots.len());
        
        stdout.execute(MoveTo(center_x + 8, center_y + message_lines.len() as u16 + 8))?;
        println!("⚔️  Conflits résolus: {}", station.conflict_count);
        
        // NOTE - Print robot types used
        stdout.execute(MoveTo(center_x + 8, center_y + message_lines.len() as u16 + 10))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        println!("🛠️  ROBOTS UTILISÉS:");
        
        stdout.execute(MoveTo(center_x + 10, center_y + message_lines.len() as u16 + 11))?;
        stdout.execute(SetForegroundColor(Color::AnsiValue(9)))?;
        print!("🤖 Explorateurs   ");
        stdout.execute(SetForegroundColor(Color::AnsiValue(10)))?;
        print!("🔋 Collecteurs d'énergie   ");
        stdout.execute(SetForegroundColor(Color::AnsiValue(13)))?;
        println!("⛏️  Collecteurs de minerais");
        
        stdout.execute(MoveTo(center_x + 10, center_y + message_lines.len() as u16 + 12))?;
        stdout.execute(SetForegroundColor(Color::AnsiValue(12)))?;
        print!("🧪 Collecteurs scientifiques   ");
        stdout.execute(SetForegroundColor(Color::White))?;
        println!("- Tous revenus sains et saufs!");
        
        // NOTE - Print exit instructions
        stdout.execute(MoveTo(center_x + 15, center_y + message_lines.len() as u16 + 15))?;
        stdout.execute(SetForegroundColor(Color::Red))?;
        println!("Appuyez sur Ctrl+C pour quitter...");
        
        // NOTE - Print robot emoji animation
        stdout.execute(MoveTo(center_x + 20, center_y + message_lines.len() as u16 + 17))?;
        stdout.execute(SetForegroundColor(Color::AnsiValue(9)))?;
        print!("🤖 ");
        stdout.execute(SetForegroundColor(Color::AnsiValue(10)))?;
        print!("🔋 ");
        stdout.execute(SetForegroundColor(Color::AnsiValue(13)))?;
        print!("⛏️  ");
        stdout.execute(SetForegroundColor(Color::AnsiValue(12)))?;
        print!("🧪 ");
        stdout.execute(SetForegroundColor(Color::Yellow))?;
        println!("← Nos héros!");
        
        stdout.flush()?;
        Ok(())
    }
}