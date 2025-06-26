// src/bin/earth.rs

/// Module imports for the Earth control center application
/// - TileType, MAP_SIZE, RobotType, RobotMode: Core simulation types
/// - SimulationState, DEFAULT_PORT: Network communication structures
use ereea::types::{TileType, MAP_SIZE, RobotType, RobotMode};
use ereea::network::{SimulationState, DEFAULT_PORT};

use std::io::{stdout, Write};
use std::collections::VecDeque;
use crossterm::{
    ExecutableCommand,
    terminal::{enable_raw_mode, disable_raw_mode, Clear, ClearType},
    cursor::MoveTo,
    style::{Color, SetForegroundColor},
};
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Structure to track the display state of the terminal interface
/// 
/// This struct manages the dynamic content that changes during simulation,
/// including initialization status and log message history.
/// 
/// # Fields
/// * `initialized` - Boolean flag to track if the fixed UI layout has been drawn
/// * `log_messages` - Rolling buffer of mission log messages (FIFO queue)
/// * `max_log_lines` - Maximum number of log lines to display (prevents overflow)
struct DisplayState {
    /// Flag indicating if the static UI layout has been initialized
    initialized: bool,
    /// FIFO queue containing recent log messages for mission tracking
    log_messages: VecDeque<String>,
    /// Maximum number of log lines to keep in memory and display
    max_log_lines: usize,
}

impl DisplayState {
    /// Creates a new DisplayState instance with default values
    /// 
    /// # Returns
    /// * `Self` - New DisplayState with uninitialized state and empty log queue
    fn new() -> Self {
        Self {
            initialized: false,        // UI layout not yet drawn
            log_messages: VecDeque::new(), // Empty message queue
            max_log_lines: 8,          // Limit to 8 visible log lines
        }
    }
    
    /// Adds a new log message to the display queue
    /// 
    /// Implements a rolling buffer - when max capacity is reached,
    /// the oldest message is removed to make space for the new one.
    /// 
    /// # Parameters
    /// * `message` - String containing the log message to add
    fn add_log(&mut self, message: String) {
        // Add new message to the end of the queue
        self.log_messages.push_back(message);
        
        // Remove oldest message if we exceed the maximum limit
        if self.log_messages.len() > self.max_log_lines {
            self.log_messages.pop_front();
        }
    }
}

/// Fixed Y-coordinate positions for the terminal user interface layout
/// These constants define the vertical positioning of each UI section
/// to maintain a consistent and organized display structure.

/// Header section at the top of the screen (title and branding)
const HEADER_Y: u16 = 0;
/// Status bar showing current simulation metrics (cycle, exploration %, etc.)
const STATUS_Y: u16 = 3;
/// Starting Y position for the exploration map display
const MAP_START_Y: u16 = 5;
/// Left margin for the map display (X offset)
const MAP_LEFT: u16 = 2;
/// Station information section (resources, conflicts, etc.)
const STATION_INFO_Y: u16 = MAP_START_Y + MAP_SIZE as u16 + 4;
/// Robot status section (individual robot details)
const ROBOTS_INFO_Y: u16 = STATION_INFO_Y + 4;
/// Mission log section (recent events and notifications)
const LOGS_Y: u16 = ROBOTS_INFO_Y + 8;
/// Legend section at the bottom (symbol explanations)
const LEGEND_Y: u16 = LOGS_Y + 12;

/// Main asynchronous entry point for the Earth control center application
/// 
/// This function establishes a TCP connection to the simulation server,
/// receives real-time simulation data, and renders a comprehensive
/// terminal-based user interface for mission monitoring.
/// 
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or any error encountered
/// 
/// # Errors
/// * Connection errors if simulation server is not running
/// * Terminal manipulation errors
/// * JSON deserialization errors from corrupted data
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // NOTE - Enable raw terminal mode for UI
    enable_raw_mode()?;
    
    // NOTE - Clear terminal for fresh UI
    let mut stdout = stdout();
    stdout.execute(Clear(ClearType::All))?;
    
    // NOTE - Connect to simulation server
    let stream = match TcpStream::connect(format!("127.0.0.1:{}", DEFAULT_PORT)).await {
        Ok(stream) => stream,
        Err(e) => {
            disable_raw_mode()?;
            eprintln!("❌ Erreur de connexion au serveur: {}", e);
            eprintln!("💡 Assurez-vous que le serveur de simulation est en cours d'exécution.");
            eprintln!("🚀 Démarrez-le avec: cargo run --bin simulation");
            return Err(e.into());
        }
    };
    
    // NOTE - Create buffered reader for incoming data
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let mut display_state = DisplayState::new();
    
    // NOTE - Add initial connection logs
    display_state.add_log("🌍 Connexion établie avec la station EREEA".to_string());
    display_state.add_log("📡 Réception des données de simulation...".to_string());
    
    // NOTE - Main event loop: receive and process simulation data
    loop {
        line.clear();
        
        // NOTE - Read a line of data from the simulation server
        if let Err(_) = reader.read_line(&mut line).await {
            display_state.add_log("❌ Connexion perdue avec la station".to_string());
            break;
        }
        
        if line.is_empty() {
            display_state.add_log("📡 Fin de transmission".to_string());
            break;
        }
        
        // NOTE - Deserialize JSON data into SimulationState
        let state: SimulationState = match serde_json::from_str(&line) {
            Ok(state) => state,
            Err(_) => {
                display_state.add_log("⚠️ Données corrompues reçues".to_string());
                continue;
            }
        };
        
        // NOTE - Check for mission completion and show victory screen
        if state.station_data.mission_complete {
            stdout.execute(Clear(ClearType::All))?;
            stdout.flush()?;
            show_victory_screen(&state)?;
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            break;
        }
        
        // NOTE - Dynamic log generation based on simulation progress
        if state.iteration % 50 == 0 {
            let exploration_pct = state.station_data.exploration_percentage;
            if exploration_pct < 30.0 {
                display_state.add_log(format!("🔍 Exploration initiale: {:.1}% - Collecteurs en attente", exploration_pct));
            } else if exploration_pct < 60.0 {
                display_state.add_log(format!("⚡ Collecte d'énergie/minerais: {:.1}%", exploration_pct));
            } else if exploration_pct < 100.0 {
                display_state.add_log(format!("🧪 Collecte scientifique: {:.1}%", exploration_pct));
            } else {
                display_state.add_log("🏁 Exploration terminée - Finalisation en cours".to_string());
            }
        }
        
        // NOTE - Log new robot deployments
        if state.robots_data.len() > 4 && state.iteration % 50 == 1 {
            display_state.add_log(format!("🤖 Nouveau robot déployé - Flotte: {} robots", 
                                        state.robots_data.len()));
        }
        
        // NOTE - Mission progress warnings
        if state.station_data.exploration_percentage > 90.0 {
            display_state.add_log("🎯 Mission proche de l'achèvement!".to_string());
        }
        
        // NOTE - Render the complete interface
        render_interface(&state, &mut display_state)?;
    }
    
    // NOTE - Restore normal terminal behavior before exiting
    disable_raw_mode()?;
    Ok(())
}

/// Main rendering coordinator for the terminal interface
/// 
/// This function manages the two-phase rendering approach:
/// 1. One-time initialization of static UI elements
/// 2. Continuous updates of dynamic content (data that changes)
/// 
/// # Parameters
/// * `state` - Current simulation state containing all game data
/// * `display_state` - Mutable UI state tracker for managing display updates
/// 
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or rendering error
fn render_interface(state: &SimulationState, display_state: &mut DisplayState) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    
    // NOTE - Initialize static layout (only once)
    if !display_state.initialized {
        initialize_fixed_layout(&mut stdout)?;
        display_state.initialized = true;
    }
    
    // NOTE - Update all dynamic content (every frame)
    update_all_dynamic_content(state, display_state, &mut stdout)?;
    
    stdout.flush()?;
    Ok(())
}

/// Initializes the static UI layout elements (borders, titles, structure)
/// 
/// This function draws all the fixed visual elements that don't change
/// during simulation execution. Called only once to optimize performance.
/// 
/// # Parameters
/// * `stdout` - Mutable reference to stdout for direct terminal writing
/// 
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or terminal manipulation error
fn initialize_fixed_layout(stdout: &mut std::io::Stdout) -> Result<(), Box<dyn std::error::Error>> {
    // NOTE - Draw header section
    stdout.execute(MoveTo(0, HEADER_Y))?;
    stdout.execute(SetForegroundColor(Color::Cyan))?;
    print!("╔══════════════════════════════════════════════════════════════════════════════╗");
    
    // Header title line with mission branding
    stdout.execute(MoveTo(0, HEADER_Y + 1))?;
    print!("║            🌍 CENTRE DE CONTRÔLE TERRE - MISSION EREEA 🚀                   ║");
    
    // Bottom border of header box
    stdout.execute(MoveTo(0, HEADER_Y + 2))?;
    print!("╚══════════════════════════════════════════════════════════════════════════════╝");
    
    // MAP SECTION: Title and bordered container for the exploration map
    stdout.execute(MoveTo(MAP_LEFT, MAP_START_Y))?;
    stdout.execute(SetForegroundColor(Color::Yellow))?;
    print!("🗺️  CARTE DE L'EXOPLANÈTE");
    
    // Calculate map display width (each tile takes 2 characters)
    let map_width = MAP_SIZE as u16 * 2;
    
    // Top border of map container
    stdout.execute(MoveTo(MAP_LEFT, MAP_START_Y + 1))?;
    stdout.execute(SetForegroundColor(Color::DarkGrey))?;
    print!("╔");
    for _ in 0..map_width { print!("═"); }
    print!("╗");
    
    // Side borders for each map row (content will be filled dynamically)
    for y in 0..MAP_SIZE {
        stdout.execute(MoveTo(MAP_LEFT, MAP_START_Y + 2 + y as u16))?;
        print!("║");
        // Fill with spaces (actual map content added dynamically)
        for _ in 0..map_width { print!(" "); }
        print!("║");
    }
    
    // Bottom border of map container
    stdout.execute(MoveTo(MAP_LEFT, MAP_START_Y + 2 + MAP_SIZE as u16))?;
    print!("╚");
    for _ in 0..map_width { print!("═"); }
    print!("╝");
    
    // STATION INFORMATION SECTION: Resource and operational data
    stdout.execute(MoveTo(0, STATION_INFO_Y))?;
    stdout.execute(SetForegroundColor(Color::Yellow))?;
    print!("╔══════════════════════════════════════════════════════════════════════════════╗");
    stdout.execute(MoveTo(0, STATION_INFO_Y + 1))?;
    print!("║                          📡 RAPPORT DE LA STATION                           ║");
    stdout.execute(MoveTo(0, STATION_INFO_Y + 2))?;
    print!("╚══════════════════════════════════════════════════════════════════════════════╝");
    
    // ROBOT STATUS SECTION: Individual robot monitoring
    stdout.execute(MoveTo(0, ROBOTS_INFO_Y))?;
    stdout.execute(SetForegroundColor(Color::Cyan))?;
    print!("╔══════════════════════════════════════════════════════════════════════════════╗");
    stdout.execute(MoveTo(0, ROBOTS_INFO_Y + 1))?;
    print!("║                            🤖 STATUT DES ROBOTS                             ║");
    stdout.execute(MoveTo(0, ROBOTS_INFO_Y + 2))?;
    print!("╚══════════════════════════════════════════════════════════════════════════════╝");
    
    // MISSION LOG SECTION: Recent events and notifications
    stdout.execute(MoveTo(0, LOGS_Y))?;
    stdout.execute(SetForegroundColor(Color::Green))?;
    print!("╔══════════════════════════════════════════════════════════════════════════════╗");
    stdout.execute(MoveTo(0, LOGS_Y + 1))?;
    print!("║                           📋 JOURNAL DE MISSION                             ║");
    stdout.execute(MoveTo(0, LOGS_Y + 2))?;
    print!("╚══════════════════════════════════════════════════════════════════════════════╝");
    
    // Pre-allocate empty lines for log messages (will be filled dynamically)
    for i in 0..8 {
        stdout.execute(MoveTo(0, LOGS_Y + 3 + i))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        print!("{:<80}", ""); // 80-character wide empty line
    }
    
    // LEGEND SECTION: Symbol explanations for map and UI elements
    stdout.execute(MoveTo(0, LEGEND_Y))?;
    stdout.execute(SetForegroundColor(Color::White))?;
    print!("╔══════════════════════════════════════════════════════════════════════════════╗");
    stdout.execute(MoveTo(0, LEGEND_Y + 1))?;
    print!("║                                 📋 LÉGENDE                                  ║");
    stdout.execute(MoveTo(0, LEGEND_Y + 2))?;
    print!("╚══════════════════════════════════════════════════════════════════════════════╝");
    
    // LEGEND CONTENT: Map symbols and their meanings (line 1)
    stdout.execute(MoveTo(0, LEGEND_Y + 3))?;
    stdout.execute(SetForegroundColor(Color::Yellow))?;
    print!("🏠 = Station     ");       // Home base location
    stdout.execute(SetForegroundColor(Color::AnsiValue(9)))?;
    print!("🤖 = Explorateur     ");   // Explorer robot type
    stdout.execute(SetForegroundColor(Color::AnsiValue(10)))?;
    print!("🔋 = Énergie     ");       // Energy collector robot
    stdout.execute(SetForegroundColor(Color::AnsiValue(13)))?;
    print!("⛏️ = Minerais");           // Mineral collector robot
    
    // LEGEND CONTENT: Additional symbols (line 2)
    stdout.execute(MoveTo(0, LEGEND_Y + 4))?;
    stdout.execute(SetForegroundColor(Color::AnsiValue(12)))?;
    print!("🧪 = Scientifique     ");  // Scientific collector robot
    stdout.execute(SetForegroundColor(Color::Green))?;
    print!("💎 = Énergie     ");       // Energy resource tile
    stdout.execute(SetForegroundColor(Color::Magenta))?;
    print!("⭐ = Minerai     ");       // Mineral resource tile
    stdout.execute(SetForegroundColor(Color::Blue))?;
    print!("🔬 = Science     ");       // Scientific resource tile
    stdout.execute(SetForegroundColor(Color::DarkGrey))?;
    print!("❓ = Inexploré");          // Unexplored tile
    
    // USER INSTRUCTIONS: Exit command
    stdout.execute(MoveTo(0, LEGEND_Y + 5))?;
    stdout.execute(SetForegroundColor(Color::Red))?;
    print!("🚨 Ctrl+C pour quitter la mission");
    
    Ok(())
}

/// Updates all dynamic content in the interface (data that changes each frame)
/// 
/// This function refreshes all variable information including:
/// - Status bar metrics
/// - Complete map state with robots and resources
/// - Station operational data
/// - Individual robot status information
/// - Mission log messages
/// 
/// # Parameters
/// * `state` - Current simulation state with all updated data
/// * `display_state` - UI state manager for log handling
/// * `stdout` - Direct terminal output handle
/// 
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or rendering error
fn update_all_dynamic_content(state: &SimulationState, display_state: &mut DisplayState, stdout: &mut std::io::Stdout) -> Result<(), Box<dyn std::error::Error>> {
    // NOTE - Update status bar
    stdout.execute(MoveTo(0, STATUS_Y))?;
    stdout.execute(SetForegroundColor(Color::White))?;
    print!("📊 Cycle: {:>4} | 🌍 Exploration: {:>5.1}% | 🤖 Robots: {:>2} | 🔋 Énergie: {:>3} | ⛏️  Minerais: {:>3} | 🧪 Science: {:>3}        ",
           state.iteration,
           state.station_data.exploration_percentage,
           state.station_data.robot_count,
           state.station_data.energy_reserves,
           state.station_data.collected_minerals,
           state.station_data.collected_scientific_data);
    
    // NOTE - Redraw entire exploration map
    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            stdout.execute(MoveTo(MAP_LEFT + 1 + (x as u16 * 2), MAP_START_Y + 2 + y as u16))?;
            let robot_here = state.robots_data.iter().find(|r| r.x == x && r.y == y);
            if x == state.map_data.station_x && y == state.map_data.station_y {
                // NOTE - Draw station
                stdout.execute(SetForegroundColor(Color::Yellow))?;
                print!("🏠");
            }
            else if let Some(robot) = robot_here {
                // NOTE - Draw robot
                let robot_color = match robot.robot_type {
                    RobotType::Explorer => Color::AnsiValue(9),
                    RobotType::EnergyCollector => Color::AnsiValue(10),
                    RobotType::MineralCollector => Color::AnsiValue(13),
                    RobotType::ScientificCollector => Color::AnsiValue(12),
                };
                stdout.execute(SetForegroundColor(robot_color))?;
                let display_char = match robot.robot_type {
                    RobotType::Explorer => "🤖",
                    RobotType::EnergyCollector => "🔋",
                    RobotType::MineralCollector => "⛏️",
                    RobotType::ScientificCollector => "🧪",
                };
                print!("{}", display_char);
            }
            else {
                // NOTE - Draw terrain/resource or unexplored
                if !state.exploration_data.explored_tiles[y][x] {
                    stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                    print!("❓");
                } else {
                    match &state.map_data.tiles[y][x] {
                        TileType::Empty => {
                            stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                            print!("·");
                        },
                        TileType::Obstacle => {
                            stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                            print!("🧱");
                        },
                        TileType::Energy => {
                            stdout.execute(SetForegroundColor(Color::Green))?;
                            print!("💎");
                        },
                        TileType::Mineral => {
                            stdout.execute(SetForegroundColor(Color::Magenta))?;
                            print!("⭐");
                        },
                        TileType::Scientific => {
                            stdout.execute(SetForegroundColor(Color::Blue))?;
                            print!("🔬");
                        },
                    }
                }
            }
        }
    }
    
    // NOTE - Update station information
    stdout.execute(MoveTo(0, STATION_INFO_Y + 3))?;
    stdout.execute(SetForegroundColor(Color::White))?;
    print!("📊 🔋 Énergie: {:>3} | ⛏️  Minerais: {:>3} | 🧪 Science: {:>3} | ⚔️  Conflits: {:>3}                          ",
           state.station_data.energy_reserves,
           state.station_data.collected_minerals,
           state.station_data.collected_scientific_data,
           state.station_data.conflict_count);
    
    // NOTE - Update robot status (up to 5 robots)
    for i in 0..5 {
        stdout.execute(MoveTo(0, ROBOTS_INFO_Y + 3 + i as u16))?;
        if i < state.robots_data.len() {
            let robot = &state.robots_data[i];
            let robot_color = match robot.robot_type {
                RobotType::Explorer => Color::AnsiValue(9),
                RobotType::EnergyCollector => Color::AnsiValue(10),
                RobotType::MineralCollector => Color::AnsiValue(13),
                RobotType::ScientificCollector => Color::AnsiValue(12),
            };
            stdout.execute(SetForegroundColor(robot_color))?;
            let robot_type_str = match robot.robot_type {
                RobotType::Explorer => "🔍 Explorateur",
                RobotType::EnergyCollector => "⚡ Énergie",
                RobotType::MineralCollector => "⛏️  Minerais",
                RobotType::ScientificCollector => "🧪 Science",
            };
            let mode_str = match robot.mode {
                RobotMode::Exploring => "🚶 Exploration",
                RobotMode::Collecting => "📦 Collecte",
                RobotMode::ReturnToStation => "🏠 Retour",
                RobotMode::Idle => "😴 Repos",
            };
            print!("Robot #{:>2}: {:<12} | 📍({:>2},{:>2}) | 🔋{:>5.1}/{:<5.1} | {} | Min:{:>2} Sci:{:>2} | 📊{:>5.1}%            ",
                   robot.id,
                   robot_type_str,
                   robot.x, robot.y,
                   robot.energy, robot.max_energy,
                   mode_str,
                   robot.minerals,
                   robot.scientific_data,
                   robot.exploration_percentage);
        } else {
            stdout.execute(SetForegroundColor(Color::White))?;
            print!("{:<90}", "");
        }
    }
    
    // NOTE - Update mission log messages
    for (i, log_line) in display_state.log_messages.iter().enumerate() {
        stdout.execute(MoveTo(0, LOGS_Y + 3 + i as u16))?;
        stdout.execute(SetForegroundColor(Color::White))?;
        print!("{:<80}", log_line);
    }
    for i in display_state.log_messages.len()..display_state.max_log_lines {
        stdout.execute(MoveTo(0, LOGS_Y + 3 + i as u16))?;
        print!("{:<80}", "");
    }
    
    Ok(())
}

/// Displays the mission completion victory screen
/// 
/// This function creates a full-screen celebration display when the mission
/// is successfully completed. It shows mission statistics, robot achievements,
/// and automatically exits after 10 seconds.
/// 
/// # Parameters
/// * `state` - Final simulation state containing mission results
/// 
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or display error
fn show_victory_screen(state: &SimulationState) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    
    // NOTE - Triple clear for full screen wipe
    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(MoveTo(0, 0))?;
    stdout.flush()?;
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // NOTE - Render main victory message box
    let center_x = 8;
    let center_y = 2;
    let message_lines = vec![
        "╔════════════════════════════════════════════════════════════════════════╗",
        "║                                                                        ║",
        "║         🎉🚀 MISSION EREEA ACCOMPLIE AVEC SUCCÈS! 🚀🎉              ║",
        "║                                                                        ║",
        "║              🌍 EXOPLANÈTE ENTIÈREMENT EXPLORÉE 🌍                   ║",
        "║                                                                        ║",
        "║                     ✅ OBJECTIFS ATTEINTS ✅                         ║",
        "║                                                                        ║",
        "║               🔍 Exploration complète: 100%                           ║",
        "║               💎 Toutes les ressources collectées                     ║",
        "║               🤖 Tous les robots rapatriés                            ║",
        "║               🏠 Retour sécurisé à la station                         ║",
        "║                                                                        ║",
        "║                        🏆 FÉLICITATIONS! 🏆                          ║",
        "║                                                                        ║",
        "║          L'humanité peut désormais coloniser cette                    ║",
        "║             exoplanète en toute sécurité!                             ║",
        "║                                                                        ║",
        "║                      🌟 MISSION RÉUSSIE 🌟                           ║",
        "║                                                                        ║",
        "║                🚀 Fermeture automatique dans 10s...                   ║",
        "║                                                                        ║",
        "╚════════════════════════════════════════════════════════════════════════╝",
    ];
    for (i, line) in message_lines.iter().enumerate() {
        stdout.execute(MoveTo(center_x, center_y + i as u16))?;
        stdout.execute(SetForegroundColor(Color::Yellow))?;
        print!("{}", line);
    }
    
    // NOTE - Mission statistics section
    let stats_y = center_y + message_lines.len() as u16 + 2;
    stdout.execute(MoveTo(center_x + 15, stats_y))?;
    stdout.execute(SetForegroundColor(Color::Cyan))?;
    print!("🎯 STATISTIQUES DE LA MISSION");
    
    stdout.execute(MoveTo(center_x + 5, stats_y + 2))?;
    stdout.execute(SetForegroundColor(Color::Green))?;
    print!("📊 Exoplanète cartographiée à {:.1}%", state.station_data.exploration_percentage);
    
    stdout.execute(MoveTo(center_x + 5, stats_y + 3))?;
    print!("💎 Minerais collectés: {}", state.station_data.collected_minerals);
    
    stdout.execute(MoveTo(center_x + 5, stats_y + 4))?;
    print!("🧪 Données scientifiques: {}", state.station_data.collected_scientific_data);
    
    stdout.execute(MoveTo(center_x + 5, stats_y + 5))?;
    print!("🤖 Robots déployés: {}", state.robots_data.len());
    
    stdout.execute(MoveTo(center_x + 5, stats_y + 6))?;
    print!("⚔️  Conflits résolus: {}", state.station_data.conflict_count);
    
    stdout.execute(MoveTo(center_x + 5, stats_y + 7))?;
    print!("🕒 Cycles de simulation: {}", state.iteration);
    
    // ROBOT TEAM RECOGNITION SECTION: Celebrate the robotic heroes
    stdout.execute(MoveTo(center_x + 5, stats_y + 9))?;
    stdout.execute(SetForegroundColor(Color::White))?;
    print!("🛠️  ÉQUIPE DE ROBOTS HÉROÏQUE:");
    
    // Display robot type legend with colors
    stdout.execute(MoveTo(center_x + 8, stats_y + 10))?;
    stdout.execute(SetForegroundColor(Color::AnsiValue(9)))?;
    print!("🔍 Explorateurs   ");
    stdout.execute(SetForegroundColor(Color::AnsiValue(10)))?;
    print!("⚡ Collecteurs d'énergie   ");
    stdout.execute(SetForegroundColor(Color::AnsiValue(13)))?;
    print!("⛏️  Collecteurs de minerais");
    
    stdout.execute(MoveTo(center_x + 8, stats_y + 11))?;
    stdout.execute(SetForegroundColor(Color::AnsiValue(12)))?;
    print!("🧪 Collecteurs scientifiques ");
    stdout.execute(SetForegroundColor(Color::Yellow))?;
    print!("- Tous revenus sains et saufs!");
    
    // ANIMATED ROBOT DISPLAY: Visual representation of the successful team
    stdout.execute(MoveTo(center_x + 25, stats_y + 13))?;
    stdout.execute(SetForegroundColor(Color::AnsiValue(9)))?;
    print!("🤖 ");   // Explorer
    stdout.execute(SetForegroundColor(Color::AnsiValue(10)))?;
    print!("🔋 ");   // Energy collector
    stdout.execute(SetForegroundColor(Color::AnsiValue(13)))?;
    print!("⛏️  ");   // Mineral collector
    stdout.execute(SetForegroundColor(Color::AnsiValue(12)))?;
    print!("🧪 ");   // Scientific collector
    stdout.execute(SetForegroundColor(Color::Yellow))?;
    print!("← NOS HÉROS!"); // Hero label
    
    // USER EXIT INSTRUCTIONS
    stdout.execute(MoveTo(center_x + 20, stats_y + 16))?;
    stdout.execute(SetForegroundColor(Color::Red))?;
    print!("Appuyez sur Ctrl+C pour quitter la mission");
    
    // FINAL DECORATIVE SEPARATOR
    stdout.execute(MoveTo(center_x, stats_y + 18))?;
    stdout.execute(SetForegroundColor(Color::Yellow))?;
    print!("════════════════════════════════════════════════════════════════════════");
    
    stdout.flush()?;
    Ok(())
}