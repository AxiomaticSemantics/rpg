use super::{
    actor::{player::Player, unit::Unit},
    item::UnitStorage,
    passive_tree::passive_tree::PassiveTree,
    plugin::{GameOverState, GameState, PlayState},
};
use crate::state::AppState;

use bevy::ecs::{
    event::{Event, EventReader},
    query::With,
    schedule::NextState,
    system::{Query, Res, ResMut},
};

use util::cleanup::CleanupStrategy;

use std::{
    env,
    fs::{self, File},
    io,
    path::Path,
};

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

    let root = env::var("RPG_SAVE_ROOT").unwrap();

    let unit_file = match create_fs_open_options(Path::new(&format!("{}/0-unit.json", root))) {
        Ok(file) => file,
        Err(e) => panic!("failed to open unit file `{e:?}`"),
    };

    match serde_json::to_writer(unit_file, &unit.0) {
        Ok(_) => println!("unit saved"),
        Err(e) => panic!("failed to serialize unit: `{e:?}`"),
    }

    let storage_file = match create_fs_open_options(Path::new(&format!("{}/0-storage.json", root)))
    {
        Ok(file) => file,
        Err(e) => panic!("failed to open storage file: `{e:?}`"),
    };

    match serde_json::to_writer(storage_file, &storage.0) {
        Ok(_) => println!("storage saved"),
        Err(e) => panic!("failed to serialize storage: `{e:?}`"),
    }

    let passive_tree_file =
        match create_fs_open_options(Path::new(&format!("{}/0-passive.json", root))) {
            Ok(file) => file,
            Err(e) => panic!("failed to open passive tree file: `{e:?}`"),
        };

    match serde_json::to_writer(passive_tree_file, &passive_tree.0) {
        Ok(_) => println!("passive skill graph saved"),
        Err(e) => panic!("failed to serialize passive tree: `{e:?}`"),
    }

    game_state.state = PlayState::GameOver(GameOverState::Exit);

    state.set(AppState::GameCleanup);
}

fn create_fs_open_options(path: &Path) -> Result<File, io::Error> {
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
}
