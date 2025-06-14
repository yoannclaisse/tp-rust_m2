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
use types::RobotType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration du terminal pour affichage couleur
    enable_raw_mode()?;
    
    // Initialisation des composants principaux
    let mut map = Map::new();      // Génération de la carte aléatoire
    let mut station = Station::new(); // Création de la station
    
    // Création des robots initiaux (équipe de départ)
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
    
    // Configurer l'ID du prochain robot
    station.next_robot_id = 5;
    
    // Variables pour la logique de création de robots
    let mut iteration = 0;
    let mut last_robot_creation = 0;
    
    // Boucle principale de simulation
    for _cycle in 0..1000 {  // Limite pour éviter boucle infinie
        // Affichage de l'état actuel
        Display::render(&map, &station, &robots)?;
        
        // Incrémenter l'horloge globale
        station.tick();
        
        // Mettre à jour tous les robots
        for robot in robots.iter_mut() {
            robot.update(&mut map, &mut station);
            
            // Gestion d'urgence : robot sans énergie
            if robot.energy <= 0.0 {
                // Téléportation d'urgence à la station
                robot.x = robot.home_station_x;
                robot.y = robot.home_station_y;
                robot.energy = robot.max_energy / 2.0; // Recharge partielle
                robot.mode = types::RobotMode::Idle;
                println!("URGENCE: Robot {} téléporté à la station!", robot.id);
            }
        }
        
        // Logique de création de nouveaux robots (tous les 50 cycles)
        if iteration - last_robot_creation >= 50 {
            if let Some(new_robot) = station.try_create_robot(&map) {
                robots.push(new_robot);
                last_robot_creation = iteration;
                println!("Nouveau robot créé ! Total: {} robots", robots.len());
            }
        }
        
        // Pause pour permettre l'observation
        thread::sleep(Duration::from_millis(300));
        iteration += 1;
    }
    
    // Restauration du terminal
    disable_raw_mode()?;
    
    println!("Simulation terminée après {} cycles", iteration);
    println!("Robots finaux: {}", robots.len());
    println!("Exploration: {:.1}%", station.get_exploration_percentage());
    
    Ok(())
}