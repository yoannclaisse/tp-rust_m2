mod types;
mod map;
mod robot;
mod display;
mod station;

use std::{thread, time::Duration};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use map::Map;
use robot::Robot;
use display::Display;
use station::Station;
use types::{RobotType, TileType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    
    let mut map = Map::new();
    let mut station = Station::new();
    
    let mut robots = vec![
        Robot::new_with_memory(
            map.station_x, map.station_y, 
            RobotType::Explorer, 1,
            map.station_x, map.station_y,
            station.global_memory.clone()
        ),
        Robot::new_with_memory(
            map.station_x, map.station_y, 
            RobotType::EnergyCollector, 2,
            map.station_x, map.station_y,
            station.global_memory.clone()
        ),
        Robot::new_with_memory(
            map.station_x, map.station_y, 
            RobotType::MineralCollector, 3,
            map.station_x, map.station_y,
            station.global_memory.clone()
        ),
        Robot::new_with_memory(
            map.station_x, map.station_y, 
            RobotType::ScientificCollector, 4,
            map.station_x, map.station_y,
            station.global_memory.clone()
        ),
    ];
    
    station.next_robot_id = 5;
    let mut iteration = 0;
    let mut last_robot_creation = 0;
    
    loop {
        // Affichage
        Display::render(&map, &station, &robots)?;
        station.tick();
        
        // VÉRIFICATION SIMPLE TOUTES LES 10 ITÉRATIONS
        if iteration % 10 == 0 {
            let exploration = station.get_exploration_percentage();
            
            // Compter ressources restantes
            let mut resources = 0;
            for y in 0..20 {
                for x in 0..20 {
                    match map.get_tile(x, y) {
                        TileType::Energy | TileType::Mineral | TileType::Scientific => resources += 1,
                        _ => {}
                    }
                }
            }
            
            // SI EXPLORATION = 100% ET AUCUNE RESSOURCE -> MISSION TERMINÉE
            if exploration >= 100.0 && resources == 0 {
                println!("\n🎉🎉🎉 MISSION TERMINÉE ! 🎉🎉🎉");
                println!("🌍 Exploration: 100%");
                println!("💎 Toutes les ressources collectées !");
                println!("🚀 Félicitations !");
                
                // Afficher le message par-dessus la carte
                Display::render_mission_complete(&map, &station, &robots)?;
                
                println!("\nFermeture dans 5 secondes...");
                thread::sleep(Duration::from_secs(5));
                break;
            }
        }
        
        // Mise à jour robots
        for robot in robots.iter_mut() {
            robot.update(&mut map, &mut station);
            
            if robot.energy <= 0.0 {
                robot.x = robot.home_station_x;
                robot.y = robot.home_station_y;
                robot.energy = robot.max_energy / 2.0;
                robot.mode = types::RobotMode::Idle;
            }
        }
        
        // Création robots
        if iteration - last_robot_creation >= 50 {
            if let Some(new_robot) = station.try_create_robot(&map) {
                robots.push(new_robot);
                last_robot_creation = iteration;
            }
        }
        
        thread::sleep(Duration::from_millis(300));
        iteration += 1;
        
        if iteration > 3000 {
            println!("Arrêt de sécurité - mission non terminée");
            break;
        }
    }
    
    disable_raw_mode()?;
    println!("Simulation terminée !");
    Ok(())
}