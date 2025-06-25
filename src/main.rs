mod types;
mod map;  
mod robot;
mod display;
mod station;

use std::{thread, time::Duration};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    
    println!("🚀 EREEA - Exploration Robotique d'Exoplanètes Autonome");
    println!("========================================================");
    println!();
    println!("Cette application utilise maintenant une architecture client-serveur.");
    println!();
    println!("Pour démarrer la simulation complète :");
    println!("1. 🖥️  Démarrez le serveur de simulation : cargo run --bin simulation");
    println!("2. 🌍 Démarrez l'interface Terre : cargo run --bin earth");
    println!();
    println!("L'interface actuelle (main.rs) sera bientôt supprimée au profit");
    println!("de l'architecture distribuée plus robuste.");
    println!();
    println!("Fermeture dans 10 secondes...");
    
    thread::sleep(Duration::from_secs(10));
    
    disable_raw_mode()?;
    Ok(())
}