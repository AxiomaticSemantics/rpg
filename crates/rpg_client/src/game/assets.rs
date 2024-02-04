use super::{
    actor::{ActorHandle, ActorInfo},
    prop::{PropHandle, PropInfo},
};

use bevy::{
    asset::{AssetServer, Assets, Handle},
    ecs::{
        system::Resource,
        world::{FromWorld, World},
    },
    math::{bounding::Aabb3d, Vec2, Vec3},
    pbr::{AlphaMode, StandardMaterial},
    render::{
        color::Color,
        mesh::{shape, Mesh},
        texture::Image,
    },
    sprite::ColorMaterial,
    utils::default,
};

use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Resource)]
pub(crate) struct RenderResources {
    pub color_materials: HashMap<Cow<'static, str>, Handle<ColorMaterial>>,
    pub materials: HashMap<Cow<'static, str>, Handle<StandardMaterial>>,
    pub meshes: HashMap<Cow<'static, str>, Handle<Mesh>>,
    pub aabbs: HashMap<Cow<'static, str>, Aabb3d>,
    pub actors: HashMap<Cow<'static, str>, ActorInfo>,
    pub props: HashMap<Cow<'static, str>, PropInfo>,
}

impl FromWorld for RenderResources {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = HashMap::<Cow<'static, str>, Handle<Mesh>>::new();

        let mut mesh_assets = world.resource_mut::<Assets<Mesh>>();
        let bar_mesh_outer = mesh_assets.add(shape::Quad::new(Vec2::new(0.792857, 0.102857)));
        meshes.insert(Cow::Owned("bar_outer".into()), bar_mesh_outer);

        let bar_mesh_inner = mesh_assets.add(shape::Quad::new(Vec2::new(0.75, 0.06)));
        meshes.insert(Cow::Owned("bar_inner".into()), bar_mesh_inner);

        let mut aabbs = HashMap::new();
        aabbs.insert(
            Cow::Owned("direct_attack".into()),
            Aabb3d {
                min: Vec3::new(-0.1, -0.1, -0.5),
                max: Vec3::new(0.1, 0.1, 0.5),
            },
        );
        aabbs.insert(
            Cow::Owned("bolt_01".into()),
            Aabb3d {
                min: Vec3::new(-0.1, -0.1, -0.25),
                max: Vec3::new(0.1, 0.1, 0.25),
            },
        );

        let mut textures = HashMap::<&'static str, Handle<Image>>::new();

        let asset_server = world.resource_mut::<AssetServer>();
        textures.insert("aura_01", asset_server.load("textures/aura_01.png"));

        textures.insert(
            "aura_02_c",
            asset_server.load("textures/lava/lava_01_color.png"),
        );
        textures.insert(
            "aura_02_e",
            asset_server.load("textures/lava/lava_01_emission.png"),
        );
        textures.insert(
            "aura_02_n",
            asset_server.load("textures/lava/lava_01_normal.png"),
        );

        textures.insert(
            "tile_color",
            asset_server.load("textures/tiles/tile_color.png"),
        );
        textures.insert(
            "tile_normal",
            asset_server.load("textures/tiles/tile_normal.png"),
        );
        textures.insert(
            "tile1_color",
            asset_server.load("textures/tiles/tile1_color.png"),
        );
        textures.insert(
            "tile1_normal",
            asset_server.load("textures/tiles/tile1_normal.png"),
        );

        let color_materials = HashMap::<Cow<'static, str>, Handle<ColorMaterial>>::new();
        let mut materials = HashMap::<Cow<'static, str>, Handle<StandardMaterial>>::new();

        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();

        materials.insert(
            "glow_red".into(),
            material_assets.add(StandardMaterial {
                base_color: Color::rgba(0.8, 0.1, 0.1, 0.8),
                emissive: Color::rgb_linear(0.6, 0., 0.),
                ..default()
            }),
        );

        materials.insert(
            "glow_green".into(),
            material_assets.add(StandardMaterial {
                base_color: Color::rgba(0.2, 0.9, 0.2, 0.5),
                emissive: Color::rgba(0.6, 0.2, 0.2, 0.8),
                ..default()
            }),
        );

        materials.insert(
            "bar_frame".into(),
            material_assets.add(StandardMaterial {
                base_color: Color::rgba(0.2, 0.2, 0.2, 0.98),
                unlit: true,
                ..default()
            }),
        );

        materials.insert(
            "bar_fill_hp".into(),
            material_assets.add(StandardMaterial {
                base_color: Color::rgba(
                    0xfa as f32 / 0xff as f32,
                    0x50 as f32 / 0xff as f32,
                    0x50 as f32 / 0xff as f32,
                    0xf7 as f32 / 0xff as f32,
                ),
                unlit: true,
                ..default()
            }),
        );

        materials.insert(
            "debug_red".into(),
            material_assets.add(StandardMaterial {
                base_color: Color::RED,
                ..default()
            }),
        );
        materials.insert(
            "debug_blue".into(),
            material_assets.add(StandardMaterial {
                base_color: Color::BLUE,
                ..default()
            }),
        );
        materials.insert(
            "debug_green".into(),
            material_assets.add(StandardMaterial {
                base_color: Color::GREEN,
                ..default()
            }),
        );

        materials.insert(
            "tile_green".into(),
            material_assets.add(StandardMaterial {
                base_color: Color::GREEN.with_a(0.5),
                ..default()
            }),
        );

        materials.insert(
            "tile_red".into(),
            material_assets.add(StandardMaterial {
                base_color: Color::RED.with_a(0.5),
                ..default()
            }),
        );

        materials.insert(
            "tile_blue".into(),
            material_assets.add(StandardMaterial {
                base_color: Color::BLUE.with_a(0.5),
                ..default()
            }),
        );

        materials.insert(
            "tile_purple".into(),
            material_assets.add(StandardMaterial {
                base_color: Color::rgba(0.8, 0.1, 0.9, 0.5),
                ..default()
            }),
        );

        materials.insert(
            "tile_orange".into(),
            material_assets.add(StandardMaterial {
                base_color: Color::rgba(0.8, 0.5, 0.1, 0.5),
                ..default()
            }),
        );

        materials.insert(
            "aura_red".into(),
            material_assets.add(StandardMaterial {
                base_color_texture: Some(textures[&"aura_01"].clone()),
                //base_color: Color::rgba(1.0, 1.0, 1.0, 0.5),
                alpha_mode: AlphaMode::Add,
                emissive: Color::WHITE,
                emissive_texture: Some(textures[&"aura_02_e"].clone()),
                //emissive: Color::rgb_linear(4.0, 0.0, 0.0).into(),
                ..default()
            }),
        );

        materials.insert(
            "lava".into(),
            material_assets.add(StandardMaterial {
                base_color_texture: Some(textures[&"aura_02_c"].clone()),
                perceptual_roughness: 0.8,
                emissive: Color::WHITE,
                emissive_texture: Some(textures[&"aura_02_e"].clone()),
                normal_map_texture: Some(textures[&"aura_02_n"].clone()),
                ..default()
            }),
        );

        materials.insert(
            "tile".into(),
            material_assets.add(StandardMaterial {
                base_color_texture: Some(textures[&"tile_color"].clone()),
                perceptual_roughness: 0.8,
                normal_map_texture: Some(textures[&"tile_normal"].clone()),
                ..default()
            }),
        );

        materials.insert(
            "tile1".into(),
            material_assets.add(StandardMaterial {
                //emissive: Color::rgb_linear(0.1, 0.0, 0.0),
                //base_color: Color::rgb_linear(2.8, 0.0, 0.0),
                base_color_texture: Some(textures[&"tile1_color"].clone()),
                perceptual_roughness: 0.8,
                normal_map_texture: Some(textures[&"tile1_normal"].clone()),
                ..default()
            }),
        );

        let asset_server = world.resource_mut::<AssetServer>();
        let mut actors = HashMap::new();

        actors.insert(
            "wizard".into(),
            ActorInfo::new(
                vec![
                    asset_server.load("wizard.glb#Animation0"),
                    asset_server.load("wizard.glb#Animation1"),
                    asset_server.load("wizard.glb#Animation2"),
                    asset_server.load("wizard.glb#Animation3"),
                ],
                ActorHandle::Scene(asset_server.load("wizard.glb#Scene0")),
            ),
        );
        actors.insert(
            "swordsman".into(),
            ActorInfo::new(
                vec![
                    asset_server.load("swordsman.glb#Animation0"),
                    asset_server.load("swordsman.glb#Animation1"),
                    asset_server.load("swordsman.glb#Animation2"),
                    asset_server.load("swordsman.glb#Animation3"),
                ],
                ActorHandle::Scene(asset_server.load("swordsman.glb#Scene0")),
            ),
        );
        actors.insert(
            "archer".into(),
            ActorInfo::new(
                vec![
                    asset_server.load("archer.glb#Animation0"),
                    asset_server.load("archer.glb#Animation1"),
                    asset_server.load("archer.glb#Animation2"),
                    asset_server.load("archer.glb#Animation3"),
                ],
                ActorHandle::Scene(asset_server.load("archer.glb#Scene0")),
            ),
        );

        let mut props = HashMap::new();
        // Combat props
        props.insert(
            "bolt_01".into(),
            PropInfo::new(PropHandle::Scene(asset_server.load("bolt_01.glb#Scene0"))),
        );

        // Environment props
        props.insert(
            "rock_1".into(),
            PropInfo::new(PropHandle::Scene(asset_server.load("rock_1.glb#Scene0"))),
        );
        props.insert(
            "ground_lamp_1".into(),
            PropInfo::new(PropHandle::Scene(
                asset_server.load("ground_lamp_1.glb#Scene0"),
            )),
        );
        props.insert(
            "wall_hedge_1".into(),
            PropInfo::new(PropHandle::Scene(
                asset_server.load("wall_hedge_1.glb#Scene0"),
            )),
        );

        // Item props
        props.insert(
            "fragment_xp".into(),
            PropInfo::new(PropHandle::Scene(
                asset_server.load("fragment_xp.glb#Scene0"),
            )),
        );
        props.insert(
            "potion_hp".into(),
            PropInfo::new(PropHandle::Scene(
                asset_server.load("potion_m_hp.glb#Scene0"),
            )),
        );
        props.insert(
            "potion_ep".into(),
            PropInfo::new(PropHandle::Scene(asset_server.load("potion_ep.glb#Scene0"))),
        );
        props.insert(
            "potion_mp".into(),
            PropInfo::new(PropHandle::Scene(asset_server.load("potion_mp.glb#Scene0"))),
        );

        Self {
            color_materials,
            meshes,
            materials,
            aabbs,
            actors,
            props,
        }
    }
}
