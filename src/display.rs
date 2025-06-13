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
        
        stdout.execute(Clear(ClearType::All))?;
        
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                stdout.execute(MoveTo(x as u16 * 2, y as u16))?;
                
                let robot_here = robots.iter().find(|r| r.x == x && r.y == y);
                
                if x == map.station_x && y == map.station_y {
                    stdout.execute(SetForegroundColor(Color::Yellow))?;
                    print!("[]");
                } else if let Some(robot) = robot_here {
                    stdout.execute(SetForegroundColor(Color::AnsiValue(robot.get_display_color())))?;
                    print!("{}:", robot.get_display_char());
                } else {
                    match map.get_tile(x, y) {
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
            println!();
        }
        
        // Station status
        stdout.execute(MoveTo(0, MAP_SIZE as u16 + 1))?;
        stdout.execute(SetForegroundColor(Color::Yellow))?;
        println!("Station: {} | Exploration: {:.1}%", 
                station.get_status(), station.get_exploration_percentage());
        
        // Robot status
        for (i, robot) in robots.iter().enumerate() {
            stdout.execute(MoveTo(0, MAP_SIZE as u16 + 3 + i as u16))?;
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
            
            println!("Robot {} ({}): {} | Pos: ({},{}) | Énergie: {:.1} | Mode: {} | Min: {} | Sci: {} | Exploré: {:.1}%", 
                    robot.id, robot.get_display_char(), robot_type, robot.x, robot.y, 
                    robot.energy, mode, robot.minerals, robot.scientific_data, robot.get_exploration_percentage());
        }
        
        stdout.flush()?;
        Ok(())
    }
}