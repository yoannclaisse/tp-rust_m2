# EREEA - Exploration Robotique d'Exoplanètes Autonome

## Documentation Technique Détaillée

---

## Table des Matières

1. [Vue d'ensemble et architecture](#1-vue-densemble-et-architecture)
2. [Organisation des modules et responsabilités](#2-organisation-des-modules-et-responsabilités)
3. [Cycle d'exécution complet (serveur et client)](#3-cycle-dexécution-complet-serveur-et-client)
4. [Appels de fonctions et interactions détaillées](#4-appels-de-fonctions-et-interactions-détaillées)
5. [Structures de données et flux d'information](#5-structures-de-données-et-flux-dinformation)
6. [Algorithmes principaux et logique métier](#6-algorithmes-principaux-et-logique-métier)
7. [Communication réseau et sérialisation](#7-communication-réseau-et-sérialisation)
8. [Conditions de victoire et métriques](#8-conditions-de-victoire-et-métriques)

---

## 1. Vue d'ensemble et architecture

EREEA simule une mission d'exploration robotique sur une exoplanète, avec une architecture **client-serveur** :

- **Serveur** (`simulation.rs`) : exécute la simulation, gère la carte, la station, les robots, et diffuse l'état.
- **Client** (`earth.rs`) : reçoit l'état, affiche la carte, les robots, la station, et les logs en temps réel.

### Schéma général

```
+---------------------+      TCP/JSON      +----------------------+
|   EARTH (Client)    | <----------------> |  SIMULATION (Server) |
+---------------------+                    +----------------------+
```

---

## 2. Organisation des modules et responsabilités

### Modules principaux

- **types.rs** : Définit les types de base (tuiles, robots, modes, constantes).
- **map.rs** : Génère la carte, fournit l'accès aux tuiles, vérifie l'accessibilité, consomme les ressources.
- **robot.rs** : Définit la structure et le comportement des robots (exploration, collecte, IA, mémoire).
- **station.rs** : Gère la station (ressources, création de robots, mémoire globale, synchronisation).
- **display.rs** : Affichage local (pour mode terminal ou client).
- **network/mod.rs** : Sérialisation/désérialisation, structures de données réseau, conversion des états.
- **bin/simulation.rs** : Point d'entrée serveur, boucle principale, gestion du multithreading et du réseau.
- **bin/earth.rs** : Point d'entrée client, boucle de réception, rendu de l'interface.

### Dépendances et flux d'appel

- `simulation.rs` (appelé par l'utilisateur) :
  - Crée `Map`, `Station`, `Robot`
  - Boucle principale : appelle `station.tick()`, puis pour chaque robot `robot.update(map, station)`
  - Après chaque cycle, appelle `create_simulation_state(map, station, robots, iteration)` (network)
  - Diffuse l'état via TCP

- `robot.rs` :
  - `update(map, station)` : cœur de la logique robot, appelle :
    - `update_memory(map, station)`
    - `should_return_to_station(map)`
    - `plan_path_to_station(map)`
    - `find_nearest_resource(map)`
    - `find_path(map, target)`
    - `move_to(x, y)`
    - `collect_resources(map)`
    - `station.deposit_resources(...)`
    - `station.share_knowledge(self)`
  - Selon le mode (`RobotMode`), la logique diverge (exploration, collecte, retour, idle)

- `station.rs` :
  - `tick()` : incrémente l'horloge
  - `try_create_robot(map)` : décide du type de robot à créer, consomme les ressources, retourne un nouveau `Robot`
  - `share_knowledge(robot)` : synchronise la mémoire du robot et de la station (résolution de conflits)
  - `deposit_resources(minerals, science)` : ajoute les ressources à la station
  - `is_mission_complete(map)` : vérifie la fin de mission (plus de ressources sur la carte)
  - `is_all_missions_complete(map, robots)` : vérifie la victoire parfaite (tout exploré, tous les robots à la base)

- `map.rs` :
  - `new()` : génère la carte procédurale (Perlin), place la station, assure l'accessibilité des ressources
  - `get_tile(x, y)` : retourne le type de tuile
  - `is_valid_position(x, y)` : vérifie si une case est franchissable
  - `consume_resource(x, y)` : supprime une ressource collectée

- `network/mod.rs` :
  - Définit les structures de données réseau (`SimulationState`, etc.)
  - Fournit : `create_map_data`, `create_robot_data`, `create_station_data`, `create_exploration_data`, `create_simulation_state`
  - Sérialise/désérialise en JSON

- `earth.rs` :
  - Boucle principale : lit les états du serveur, désérialise, appelle `render_interface(state, display_state)`
  - Affiche la carte, les robots, la station, les logs, la victoire

---

## 3. Cycle d'exécution complet (serveur et client)

### Serveur (`simulation.rs`)

1. **Initialisation** :
    - Génère la carte (`Map::new`)
    - Crée la station (`Station::new`)
    - Crée les robots initiaux (`Robot::new_with_memory`)
2. **Boucle principale** :
    - `station.tick()`
    - Pour chaque robot : `robot.update(&mut map, &mut station)`
        - Peut modifier la carte (collecte), la station (dépôt, sync), sa propre mémoire
    - Vérifie la fin de mission (`station.is_mission_complete(&map)`)
    - Tente de créer un robot (`station.try_create_robot(&map)`)
    - Prépare l'état réseau (`create_simulation_state`)
    - Diffuse l'état à tous les clients via TCP
3. **Arrêt** : quand la mission est terminée

### Client (`earth.rs`)

1. **Connexion** : se connecte au serveur TCP
2. **Boucle principale** :
    - Lit chaque ligne JSON (état complet)
    - Désérialise en `SimulationState`
    - Si mission terminée : affiche l'écran de victoire, quitte
    - Sinon : appelle `render_interface(state, display_state)` pour afficher la carte, robots, station, logs
3. **Arrêt** : sur Ctrl+C ou fin de transmission

---

## 4. Appels de fonctions et interactions détaillées

### Appel typique lors d'un cycle serveur

```
simulation.rs (main)
│
├─> station.tick()
│
├─> for robot in robots:
│     └─> robot.update(map, station)
│           ├─> robot.update_memory(map, station)
│           ├─> robot.should_return_to_station(map)
│           ├─> robot.plan_path_to_station(map)
│           ├─> robot.find_nearest_resource(map)
│           ├─> robot.find_path(map, target)
│           ├─> robot.move_to(x, y)
│           ├─> robot.collect_resources(map)
│           │     └─> map.consume_resource(x, y)
│           ├─> map.get_tile(x, y)
│           ├─> map.is_valid_position(x, y)
│           ├─> station.deposit_resources(minerals, science)
│           └─> station.share_knowledge(robot)
│
├─> station.try_create_robot(map)
│     ├─> station.determine_needed_robot_type(map)
│     └─> Robot::new_with_memory(...)
│
├─> station.is_mission_complete(map)
│
└─> create_simulation_state(map, station, robots, iteration)
      ├─> create_map_data(map)
      ├─> create_robot_data(robot) pour chaque robot
      ├─> create_station_data(station, map)
      └─> create_exploration_data(station)
```

### Appel typique côté client

```
earth.rs (main)
│
├─> Boucle: pour chaque état reçu
│     ├─> Désérialise JSON en SimulationState
│     ├─> Si mission_complete: show_victory_screen(state)
│     └─> Sinon: render_interface(state, display_state)
│           ├─> Affiche la carte (avec robots, station, ressources)
│           ├─> Affiche les infos station et robots
│           └─> Affiche les logs et la légende
```

---

## 5. Structures de données et flux d'information

### Carte (`Map`)

- `tiles: Vec<Vec<TileType>>` : grille 2D de tuiles
- `station_x, station_y` : position de la station

### Robot (`Robot`)

- Position, énergie, inventaire, type, mode
- `memory: Vec<Vec<TerrainData>>` : mémoire locale (exploration, timestamp, robot_id/type)
- `path_to_station: VecDeque<(usize, usize)>` : chemin planifié (A*)
- `id`, `home_station_x/y`, `last_sync_time`, etc.

### Station (`Station`)

- Ressources (énergie, minerais, science)
- `global_memory: Vec<Vec<TerrainData>>` : mémoire partagée (fusionnée avec les robots)
- `conflict_count`, `next_robot_id`, `current_time`

### Réseau (`SimulationState`)

- `map_data`, `robots_data`, `station_data`, `exploration_data`, `iteration`
- Sérialisé/désérialisé en JSON pour transmission

---

## 6. Algorithmes principaux et logique métier

### Génération de carte (Perlin)

- Génère une grille bruitée, attribue les tuiles selon des seuils
- Zone libre autour de la station
- Vérifie l’accessibilité de chaque ressource (BFS), crée un chemin si besoin

### IA des robots

- **Explorateur** : cherche les cases non explorées sur toute la carte, planifie un chemin (A*), sinon mouvement intelligent
- **Collecteurs** : cherchent la ressource la plus proche de leur type, collectent, retournent à la station si inventaire plein ou énergie faible
- **Machine à états** : chaque robot a un `mode` (Exploring, Collecting, ReturnToStation, Idle) qui détermine son comportement

### Synchronisation mémoire (Git-like)

- À chaque retour à la station, le robot fusionne sa mémoire avec la station (résolution par timestamp)
- La station met à jour sa mémoire globale, puis la renvoie au robot

### Navigation (A*)

- Recherche du chemin optimal entre deux points, évite les obstacles
- Heuristique : distance de Manhattan

---

## 7. Communication réseau et sérialisation

- **Serveur** : sérialise l’état complet (`SimulationState`) en JSON, diffuse à tous les clients connectés via TCP
- **Client** : lit chaque ligne JSON, désérialise, met à jour l’interface
- **Structures réseau** : `MapData`, `RobotData`, `StationData`, `ExplorationData`, `SimulationState`

---

## 8. Conditions de victoire et métriques

- **Mission complète** : toutes les ressources collectées (plus aucune tuile Energy, Mineral, Scientific sur la carte)
- **Mission parfaite** : 100% de la carte explorée, tous les robots à la station en mode Idle, aucune ressource restante
- **Affichage** : écran de victoire détaillé côté client

---

## Résumé du flux d'appel

- **simulation.rs** : boucle principale → station.tick() → robots.update() → station.try_create_robot() → create_simulation_state() → diffusion TCP
- **robot.rs** : update() → (selon mode) → planification, déplacement, collecte, synchronisation mémoire
- **station.rs** : tick(), try_create_robot(), share_knowledge(), deposit_resources(), is_mission_complete()
- **map.rs** : génération, accès tuiles, validation, consommation ressources
- **network/mod.rs** : conversion des états, sérialisation/désérialisation
- **earth.rs** : réception état, affichage, gestion logs, écran de victoire

---

Ce document vise à donner une vision claire et exhaustive du fonctionnement interne du projet, des interactions entre modules, et du flux d'exécution complet, pour faciliter la compréhension, la maintenance et l'évolution du code.
