// Serveur de simulation EREEA
// Exécute la logique de simulation et diffuse l'état via TCP aux clients connectés

use ereea::types::{MAP_SIZE, RobotType, RobotMode};
use ereea::map::Map;
use ereea::robot::Robot;
use ereea::station::Station;
use ereea::network::{SimulationState, DEFAULT_PORT, create_simulation_state};

use std::sync::{Arc, Mutex};
use std::{thread, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, Mutex as TokioMutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Démarrage du serveur de simulation EREEA...");
    
    // === PHASE 1: INITIALISATION DES COMPOSANTS ===
    
    println!("📍 Étape 1: Génération de l'exoplanète...");
    let map = Arc::new(Mutex::new(Map::new()));
    println!("✅ Exoplanète générée avec succès.");
    
    println!("🏗️  Étape 2: Construction de la station spatiale...");
    let station = Arc::new(Mutex::new(Station::new()));
    println!("✅ Station spatiale opérationnelle.");
    
    // Extraction des coordonnées pour éviter les verrous multiples
    println!("📋 Étape 3: Configuration des robots initiaux...");
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
    println!("✅ Équipe de robots déployée sur l'exoplanète.");
    
    // === PHASE 2: CONFIGURATION DU SYSTÈME DE COMMUNICATION ===
    
    println!("📡 Étape 4: Configuration du système de communication...");
    // Canal pour diffuser l'état de simulation aux clients connectés
    let (state_tx, mut state_rx) = mpsc::channel::<SimulationState>(100);
    println!("✅ Canal de communication configuré.");
    
    // === PHASE 3: DÉMARRAGE DU THREAD DE SIMULATION ===
    
    println!("⚙️  Étape 5: Démarrage du moteur de simulation...");
    let map_for_sim = map.clone();
    let station_for_sim = station.clone();
    let robots_for_sim = robots.clone();
    
    // Thread principal de simulation (logique métier)
    let _simulation_thread = thread::spawn(move || {
        println!("🔄 Moteur de simulation actif.");
        let mut iteration = 0;
        let mut last_robot_creation = 0;
        
        // Boucle principale de simulation
        loop {
            // Log de progression toutes les 10 itérations
            if iteration % 10 == 0 {
                println!("📊 Cycle de simulation: {}", iteration);
            }
            
            // Incrément de l'horloge globale
            if let Ok(mut station_lock) = station_for_sim.lock() {
                station_lock.tick();
            } else {
                eprintln!("❌ Erreur de verrouillage station (tick)");
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
                                println!("🚨 URGENCE: Robot {} en panne d'énergie, rapatriement!", robot.id);
                                robot.x = robot.home_station_x;
                                robot.y = robot.home_station_y;
                                robot.energy = robot.max_energy / 2.0;
                                robot.mode = RobotMode::Idle;
                            }
                        }
                        
                        // Logique de création de nouveaux robots (tous les 50 cycles)
                        if iteration - last_robot_creation >= 50 {
                            if let Some(new_robot) = station_lock.try_create_robot(&map_lock) {
                                robots_lock.push(new_robot);
                                last_robot_creation = iteration;
                                println!("🤖 Nouveau robot déployé! Flotte totale: {} robots", robots_lock.len());
                            }
                        }
                    },
                    _ => {
                        eprintln!("❌ Erreur de verrouillage lors de la mise à jour des robots");
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
                        eprintln!("❌ Erreur lors de la création de l'état de simulation");
                        Err(())
                    }
                }
            };
            
            // Diffusion de l'état aux clients connectés
            if let Ok(state) = state_result {
                if let Err(_) = state_tx.blocking_send(state) {
                    println!("⚠️  Aucun client connecté pour recevoir les données");
                    // Continuer la simulation même sans clients
                }
            }
            
            // Pause entre les cycles de simulation
            thread::sleep(Duration::from_millis(300));
            iteration += 1;
        }
        
        println!("🔄 Moteur de simulation arrêté.");
    });
    
    println!("✅ Moteur de simulation lancé en arrière-plan.");
    
    // === PHASE 4: CONFIGURATION DU SERVEUR RÉSEAU ===
    
    println!("🌐 Étape 6: Ouverture des communications avec la Terre...");
    let listener = match TcpListener::bind(format!("127.0.0.1:{}", DEFAULT_PORT)).await {
        Ok(l) => {
            println!("✅ Liaison établie sur le port {}", DEFAULT_PORT);
            l
        },
        Err(e) => {
            eprintln!("❌ ERREUR: Impossible d'établir la liaison sur le port {}: {:?}", DEFAULT_PORT, e);
            eprintln!("💡 Vérifiez qu'aucun autre programme n'utilise ce port.");
            return Err(e.into());
        }
    };
    
    println!("📡 Station prête à transmettre vers la Terre!");
    println!("🌍 Démarrez l'interface Terre avec: cargo run --bin earth");
    
    // === PHASE 5: GESTION DES CONNEXIONS CLIENTES ===
    
    println!("📺 Étape 7: Initialisation du système de diffusion...");
    // Stockage thread-safe des connexions clients
    let client_streams = Arc::new(TokioMutex::new(Vec::<TcpStream>::new()));
    let client_streams_clone = client_streams.clone();
    println!("✅ Système de diffusion initialisé.");
    
    // Tâche asynchrone pour diffuser l'état aux clients connectés
    println!("📤 Étape 8: Activation de la diffusion de données...");
    tokio::spawn(async move {
        println!("📤 Diffuseur de données activé.");
        
        // Boucle de diffusion
        while let Some(state) = state_rx.recv().await {
            // Sérialisation de l'état en JSON pour transmission
            let state_json = match serde_json::to_string(&state) {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("❌ Erreur de sérialisation: {:?}", e);
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
                println!("📡 Connexion Terre #{} fermée", i);
                streams.remove(*i);
            }
        }
        
        println!("📤 Diffuseur de données arrêté.");
    });
    
    println!("✅ Diffusion de données activée.");
    
    // === PHASE 6: BOUCLE D'ACCEPTATION DES CONNEXIONS ===
    
    println!("🚀 EREEA opérationnel! En attente de connexions de la Terre...");
    // println!("=" .repeat(60));
    
    // Boucle principale d'acceptation des connexions
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("🌍 Nouvelle connexion depuis la Terre: {}", addr);
                
                // Ajouter le nouveau client à la liste de diffusion
                let mut streams = client_streams.lock().await;
                streams.push(stream);
                println!("📊 Clients connectés: {}", streams.len());
            }
            Err(e) => {
                eprintln!("❌ Erreur lors de l'acceptation d'une connexion: {:?}", e);
            }
        }
    }
}