use crate::state::AppState;

use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        schedule::NextState,
        system::{Commands, Res, ResMut, Resource},
        world::{FromWorld, World},
    },
    log::info,
};

use rpg_core::metadata::Metadata as RpgMetadata;
use rpg_world::metadata::Metadata as WorldMetadata;

use util::assets::json::JsonSource;

use serde_json::from_slice;

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
    pub zone: Handle<JsonSource>,
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
            zone: server.load("metadata/zone.json"),
        }
    }
}

#[derive(Resource)]
pub(crate) struct MetadataResources {
    pub(crate) rpg: RpgMetadata,
    pub(crate) world: WorldMetadata,
}

pub(crate) fn load_metadata(
    mut commands: Commands,
    mut json_sources: ResMut<Assets<JsonSource>>,
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
        && asset_server.is_loaded_with_dependencies(json_assets.passive_tree.id())
        && asset_server.is_loaded_with_dependencies(json_assets.zone.id())
    {
        commands.insert_resource(MetadataResources {
            rpg: RpgMetadata {
                item: from_slice(json_sources.get(&json_assets.item).unwrap().0.as_slice())
                    .unwrap(),
                unit: from_slice(json_sources.get(&json_assets.unit).unwrap().0.as_slice())
                    .unwrap(),
                skill: from_slice(json_sources.get(&json_assets.skill).unwrap().0.as_slice())
                    .unwrap(),
                level: from_slice(json_sources.get(&json_assets.level).unwrap().0.as_slice())
                    .unwrap(),
                stat: from_slice(json_sources.get(&json_assets.stat).unwrap().0.as_slice())
                    .unwrap(),
                modifier: from_slice(
                    json_sources
                        .get(&json_assets.modifier)
                        .unwrap()
                        .0
                        .as_slice(),
                )
                .unwrap(),
                passive_tree: from_slice(
                    json_sources
                        .get(&json_assets.passive_tree)
                        .unwrap()
                        .0
                        .as_slice(),
                )
                .unwrap(),
            },
            world: WorldMetadata {
                zone: from_slice(json_sources.get(&json_assets.zone).unwrap().0.as_slice())
                    .unwrap(),
            },
        });

        json_sources.remove(json_assets.item.id());
        json_sources.remove(json_assets.unit.id());
        json_sources.remove(json_assets.skill.id());
        json_sources.remove(json_assets.stat.id());
        json_sources.remove(json_assets.modifier.id());
        json_sources.remove(json_assets.level.id());
        json_sources.remove(json_assets.passive_tree.id());
        json_sources.remove(json_assets.zone.id());

        commands.remove_resource::<JsonAssets>();

        info!("metadata initialized");

        next_state.set(AppState::Lobby);
    }
}
