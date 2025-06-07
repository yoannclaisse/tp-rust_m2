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
    
    let map = Map::new();
    let mut robots = vec![
        Robot::new(map.station_x, map.station_y, RobotType::Explorer),
    ];
    
    for _iteration in 0..100 {
        Display::render(&map, &robots)?;
        
        for robot in robots.iter_mut() {
            robot.update(&map);
            
            if robot.energy <= 0.0 {
                robot.x = map.station_x;
                robot.y = map.station_y;
                robot.energy = robot.max_energy;
            }
        }
        
        thread::sleep(Duration::from_millis(200));
    }
    
    disable_raw_mode()?;
    Ok(())
}