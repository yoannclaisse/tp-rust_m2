use crate::types::{TileType, MAP_SIZE};
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
use crossterm::{
    style::{Color, SetForegroundColor, SetBackgroundColor, ResetColor, Print},
    ExecutableCommand,
};
use std::io::{self, Write};

pub struct Map {
    pub tiles: Vec<Vec<TileType>>,
    pub station_x: usize,
    pub station_y: usize,
}

impl Map {
    pub fn new() -> Self {
        let seed: u32 = rand::thread_rng().r#gen();
        let perlin = Perlin::new(seed);
        let mut tiles = vec![vec![TileType::Empty; MAP_SIZE]; MAP_SIZE];
        
        let station_x = MAP_SIZE / 2;
        let station_y = MAP_SIZE / 2;
        
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                let nx = x as f64 / MAP_SIZE as f64;
                let ny = y as f64 / MAP_SIZE as f64;
                let value = perlin.get([nx * 4.0, ny * 4.0]);
                
                tiles[y][x] = if value > 0.5 {
                    TileType::Obstacle
                } else if value > 0.3 {
                    TileType::Energy
                } else if value > 0.1 {
                    TileType::Mineral
                } else if value > 0.0 {
                    TileType::Scientific
                } else {
                    TileType::Empty
                };
            }
        }
        
        // Station area clear
        for dy in -2..=2 {
            for dx in -2..=2 {
                let sx = (station_x as isize + dx).clamp(0, MAP_SIZE as isize - 1) as usize;
                let sy = (station_y as isize + dy).clamp(0, MAP_SIZE as isize - 1) as usize;
                tiles[sy][sx] = TileType::Empty;
            }
        }
        
        Self {
            tiles,
            station_x,
            station_y,
        }
    }
    
    pub fn get_tile(&self, x: usize, y: usize) -> TileType {
        self.tiles[y][x].clone()
    }
    
    pub fn is_valid_position(&self, x: usize, y: usize) -> bool {
        x < MAP_SIZE && y < MAP_SIZE && self.tiles[y][x] != TileType::Obstacle
    }

    pub fn display(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        
        stdout.execute(Print("\n=== Carte générée ===\n"))?;
        stdout.execute(Print(format!("Station: ({}, {})\n", self.station_x, self.station_y)))?;
        stdout.execute(Print("Légende: [S]tation | [.]Vide | [#]Obstacle | [E]nergie | [M]inéral | [?]Scientifique\n\n"))?;
        
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                if x == self.station_x && y == self.station_y {
                    // Station en rouge avec fond
                    stdout.execute(SetForegroundColor(Color::Red))?;
                    stdout.execute(SetBackgroundColor(Color::White))?;
                    stdout.execute(Print("S"))?;
                    stdout.execute(ResetColor)?;
                } else {
                    match self.tiles[y][x] {
                        TileType::Empty => {
                            stdout.execute(SetForegroundColor(Color::DarkGrey))?;
                            stdout.execute(Print("."))?;
                        },
                        TileType::Obstacle => {
                            stdout.execute(SetForegroundColor(Color::Black))?;
                            stdout.execute(SetBackgroundColor(Color::DarkGrey))?;
                            stdout.execute(Print("#"))?;
                            stdout.execute(ResetColor)?;
                        },
                        TileType::Energy => {
                            stdout.execute(SetForegroundColor(Color::Yellow))?;
                            stdout.execute(Print("E"))?;
                        },
                        TileType::Mineral => {
                            stdout.execute(SetForegroundColor(Color::Blue))?;
                            stdout.execute(Print("M"))?;
                        },
                        TileType::Scientific => {
                            stdout.execute(SetForegroundColor(Color::Green))?;
                            stdout.execute(Print("?"))?;
                        },
                    }
                    stdout.execute(ResetColor)?;
                }
            }
             // Nouvelle ligne
            stdout.execute(Print("\n"))?;
        }
        // Ligne vide à la fin
        stdout.execute(Print("\n"))?;
        stdout.flush()?;
        Ok(())
    }
}