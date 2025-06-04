mod types;
mod map;
mod robot;

use map::Map;
use robot::Robot;
use types::RobotType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("EREEA - Robot Swarm Simulation");
    let map = Map::new();
    println!("Map generated with station at ({}, {})", map.station_x, map.station_y);
    let mut robot = Robot::new(map.station_x, map.station_y, RobotType::Explorer);

    for i in 0..10 {
        robot.update(&map);
        println!("Step {}: Robot at ({}, {}) with energy {:.1}",
                 i, robot.x, robot.y, robot.energy);
    }

    // Display the map with the robot
    map.display(&[&robot])?;

    Ok(())
}
