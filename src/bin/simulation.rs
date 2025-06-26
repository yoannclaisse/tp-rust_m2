// Serveur de simulation EREEA
// Exécute la logique de simulation et diffuse l'état via TCP aux clients connectés

use ereea::types::{RobotType, RobotMode, MAP_SIZE, TileType};
use ereea::map::Map;
use ereea::robot::Robot;
use ereea::station::Station;
use ereea::network::{SimulationState, DEFAULT_PORT, create_simulation_state};

use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, Mutex as TokioMutex};

// Macro pour les logs du serveur (vers stderr)
macro_rules! server_log {
    ($($arg:tt)*) => {
        eprintln!("[SERVEUR] {}", format!($($arg)*));
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server_log!("🚀 Démarrage du serveur de simulation EREEA...");
    
    // === PHASE 1: INITIALISATION DES COMPOSANTS ===
    
    // NOTE - Generating the exoplanet map
    server_log!("📍 Étape 1: Génération de l'exoplanète...");
    let map = Arc::new(Mutex::new(Map::new()));
    
    // NOTE - Counting resources on the generated map
    {
        let map_lock = map.lock().unwrap();
        let mut resource_count = 0;
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                match map_lock.get_tile(x, y) {
                    TileType::Energy | TileType::Mineral | TileType::Scientific => resource_count += 1,
                    _ => {}
                }
            }
        }
        server_log!("✅ Exoplanète générée avec {} ressources à la position station ({}, {})", 
                 resource_count, map_lock.station_x, map_lock.station_y);
    }
    
    // NOTE - Building the space station
    server_log!("🏗️  Étape 2: Construction de la station spatiale...");
    let station = Arc::new(Mutex::new(Station::new()));
    server_log!("✅ Station spatiale opérationnelle.");
    
    // NOTE - Extracting coordinates for robots
    server_log!("📋 Étape 3: Configuration des robots initiaux...");
    let (station_x, station_y, global_memory_clone) = {
        let map_lock = map.lock().unwrap();
        let station_lock = station.lock().unwrap();
        
        (
            map_lock.station_x,
            map_lock.station_y,
            station_lock.global_memory.clone()
        )
    };
    
    // NOTE - Creating the initial robot team
    let robots = Arc::new(Mutex::new(vec![
        Robot::new_with_memory(
            station_x, station_y, 
            RobotType::Explorer, 1,
            station_x, station_y,
            global_memory_clone.clone()
        ),
        Robot::new_with_memory(
            station_x, station_y, 
            RobotType::EnergyCollector, 2,
            station_x, station_y,
            global_memory_clone.clone()
        ),
        Robot::new_with_memory(
            station_x, station_y, 
            RobotType::MineralCollector, 3,
            station_x, station_y,
            global_memory_clone.clone()
        ),
        Robot::new_with_memory(
            station_x, station_y, 
            RobotType::ScientificCollector, 4,
            station_x, station_y,
            global_memory_clone.clone()
        ),
    ]));
    
    // NOTE - Setting next robot ID
    station.lock().unwrap().next_robot_id = 5;
    
    // NOTE - Activating robots
    for robot in robots.lock().unwrap().iter_mut() {
        robot.mode = RobotMode::Exploring;
    }
    server_log!("✅ Équipe de robots déployée sur l'exoplanète.");
    
    // === PHASE 2: CONFIGURATION DU SYSTÈME DE COMMUNICATION ===
    
    // NOTE - Setting up communication channel for simulation state
    server_log!("📡 Étape 4: Configuration du système de communication...");
    let (state_tx, mut state_rx) = mpsc::channel::<SimulationState>(100);
    server_log!("✅ Canal de communication configuré.");
    
    // === PHASE 3: DÉMARRAGE DU THREAD DE SIMULATION ===
    
    // NOTE - Spawning simulation engine thread
    server_log!("⚙️  Étape 5: Démarrage du moteur de simulation...");
    let map_for_sim = map.clone();
    let station_for_sim = station.clone();
    let robots_for_sim = robots.clone();
    
    // NOTE - Main simulation loop
    let _simulation_thread = thread::spawn(move || {
        server_log!("🔄 Moteur de simulation actif.");
        let mut iteration = 0;
        let mut last_robot_creation = 0;
        let mut last_status_log = 0;
        
        // NOTE - Simulation main loop
        loop {
            // NOTE - Periodic progress log
            if iteration % 100 == 0 && iteration != last_status_log {
                let exploration_pct = if let Ok(station_lock) = station_for_sim.lock() {
                    station_lock.get_exploration_percentage()
                } else {
                    0.0
                };
                server_log!("📊 Cycle: {} - Exploration: {:.1}%", iteration, exploration_pct);
                last_status_log = iteration;
            }
            
            // NOTE - Advance global clock
            if let Ok(mut station_lock) = station_for_sim.lock() {
                station_lock.tick();
            } else {
                server_log!("❌ Erreur de verrouillage station (tick)");
                break;
            }
            
            // NOTE - Update all robots and handle emergencies
            {
                let robots_result = robots_for_sim.lock();
                let map_result = map_for_sim.lock();
                let station_result = station_for_sim.lock();
                
                // NOTE - Atomic processing with all locks
                match (robots_result, map_result, station_result) {
                    (Ok(mut robots_lock), Ok(mut map_lock), Ok(mut station_lock)) => {
                        // NOTE - Update each robot
                        for robot in robots_lock.iter_mut() {
                            robot.update(&mut map_lock, &mut station_lock);
                            
                            // NOTE - Emergency: robot out of energy
                            if robot.energy <= 0.0 {
                                server_log!("🚨 URGENCE: Robot {} en panne d'énergie, rapatriement!", robot.id);
                                robot.x = robot.home_station_x;
                                robot.y = robot.home_station_y;
                                robot.energy = robot.max_energy / 2.0;
                                robot.mode = RobotMode::Idle;
                            }
                        }
                        
                        // NOTE - Check if mission is complete BEFORE creating new robots
                        if station_lock.is_mission_complete(&map_lock) {
                            server_log!("🎉 MISSION TERMINÉE! Toutes les ressources collectées!");
                            
                            // NOTE - Wait for all robots to return to base
                            let all_robots_home = robots_lock.iter().all(|r| {
                                r.x == r.home_station_x && r.y == r.home_station_y && 
                                (r.mode == RobotMode::Idle || r.mode == RobotMode::ReturnToStation)
                            });
                            
                            if all_robots_home {
                                server_log!("🏠 Tous les robots sont revenus à la base!");
                                server_log!("📊 STATISTIQUES FINALES:");
                                server_log!("   🔋 Énergie collectée: {}", station_lock.energy_reserves);
                                server_log!("   ⛏️ Minerais collectés: {}", station_lock.collected_minerals);
                                server_log!("   🧪 Données scientifiques: {}", station_lock.collected_scientific_data);
                                server_log!("   🌍 Exploration: {:.1}%", station_lock.get_exploration_percentage());
                                server_log!("   🤖 Robots déployés: {}", robots_lock.len());
                                
                                // NOTE - Broadcast final state for a few cycles then exit
                                static mut FINAL_CYCLES: u32 = 0;
                                unsafe {
                                    FINAL_CYCLES += 1;
                                    if FINAL_CYCLES >= 10 {
                                        server_log!("🚀 MISSION EREEA TERMINÉE AVEC SUCCÈS!");
                                        server_log!("🛑 Arrêt automatique de la simulation...");
                                        std::process::exit(0);
                                    }
                                }
                            }
                            
                            // NOTE - Continue broadcasting final state, no more robot creation
                        } else {
                            // NOTE - Robot creation logic (every 50 cycles)
                            if iteration - last_robot_creation >= 50 {
                                // NOTE - Check if more explorers are needed
                                let exploration_percentage = station_lock.get_exploration_percentage();
                                let explorer_count = robots_lock.iter().filter(|r| r.robot_type == RobotType::Explorer).count();
                                
                                // NOTE - Create more explorers if exploration is low and few explorers exist
                                let need_more_explorers = exploration_percentage < 80.0 && explorer_count < 3;
                                
                                if let Some(mut new_robot) = station_lock.try_create_robot(&map_lock) {
                                    // NOTE - Force explorer creation if needed
                                    if need_more_explorers {
                                        new_robot.robot_type = RobotType::Explorer;
                                        server_log!("🔍 Création prioritaire d'un explorateur pour accélérer la découverte");
                                    }
                                    
                                    robots_lock.push(new_robot);
                                    last_robot_creation = iteration;
                                    server_log!("🤖 Nouveau robot déployé! Flotte totale: {} robots", robots_lock.len());
                                }
                            }
                        }
                    },
                    _ => {
                        server_log!("❌ Erreur de verrouillage lors de la mise à jour des robots");
                        break;
                    }
                }
            }
            
            // NOTE - Create and broadcast simulation state
            let state_result = {
                match (map_for_sim.lock(), station_for_sim.lock(), robots_for_sim.lock()) {
                    (Ok(map_lock), Ok(station_lock), Ok(robots_lock)) => {
                        Ok(create_simulation_state(&map_lock, &station_lock, &robots_lock, iteration))
                    },
                    _ => {
                        server_log!("❌ Erreur lors de la création de l'état de simulation");
                        Err(())
                    }
                }
            };
            
            // NOTE - Broadcast state to connected clients
            if let Ok(state) = state_result {
                if let Err(_) = state_tx.blocking_send(state) {
                    if iteration % 1000 == 0 {
                        server_log!("⚠️  Aucun client connecté pour recevoir les données");
                    }
                }
            }
            
            // NOTE - Simulation cycle pause
            thread::sleep(Duration::from_millis(300));
            iteration += 1;
        }
        
        server_log!("🔄 Moteur de simulation arrêté.");
    });
    
    server_log!("✅ Moteur de simulation lancé en arrière-plan.");
    
    // === PHASE 4: CONFIGURATION DU SERVEUR RÉSEAU ===
    
    // NOTE - Opening TCP listener for Earth connections
    server_log!("🌐 Étape 6: Ouverture des communications avec la Terre...");
    let listener = match TcpListener::bind(format!("127.0.0.1:{}", DEFAULT_PORT)).await {
        Ok(l) => {
            server_log!("✅ Liaison établie sur le port {}", DEFAULT_PORT);
            l
        },
        Err(e) => {
            server_log!("❌ ERREUR: Impossible d'établir la liaison sur le port {}: {:?}", DEFAULT_PORT, e);
            server_log!("💡 Vérifiez qu'aucun autre programme n'utilise ce port.");
            return Err(e.into());
        }
    };
    
    server_log!("📡 Station prête à transmettre vers la Terre!");
    server_log!("🌍 Démarrez l'interface Terre avec: cargo run --bin earth");
    
    // === PHASE 5: GESTION DES CONNEXIONS CLIENTES ===
    
    // NOTE - Initializing client connection storage
    server_log!("📺 Étape 7: Initialisation du système de diffusion...");
    let client_streams = Arc::new(TokioMutex::new(Vec::<TcpStream>::new()));
    let client_streams_clone = client_streams.clone();
    server_log!("✅ Système de diffusion initialisé.");
    
    // NOTE - Spawning async task for broadcasting simulation state
    server_log!("📤 Étape 8: Activation de la diffusion de données...");
    tokio::spawn(async move {
        server_log!("📤 Diffuseur de données activé.");
        
        // NOTE - Main broadcast loop
        while let Some(state) = state_rx.recv().await {
            // NOTE - Serialize simulation state to JSON
            let state_json = match serde_json::to_string(&state) {
                Ok(json) => json,
                Err(e) => {
                    server_log!("❌ Erreur de sérialisation: {:?}", e);
                    continue;
                }
            };
            
            // NOTE - Broadcast to all connected clients
            let mut disconnected_indices = Vec::new();
            let mut streams = client_streams_clone.lock().await;
            
            for (i, stream) in streams.iter_mut().enumerate() {
                if let Err(_) = stream.write_all(state_json.as_bytes()).await {
                    disconnected_indices.push(i);
                } else {
                    if let Err(_) = stream.write_all(b"\n").await {
                        disconnected_indices.push(i);
                    }
                }
            }
            
            // NOTE - Clean up closed connections
            for i in disconnected_indices.iter().rev() {
                server_log!("📡 Connexion Terre #{} fermée", i);
                streams.remove(*i);
            }
        }
        
        server_log!("📤 Diffuseur de données arrêté.");
    });
    
    server_log!("✅ Diffusion de données activée.");
    
    // === PHASE 6: BOUCLE D'ACCEPTATION DES CONNEXIONS ===
    
    server_log!("🚀 EREEA opérationnel! En attente de connexions de la Terre...");
    
    // NOTE - Main loop for accepting new client connections
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                server_log!("🌍 Nouvelle connexion depuis la Terre: {}", addr);
                
                // NOTE - Add new client to broadcast list
                let mut streams = client_streams.lock().await;
                streams.push(stream);
                server_log!("📊 Clients connectés: {}", streams.len());
            }
            Err(e) => {
                server_log!("❌ Erreur lors de l'acceptation d'une connexion: {:?}", e);
            }
        }
    }
}