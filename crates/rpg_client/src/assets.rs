use bevy::{
    asset::{AssetServer, Handle},
    audio::AudioSource,
    ecs::{
        system::Resource,
        world::{FromWorld, World},
    },
    render::texture::Image,
};

use util::assets::json::JsonSource;

use std::collections::HashMap;

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
    pub prop: Handle<JsonSource>,
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
            prop: server.load("metadata/prop.json"),
        }
    }
}

#[derive(Resource)]
pub struct TextureAssets {
    pub icons: HashMap<&'static str, Handle<Image>>,
    pub bevy_logo: Handle<Image>,
}

impl FromWorld for TextureAssets {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource_mut::<AssetServer>();

        let mut icons = HashMap::new();
        icons.insert("checkmark", server.load("textures/icons/checkmark.png"));
        icons.insert("transparent", server.load("textures/icons/transparent.png"));
        icons.insert("frame", server.load("ui/frame.png"));
        icons.insert("p_circle", server.load("ui/circle_512.png"));

        TextureAssets {
            icons,
            bevy_logo: server.load("textures/bevy.png"),
        }
    }
}

#[derive(Resource)]
pub struct AudioAssets {
    pub background_tracks: HashMap<&'static str, Handle<AudioSource>>,
    pub foreground_tracks: HashMap<&'static str, Handle<AudioSource>>,
}

impl FromWorld for AudioAssets {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource_mut::<AssetServer>();

        let mut background_tracks = HashMap::new();
        background_tracks.insert("bg_loop1", server.load("audio/bg_loop1.ogg"));
        background_tracks.insert("bg_loop2", server.load("audio/bg_loop2.ogg"));
        background_tracks.insert("bg_loop3", server.load("audio/bg_loop3.ogg"));
        background_tracks.insert("bg_loop4", server.load("audio/bg_loop4.ogg"));
        background_tracks.insert("bg_loop5", server.load("audio/bg_loop5.ogg"));
        background_tracks.insert("bg_loop6", server.load("audio/bg_loop6.ogg"));
        background_tracks.insert("bg_loop7", server.load("audio/bg_loop7.ogg"));
        background_tracks.insert("bg_loop8", server.load("audio/bg_loop8.ogg"));

        background_tracks.insert("env_swamp", server.load("audio/env_swamp.ogg"));

        let mut foreground_tracks = HashMap::new();
        //foreground_tracks.insert("walk_grass_01", server.load("audio/walk_grass_01.ogg"));
        //foreground_tracks.insert("walk_rock_01", server.load("audio/walk_rock_01.ogg"));

        foreground_tracks.insert("hit_blocked", server.load("audio/hit_blocked.ogg"));
        foreground_tracks.insert("hit_death", server.load("audio/creature_die_01.ogg"));
        foreground_tracks.insert("hit_soft", server.load("audio/creature_hurt_01.ogg"));
        foreground_tracks.insert("hit_hard", server.load("audio/creature_hurt_02.ogg"));

        foreground_tracks.insert("attack_proj1", server.load("audio/spell_01.ogg"));
        foreground_tracks.insert("attack_proj2", server.load("audio/spell_02.ogg"));

        foreground_tracks.insert("item_drop_gem", server.load("audio/item_drop_gem.ogg"));
        foreground_tracks.insert(
            "item_drop_potion",
            server.load("audio/item_drop_potion.ogg"),
        );

        foreground_tracks.insert("item_pickup", server.load("audio/item_pickup.ogg"));

        Self {
            background_tracks,
            foreground_tracks,
        }
    }
}
