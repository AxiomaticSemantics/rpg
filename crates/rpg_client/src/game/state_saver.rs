use super::{
    actor::{player::Player, unit::Unit},
    item::UnitStorage,
    passive_tree::passive_tree::PassiveTree,
    plugin::{GameOverState, GameState, PlayState},
};
use crate::{menu::plugin::MenuCamera, state::AppState};

use bevy::{
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        event::{Event, EventReader},
        query::With,
        schedule::NextState,
        system::{Commands, Query, Res, ResMut},
    },
};

use util::cleanup::CleanupStrategy;

use std::{fs, path::Path};

#[derive(Event)]
pub struct SaveGame;

pub fn save_character(
    mut state: ResMut<NextState<AppState>>,
    mut save_event: EventReader<SaveGame>,
    mut game_state: ResMut<GameState>,
    passive_tree: Res<PassiveTree>,
    player_q: Query<(&Unit, &UnitStorage), With<Player>>,
) {
    if save_event.is_empty() {
        return;
    }

    save_event.clear();

    let (unit, storage) = player_q.single();

    let root = std::env::var("RPG_SAVE_ROOT").unwrap();

    let unit_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(Path::new(&format!("{}/0-unit.json", root)))
        .unwrap();

    if let Ok(_) = serde_json::to_writer(unit_file, &unit.0) {
        println!("unit saved");
    } else {
        println!("error saving unit");
    }

    let storage_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(Path::new(&format!("{}/0-storage.json", root)))
        .unwrap();

    if let Ok(_) = serde_json::to_writer(storage_file, &storage.0) {
        println!("storage saved");
    } else {
        println!("error saving storage");
    }

    let passive_tree_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(Path::new(&format!("{}/0-passive.json", root)))
        .unwrap();

    match serde_json::to_writer(passive_tree_file, &passive_tree.0) {
        Ok(_) => {
            println!("passive skill graph saved");
        }
        Err(e) => {
            println!("error saving passive skill graph: {e:?}");
        }
    }

    game_state.state = PlayState::GameOver(GameOverState::Exit);

    state.set(AppState::GameCleanup);
}
