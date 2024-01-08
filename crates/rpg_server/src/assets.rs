use bevy::{
    asset::{AssetServer, Handle},
    ecs::{
        system::Resource,
        world::{FromWorld, World},
    },
};

use util::assets::json::JsonSource;

// FIXME dedup
#[derive(Resource)]
pub struct JsonAssets {
    pub item: Handle<JsonSource>,
    pub unit: Handle<JsonSource>,
    pub skill: Handle<JsonSource>,
    pub stat: Handle<JsonSource>,
    pub modifier: Handle<JsonSource>,
    pub level: Handle<JsonSource>,
    pub prop: Handle<JsonSource>,
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
            prop: server.load("metadata/prop.json"),
            passive_tree: server.load("metadata/passive_tree.json"),
        }
    }
}
