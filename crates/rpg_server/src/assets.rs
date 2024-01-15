use crate::state::AppState;

use bevy::{
    asset::{AssetEvent, AssetServer, Assets, Handle},
    ecs::{
        event::{Event, EventReader, EventWriter},
        schedule::NextState,
        system::{Commands, Res, ResMut, Resource},
        world::{FromWorld, World},
    },
};

use rpg_core::metadata::Metadata;
use util::assets::json::JsonSource;

use serde_json::from_slice;

#[derive(Event)]
struct LoadMetadata;

// FIXME dedup
#[derive(Resource)]
pub struct JsonAssets {
    pub item: Handle<JsonSource>,
    pub unit: Handle<JsonSource>,
    pub skill: Handle<JsonSource>,
    pub stat: Handle<JsonSource>,
    pub modifier: Handle<JsonSource>,
    pub level: Handle<JsonSource>,
    pub passive_tree: Handle<JsonSource>,
}

impl FromWorld for JsonAssets {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource_mut::<AssetServer>();

        JsonAssets {
            item: server.load("metadata/item.json"),
            unit: server.load("metadata/unit.json"),
            skill: server.load("metadata/skill.json"),
            stat: server.load("metadata/stats.json"),
            modifier: server.load("metadata/modifiers.json"),
            level: server.load("metadata/level.json"),
            passive_tree: server.load("metadata/passive_tree.json"),
        }
    }
}

#[derive(Resource)]
pub(crate) struct MetadataResources(pub(crate) Metadata);

impl FromWorld for MetadataResources {
    fn from_world(world: &mut World) -> Self {
        let json_sources = world.resource::<Assets<JsonSource>>();
        let json = world.resource::<JsonAssets>();

        let item = from_slice(json_sources.get(&json.item).unwrap().0.as_slice()).unwrap();
        let unit = from_slice(json_sources.get(&json.unit).unwrap().0.as_slice()).unwrap();
        let skill = from_slice(json_sources.get(&json.skill).unwrap().0.as_slice()).unwrap();
        let level = from_slice(json_sources.get(&json.level).unwrap().0.as_slice()).unwrap();
        let stat = from_slice(json_sources.get(&json.stat).unwrap().0.as_slice()).unwrap();
        let modifier = from_slice(json_sources.get(&json.modifier).unwrap().0.as_slice()).unwrap();

        // Passive tree metadata
        let passive_tree =
            from_slice(json_sources.get(&json.passive_tree).unwrap().0.as_slice()).unwrap();

        Self(Metadata {
            item,
            unit,
            skill,
            level,
            stat,
            modifier,
            passive_tree,
        })
    }
}

pub(crate) fn load_metadata(
    mut commands: Commands,
    json_assets: Res<JsonAssets>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if asset_server.is_loaded_with_dependencies(json_assets.item.id())
        && asset_server.is_loaded_with_dependencies(json_assets.unit.id())
        && asset_server.is_loaded_with_dependencies(json_assets.skill.id())
        && asset_server.is_loaded_with_dependencies(json_assets.stat.id())
        && asset_server.is_loaded_with_dependencies(json_assets.modifier.id())
        && asset_server.is_loaded_with_dependencies(json_assets.level.id())
        && asset_server.is_loaded_with_dependencies(json_assets.level.id())
        && asset_server.is_loaded_with_dependencies(json_assets.passive_tree.id())
    {
        commands.init_resource::<MetadataResources>();

        next_state.set(AppState::Lobby);

        println!("json loaded, initializing metadata");
    }
}
