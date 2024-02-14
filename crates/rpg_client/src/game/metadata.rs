use super::prop::metadata::Metadata as PropMetadata;
use crate::assets::JsonAssets;

use rpg_core::metadata::Metadata;
use rpg_world::metadata::Metadata as WorldMetadata;
use util::assets::json::JsonSource;

use bevy::{
    asset::Assets,
    ecs::{
        system::Resource,
        world::{FromWorld, World},
    },
};

use serde_json::from_slice;

#[derive(Resource)]
pub(crate) struct MetadataResources {
    pub(crate) rpg: Metadata,
    pub(crate) world: WorldMetadata,
    pub(crate) props: PropMetadata,
}

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

        let zone = from_slice(json_sources.get(&json.zone).unwrap().0.as_slice()).unwrap();
        // Passive tree metadata
        let passive_tree =
            from_slice(json_sources.get(&json.passive_tree).unwrap().0.as_slice()).unwrap();

        Self {
            rpg: Metadata {
                item,
                unit,
                skill,
                level,
                stat,
                modifier,
                passive_tree,
            },
            world: WorldMetadata { zone },
            //passive_tree: PassiveTreeMetadata { passive_tree },
            props: PropMetadata::default(),
        }
    }
}
