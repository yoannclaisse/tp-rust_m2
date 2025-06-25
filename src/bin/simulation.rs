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
    
    server_log!("📍 Étape 1: Génération de l'exoplanète...");
    let map = Arc::new(Mutex::new(Map::new()));
    
    // Debug: Afficher quelques informations sur la carte générée
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
    
    server_log!("🏗️  Étape 2: Construction de la station spatiale...");
    let station = Arc::new(Mutex::new(Station::new()));
    server_log!("✅ Station spatiale opérationnelle.");
    
    // Extraction des coordonnées pour éviter les verrous multiples
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
    
    // Création de l'équipe de robots initiale
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
    
    // Configuration de l'ID du prochain robot
    station.lock().unwrap().next_robot_id = 5;
    
    // Activation des robots
    for robot in robots.lock().unwrap().iter_mut() {
        robot.mode = RobotMode::Exploring;
    }
    server_log!("✅ Équipe de robots déployée sur l'exoplanète.");
    
    // === PHASE 2: CONFIGURATION DU SYSTÈME DE COMMUNICATION ===
    
    server_log!("📡 Étape 4: Configuration du système de communication...");
    // Canal pour diffuser l'état de simulation aux clients connectés
    let (state_tx, mut state_rx) = mpsc::channel::<SimulationState>(100);
    server_log!("✅ Canal de communication configuré.");
    
    // === PHASE 3: DÉMARRAGE DU THREAD DE SIMULATION ===
    
    server_log!("⚙️  Étape 5: Démarrage du moteur de simulation...");
    let map_for_sim = map.clone();
    let station_for_sim = station.clone();
    let robots_for_sim = robots.clone();
    
    // Thread principal de simulation (logique métier)
    let _simulation_thread = thread::spawn(move || {
        server_log!("🔄 Moteur de simulation actif.");
        let mut iteration = 0;
        let mut last_robot_creation = 0;
        let mut last_status_log = 0;
        
        // Boucle principale de simulation
        loop {
            // Log de progression moins fréquent (toutes les 100 itérations)
            if iteration % 100 == 0 && iteration != last_status_log {
                let exploration_pct = if let Ok(station_lock) = station_for_sim.lock() {
                    station_lock.get_exploration_percentage()
                } else {
                    0.0
                };
                server_log!("📊 Cycle: {} - Exploration: {:.1}%", iteration, exploration_pct);
                last_status_log = iteration;
            }
            
            // Incrément de l'horloge globale
            if let Ok(mut station_lock) = station_for_sim.lock() {
                station_lock.tick();
            } else {
                server_log!("❌ Erreur de verrouillage station (tick)");
                break;
            }
            
            // Mise à jour de tous les robots
            {
                let robots_result = robots_for_sim.lock();
                let map_result = map_for_sim.lock();
                let station_result = station_for_sim.lock();
                
                // Traitement atomique avec tous les verrous
                match (robots_result, map_result, station_result) {
                    (Ok(mut robots_lock), Ok(mut map_lock), Ok(mut station_lock)) => {
                        // Mise à jour de chaque robot
                        for robot in robots_lock.iter_mut() {
                            robot.update(&mut map_lock, &mut station_lock);
                            
                            // Gestion d'urgence: robot en panne d'énergie
                            if robot.energy <= 0.0 {
                                server_log!("🚨 URGENCE: Robot {} en panne d'énergie, rapatriement!", robot.id);
                                robot.x = robot.home_station_x;
                                robot.y = robot.home_station_y;
                                robot.energy = robot.max_energy / 2.0;
                                robot.mode = RobotMode::Idle;
                            }
                        }
                        
                        // Vérifier si la mission est terminée AVANT de créer de nouveaux robots
                        if station_lock.is_mission_complete(&map_lock) {
                            server_log!("🎉 MISSION TERMINÉE! Toutes les ressources collectées!");
                            // Continuer à diffuser l'état final mais ne plus créer de robots
                        } else {
                            // Logique de création de nouveaux robots (tous les 50 cycles)
                            if iteration - last_robot_creation >= 50 {
                                if let Some(new_robot) = station_lock.try_create_robot(&map_lock) {
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
            
            // Création et diffusion de l'état de simulation
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
            
            // Diffusion de l'état aux clients connectés
            if let Ok(state) = state_result {
                if let Err(_) = state_tx.blocking_send(state) {
                    // Ne pas logger à chaque fois qu'il n'y a pas de clients
                    if iteration % 1000 == 0 {
                        server_log!("⚠️  Aucun client connecté pour recevoir les données");
                    }
                }
            }
            
            // Pause entre les cycles de simulation
            thread::sleep(Duration::from_millis(300));
            iteration += 1;
        }
        
        server_log!("🔄 Moteur de simulation arrêté.");
    });
    
    server_log!("✅ Moteur de simulation lancé en arrière-plan.");
    
    // === PHASE 4: CONFIGURATION DU SERVEUR RÉSEAU ===
    
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
    
    server_log!("📺 Étape 7: Initialisation du système de diffusion...");
    // Stockage thread-safe des connexions clients
    let client_streams = Arc::new(TokioMutex::new(Vec::<TcpStream>::new()));
    let client_streams_clone = client_streams.clone();
    server_log!("✅ Système de diffusion initialisé.");
    
    // Tâche asynchrone pour diffuser l'état aux clients connectés
    server_log!("📤 Étape 8: Activation de la diffusion de données...");
    tokio::spawn(async move {
        server_log!("📤 Diffuseur de données activé.");
        
        // Boucle de diffusion
        while let Some(state) = state_rx.recv().await {
            // Sérialisation de l'état en JSON pour transmission
            let state_json = match serde_json::to_string(&state) {
                Ok(json) => json,
                Err(e) => {
                    server_log!("❌ Erreur de sérialisation: {:?}", e);
                    continue;
                }
            };
            
            // Diffusion à tous les clients connectés
            let mut disconnected_indices = Vec::new();
            let mut streams = client_streams_clone.lock().await;
            
            // Envoi des données à chaque client
            for (i, stream) in streams.iter_mut().enumerate() {
                // Envoi du JSON
                if let Err(_) = stream.write_all(state_json.as_bytes()).await {
                    disconnected_indices.push(i);
                } else {
                    // Envoi du délimiteur de fin de message
                    if let Err(_) = stream.write_all(b"\n").await {
                        disconnected_indices.push(i);
                    }
                }
            }
            
            // Nettoyage des connexions fermées
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
    
    // Boucle principale d'acceptation des connexions
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                server_log!("🌍 Nouvelle connexion depuis la Terre: {}", addr);
                
                // Ajouter le nouveau client à la liste de diffusion
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