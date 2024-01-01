use super::{
    actor::{player::Player, unit::Unit},
    item::UnitStorage,
    passive_tree::PassiveTree,
    plugin::{GameOverState, GameState, PlayState},
};
use crate::state::AppState;

use bevy::ecs::{
    component::Component,
    event::{Event, EventReader},
    query::With,
    schedule::NextState,
    system::{Query, ResMut, Resource},
};

use rpg_core::{
    passive_tree::PassiveSkillGraph, storage::UnitStorage as RpgUnitStorage, unit::Unit as RpgUnit,
};

use std::{
    env,
    fs::{self, File},
    io,
    path::Path,
};

#[derive(Event)]
pub struct SaveGame;

#[derive(Event)]
pub struct LoadCharacter(pub u8);

pub struct CharacterState {
    pub unit: RpgUnit,
    pub storage: RpgUnitStorage,
    pub passive_tree: PassiveSkillGraph,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Component)]
pub struct SaveSlotId(pub u8);

pub struct SaveSlot {
    pub slot: u8,
    pub state: Option<CharacterState>,
}

#[derive(Resource)]
pub struct SaveSlots {
    pub slots: Vec<SaveSlot>,
}

impl Default for SaveSlots {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveSlots {
    pub fn new() -> Self {
        let mut slots = Vec::with_capacity(12);
        for i in 0..12 {
            slots.push(SaveSlot {
                slot: i,
                state: None,
            });
        }

        Self { slots }
    }
}

pub fn load_save_slots(mut save_slots: ResMut<SaveSlots>) {
    for slot in 0..12 {
        let save_slot = &mut save_slots.slots[slot];

        let root = env::var("RPG_SAVE_ROOT").unwrap();

        let unit_file = match open_read(Path::new(&format!("{}/{slot}-unit.json", root))) {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => panic!("failed to open unit file `{e:?}`"),
        };

        let unit = serde_json::from_reader(unit_file);
        let unit = match unit {
            Ok(unit) => unit,
            Err(e) => panic!("Expected valid unit file {e}"),
        };

        let storage_file = match open_read(Path::new(&format!("{}/{slot}-storage.json", root))) {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => panic!("failed to open storage file `{e:?}`"),
        };

        let storage = serde_json::from_reader(storage_file);
        let storage = match storage {
            Ok(storage) => storage,
            Err(e) => panic!("Expected valid storage file `{e:?}`"),
        };

        let passive_file = match open_read(Path::new(&format!("{}/{slot}-passive.json", root))) {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => panic!("failed to open passive file `{e:?}`"),
        };

        let passive_tree = serde_json::from_reader(passive_file);
        let passive_tree = match passive_tree {
            Ok(tree) => tree,
            Err(e) => panic!("Expected valid passive file `{e:?}`"),
        };

        save_slot.state = Some(CharacterState {
            unit,
            storage,
            passive_tree,
        });
    }
}

pub fn save_character(
    mut state: ResMut<NextState<AppState>>,
    mut save_event: EventReader<SaveGame>,
    mut game_state: ResMut<GameState>,
    player_q: Query<(&Unit, &UnitStorage, &PassiveTree), With<Player>>,
) {
    if save_event.is_empty() {
        return;
    }

    save_event.clear();

    let (unit, storage, passive_tree) = player_q.single();

    let root = env::var("RPG_SAVE_ROOT").unwrap();

    let unit_file = match open_write(Path::new(&format!("{}/0-unit.json", root))) {
        Ok(file) => file,
        Err(e) => panic!("failed to open unit file `{e:?}`"),
    };

    match serde_json::to_writer(unit_file, &unit.0) {
        Ok(_) => println!("unit saved"),
        Err(e) => panic!("failed to serialize unit: `{e:?}`"),
    }

    let storage_file = match open_write(Path::new(&format!("{}/0-storage.json", root))) {
        Ok(file) => file,
        Err(e) => panic!("failed to open storage file: `{e:?}`"),
    };

    match serde_json::to_writer(storage_file, &storage.0) {
        Ok(_) => println!("storage saved"),
        Err(e) => panic!("failed to serialize storage: `{e:?}`"),
    }

    let passive_tree_file = match open_write(Path::new(&format!("{}/0-passive.json", root))) {
        Ok(file) => file,
        Err(e) => panic!("failed to open passive tree file: `{e:?}`"),
    };

    match serde_json::to_writer(passive_tree_file, &passive_tree.0) {
        Ok(_) => println!("passive skill graph saved"),
        Err(e) => panic!("failed to serialize passive tree: `{e:?}`"),
    }

    game_state.state = PlayState::GameOver(GameOverState::Saved);

    state.set(AppState::GameCleanup);
}

fn open_read(path: &Path) -> Result<File, io::Error> {
    fs::OpenOptions::new().read(true).open(path)
}

fn open_write(path: &Path) -> Result<File, io::Error> {
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
}
