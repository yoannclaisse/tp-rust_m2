mod types;
mod map;

use map::Map;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("EREEA - Robot Swarm Simulation");
    let map = Map::new();
    println!("Map generated with station at ({}, {})", map.station_x, map.station_y);
    
    // Afficher la carte avec des couleurs
    map.display()?;
    
    Ok(())
}