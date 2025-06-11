mod types;
mod map;
mod robot;
mod display;

use std::{thread, time::Duration};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use map::Map;
use robot::Robot;
use display::Display;
use types::RobotType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    
    let mut map = Map::new();
    let mut robots = vec![
        Robot::new(map.station_x, map.station_y, RobotType::Explorer),
        Robot::new(map.station_x, map.station_y, RobotType::EnergyCollector),
        Robot::new(map.station_x, map.station_y, RobotType::MineralCollector),
        Robot::new(map.station_x, map.station_y, RobotType::ScientificCollector),
    ];
    
    for _iteration in 0..500 {
        Display::render(&map, &robots)?;
        
        for robot in robots.iter_mut() {
            robot.update(&mut map);
            
            if robot.energy <= 0.0 {
                robot.x = map.station_x;
                robot.y = map.station_y;
                robot.energy = robot.max_energy / 2.0;
            }
        }
        
        thread::sleep(Duration::from_millis(300));
    }
    
    disable_raw_mode()?;
    Ok(())
}