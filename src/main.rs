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
    enable_raw_mode()?;
    
    let mut map = Map::new();
    let mut station = Station::new();
    
    let mut robots = vec![
        Robot::new_with_memory(
            map.station_x, 
            map.station_y, 
            RobotType::Explorer, 
            1,
            map.station_x, 
            map.station_y,
            station.global_memory.clone()
        ),
        Robot::new_with_memory(
            map.station_x, 
            map.station_y, 
            RobotType::EnergyCollector, 
            2,
            map.station_x, 
            map.station_y,
            station.global_memory.clone()
        ),
        Robot::new_with_memory(
            map.station_x, 
            map.station_y, 
            RobotType::MineralCollector, 
            3,
            map.station_x, 
            map.station_y,
            station.global_memory.clone()
        ),
        Robot::new_with_memory(
            map.station_x, 
            map.station_y, 
            RobotType::ScientificCollector, 
            4,
            map.station_x, 
            map.station_y,
            station.global_memory.clone()
        ),
    ];
    
    station.next_robot_id = 5;
    
    for _iteration in 0..500 {
        Display::render(&map, &station, &robots)?;
        
        station.tick();
        
        for robot in robots.iter_mut() {
            robot.update(&mut map, &mut station);
            
            if robot.energy <= 0.0 {
                robot.x = robot.home_station_x;
                robot.y = robot.home_station_y;
                robot.energy = robot.max_energy / 2.0;
            }
        }
        
        thread::sleep(Duration::from_millis(300));
    }
    
    disable_raw_mode()?;
    Ok(())
}