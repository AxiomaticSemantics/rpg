#![allow(clippy::too_many_arguments)]

use crate::{
    assets::TextureAssets,
    game::{
        actor::player::Player, assets::RenderResources, controls::Controls,
        metadata::MetadataResources, plugin::GameSessionCleanup,
    },
};
use rpg_core::{
    passive_tree::{EdgeNodes, Node, NodeId, NodeKind, PassiveSkillGraph},
    stat::value::ValueKind,
};
use rpg_util::unit::Unit;
use ui_util::style::UiTheme;
use util::cleanup::CleanupStrategy;

use bevy::{
    asset::{Assets, Handle},
    core_pipeline::{core_2d::Camera2dBundle, core_3d::Camera3d},
    ecs::{
        component::Component,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::{BuildChildren, Parent},
    input::{keyboard::KeyCode, mouse::MouseButton, ButtonInput},
    math::{Vec2, Vec3},
    prelude::{Deref, DerefMut},
    render::{
        camera::{Camera, ClearColorConfig, OrthographicProjection},
        color::Color,
        mesh::{
            shape::{Circle, Quad},
            Mesh,
        },
        view::RenderLayers,
    },
    sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2dHandle},
    text::{BreakLineOn, JustifyText, Text, TextSection},
    time::Time,
    transform::components::{GlobalTransform, Transform},
    ui::{
        node_bundles::{NodeBundle, TextBundle},
        AlignSelf, Display, FlexDirection, Style, UiRect, Val,
    },
    utils::default,
    window::{PrimaryWindow, Window},
};

use petgraph::algo;

#[derive(Component, Deref, DerefMut)]
pub struct PassiveTree(pub PassiveSkillGraph);

#[derive(Component)]
pub struct PassiveTreeCamera;

#[derive(Deref, DerefMut, Component)]
pub struct PassiveTreeConnection(pub EdgeNodes);

pub enum PassiveNodeState {
    Normal,
    Hovered,
    Allocated,
}

#[derive(Component, Deref, DerefMut)]
pub struct PassiveTreeNode(pub NodeId);

#[derive(Component)]
pub struct PassiveTreeLegend;

#[derive(Component)]
pub struct PassiveTreePopup;

#[derive(Component)]
pub struct PassiveTreePopupHeader;

#[derive(Component)]
pub struct PassiveTreePopupBody;

#[derive(Component)]
pub struct PassiveTreePopupFlavour;

pub(crate) fn setup(
    mut commands: Commands,
    metadata: Res<MetadataResources>,
    ui_theme: Res<UiTheme>,
    mut renderables: ResMut<RenderResources>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    textures: Res<TextureAssets>,
    player_q: Query<&Unit, With<Player>>,
) {
    let first_pass_layer = RenderLayers::layer(1);

    let node_info = &metadata.rpg.passive_tree.node_info;

    let line_mesh = meshes.add(Quad::new(Vec2::new(1., 0.125) * 100.));
    let circle_root = meshes.add(Circle::new(node_info.root_size * 100.));
    let circle_major = meshes.add(Circle::new(node_info.major_size * 100.));
    let circle_minor = meshes.add(Circle::new(node_info.minor_size * 100.));

    let circle_material_root = materials.add(ColorMaterial {
        color: Color::rgb_u8(255, 108, 171),
        texture: Some(textures.icons[&"p_circle"].clone()),
    });

    let circle_material_root_allocated = materials.add(ColorMaterial {
        color: Color::rgb_u8(255, 48, 111),
        texture: Some(textures.icons[&"p_circle"].clone()),
    });

    let circle_material_root_hovered = materials.add(ColorMaterial {
        color: Color::rgb_u8(255, 128, 191),
        texture: Some(textures.icons[&"p_circle"].clone()),
    });

    let circle_material_major = materials.add(ColorMaterial {
        color: Color::rgb_u8(108, 255, 214),
        texture: Some(textures.icons[&"p_circle"].clone()),
    });

    let circle_material_major_allocated = materials.add(ColorMaterial {
        color: Color::rgb_u8(48, 255, 154),
        texture: Some(textures.icons[&"p_circle"].clone()),
    });

    let circle_material_major_hovered = materials.add(ColorMaterial {
        color: Color::rgb_u8(128, 255, 234),
        texture: Some(textures.icons[&"p_circle"].clone()),
    });

    let circle_material_minor = materials.add(ColorMaterial {
        color: Color::rgb_u8(129, 108, 255),
        texture: Some(textures.icons[&"p_circle"].clone()),
    });

    let circle_material_minor_allocated = materials.add(ColorMaterial {
        color: Color::rgb_u8(69, 48, 255),
        texture: Some(textures.icons[&"p_circle"].clone()),
    });

    let circle_material_minor_hovered = materials.add(ColorMaterial {
        color: Color::rgb_u8(149, 128, 255),
        texture: Some(textures.icons[&"p_circle"].clone()),
    });

    renderables
        .color_materials
        .insert("root_node".into(), circle_material_root.clone());

    renderables.color_materials.insert(
        "root_node_hovered".into(),
        circle_material_root_hovered.clone(),
    );

    renderables.color_materials.insert(
        "root_node_allocated".into(),
        circle_material_root_allocated.clone(),
    );

    renderables
        .color_materials
        .insert("major_node".into(), circle_material_major.clone());

    renderables.color_materials.insert(
        "major_node_hovered".into(),
        circle_material_major_hovered.clone(),
    );

    renderables.color_materials.insert(
        "major_node_allocated".into(),
        circle_material_major_allocated.clone(),
    );

    renderables
        .color_materials
        .insert("minor_node".into(), circle_material_minor.clone());

    renderables.color_materials.insert(
        "minor_node_hovered".into(),
        circle_material_minor_hovered.clone(),
    );

    renderables.color_materials.insert(
        "minor_node_allocated".into(),
        circle_material_minor_allocated.clone(),
    );

    let line_material = materials.add(ColorMaterial {
        color: Color::rgb(0.3, 0.3, 0.3),
        ..default()
    });

    let line_allocated_material = materials.add(ColorMaterial {
        color: Color::rgb(0.85, 0.85, 0.85),
        ..default()
    });

    renderables
        .color_materials
        .insert("line".into(), line_material.clone());

    renderables
        .color_materials
        .insert("line_allocated".into(), line_allocated_material.clone());

    let player = player_q.single();
    let class_root = PassiveSkillGraph::get_class_root(player.class);

    /*
    let node_scale = 3.0;
    let cluster_scale = 9.0;
    let section_scale = 27.0;
    let zone_scale = 81.0;

    for z in 0..7 {
        let zone_name = match z {
            0 => "StrDexInt",
            1 => "Str",
            2 => "StrDex",
            3 => "Dex",
            4 => "DexInt",
            5 => "Int",
            6 => "IntStr",
            _ => unreachable!(),
        };
        let zone_hex_pos = HexPosition::from_index(z);
        let zone_pos = zone_hex_pos.unit_position();
        let zone_origin = Vec3::new(zone_pos.x, zone_pos.y, 0.0) * zone_scale;

        for s in 0..7 {
            let section_hex_pos = HexPosition::from_index(s);
            let section_pos = section_hex_pos.unit_position();
            let section_origin =
                zone_origin + Vec3::new(section_pos.x, section_pos.y, 0.0) * section_scale;

            let is_root_section = z == s;
            for c in 0..7 {
                let cluster_hex_pos = HexPosition::from_index(c);
                let cluster_pos = cluster_hex_pos.unit_position();
                let cluster_origin =
                    section_origin + Vec3::new(cluster_pos.x, cluster_pos.y, 0.0) * cluster_scale;

                let is_root_cluster = is_root_section && c == 6;
                for n in 0..7 {
                    let node_hex_pos = HexPosition::from_index(n);
                    let node_pos = node_hex_pos.unit_position();
                    let node_origin =
                        cluster_origin + Vec3::new(node_pos.x, node_pos.y, 0.0) * node_scale;

                    let (kind, circle_material, circle_mesh) = match c {
                        _ if n == 6 && is_root_cluster => (
                            NodeKind::Root,
                            circle_material_root.clone(),
                            circle_root.clone(),
                        ),
                        _ if n == 6 && !is_root_cluster => (
                            NodeKind::Major,
                            circle_material_major.clone(),
                            circle_major.clone(),
                        ),
                        _ => (
                            NodeKind::Minor,
                            circle_material_minor.clone(),
                            circle_minor.clone(),
                        ),
                    };

                    let mut connections = vec![];

                    if n < 6 {
                        connections.push(NodeId::from_coordinates(
                            z as u16,
                            s as u16,
                            c as u16,
                            ((n + 1) % 6) as u16,
                        ));
                    } else if n == 6 {
                        if c < 6 {
                            connections.push(NodeId::from_coordinates(
                                z as u16,
                                s as u16,
                                c as u16,
                                cluster_hex_pos.opposite().to_index() as u16,
                            ));
                        } else {
                            connections
                                .push(NodeId::from_coordinates(z as u16, s as u16, c as u16, 0));
                        }
                    }

                    // inner cluster connections
                    if c == 6 && n != 6 && n % 2 == 0 {
                        connections.push(NodeId::from_coordinates(
                            z as u16,
                            s as u16,
                            ((c + n) % 6) as u16,
                            node_hex_pos.opposite().to_index() as u16,
                        ));
                    } else if c != 6 && n != 6 && c == (n + 4) % 6 && c % 2 == 0 {
                        connections.push(NodeId::from_coordinates(
                            z as u16,
                            s as u16,
                            ((c + 1) % 6) as u16,
                            node_hex_pos.opposite().to_index() as u16,
                        ));
                    } else if s != 6
                        && c != 6
                        && n != 6
                        && c % 2 == 0
                        && c == (n + 3) % 6
                        && s == (c + 4) % 6
                    {
                        connections.push(NodeId::from_coordinates(
                            z as u16,
                            ((s + 1) % 6) as u16,
                            //cluster_hex_pos.opposite().to_index() as u16,
                            ((c + 3) % 6) as u16,
                            node_hex_pos.opposite().to_index() as u16,
                        ));
                    }

                    /*
                    if !section_hex_pos.is_ring_pos()
                        && cluster_hex_pos.is_ring_pos()
                        && node_hex_pos.is_ring_pos()
                        && n % 2 == 0
                        && c == node_hex_pos.opposite().to_index()
                    //node_hex_pos.opposite().to_index()
                    //&& n == cluster_hex_pos.opposite().to_index()
                    //&& c == (s + 1) % 6
                    //&& s == (n + 3) % 6
                    //&& s = (
                    //cluster_hex_pos.opposite().to_index()
                    {
                        println!("k");
                        connections.push(NodeId::from_coordinates(
                            z as u16,
                            cluster_hex_pos.opposite().to_index() as u16,
                            ((c + 3) % 6) as u16,
                            //cluster_hex_pos.opposite().to_index() as u16,
                            //((c + 3) % 6) as u16,
                            //((n + cluster_hex_pos.opposite().to_index()) % 6) as u16,
                            ((node_hex_pos.opposite().to_index()) % 6) as u16,
                        ));
                    }*/

                    let node_id = NodeId(id);
                    let node = Node {
                        name: format!("{}{:?}{}", zone_name, kind, node_id).into(),
                        id: node_id,
                        kind,
                        position: node_origin.xy(),
                        connections,
                    };

                    println!(
                        "{{ \"name\": \"{}\", \"id\": {}, \"position\": {},\n\"kind\": \"{kind:?}\", \"connections\": {:?} }},\n",
                        node.name, node_id.0, node_origin.xy(), node.connections
                    );

                    let graph_index = graph.add_node(node_id);
                    graph_indices.insert(node_id, graph_index);
                    passive_indices.insert(node_id, graph_indices.len() - 1);

                    passive_tree.nodes.push(node);

                    let node_transform = Transform::from_translation(node_origin);
                    commands.spawn((
                        first_pass_layer,
                        MaterialMeshBundle {
                            transform: node_transform,
                            mesh: circle_mesh.clone(),
                            material: circle_material.clone(),
                            ..default()
                        },
                    ));

                    id += 1;
                }
            }
        }
    }
    */

    for node in &metadata.rpg.passive_tree.nodes {
        let (circle_material, circle_mesh) = match node.kind {
            NodeKind::Root => {
                if node.id == class_root {
                    (circle_material_root_allocated.clone(), circle_root.clone())
                } else {
                    (circle_material_root.clone(), circle_root.clone())
                }
            }
            NodeKind::Major => (circle_material_major.clone(), circle_major.clone()),
            NodeKind::Minor => (circle_material_minor.clone(), circle_minor.clone()),
        };

        let node_transform = Transform::from_translation((node.position * 100.).extend(1.0));
        commands.spawn((
            first_pass_layer,
            GameSessionCleanup,
            CleanupStrategy::Despawn,
            PassiveTreeNode(node.id),
            MaterialMesh2dBundle {
                transform: node_transform,
                mesh: Mesh2dHandle(circle_mesh.clone()),
                material: circle_material.clone(),
                ..default()
            },
        ));

        let node_size = node.get_size(node_info) * 100.;
        let node_origin = (node.position * 100.).extend(1.01);

        for connection in node.connections.iter() {
            let connection_idx = metadata.rpg.passive_tree.passive_indices[&connection];
            let connection_descriptor = &metadata.rpg.passive_tree.nodes[connection_idx];

            let connection_size = connection_descriptor.get_size(node_info) * 100.;
            let connection_origin = (connection_descriptor.position * 100.).extend(1.01);

            let line_transform =
                get_line_transform(&node_origin, node_size, &connection_origin, connection_size);

            commands.spawn((
                GameSessionCleanup,
                CleanupStrategy::Despawn,
                first_pass_layer,
                PassiveTreeConnection(EdgeNodes {
                    lhs: node.id,
                    rhs: *connection,
                }),
                MaterialMesh2dBundle {
                    transform: line_transform,
                    mesh: Mesh2dHandle(line_mesh.clone()),
                    material: line_material.clone(),
                    ..default()
                },
            ));
        }
    }

    let root_pos = metadata
        .rpg
        .passive_tree
        .nodes
        .iter()
        .find(|n| n.id == class_root)
        .unwrap()
        .position;

    commands.spawn((
        GameSessionCleanup,
        CleanupStrategy::Despawn,
        PassiveTreeCamera,
        first_pass_layer,
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::rgb(0.05, 0.04, 0.04)),
                order: -1,
                is_active: false,
                ..default()
            },
            transform: Transform::from_translation((root_pos * 100.).extend(1.)),
            ..default()
        },
    ));

    let frame_style = Style {
        flex_direction: FlexDirection::Column,
        align_self: AlignSelf::FlexStart,
        display: Display::None,
        padding: UiRect::all(ui_theme.padding),
        border: UiRect::all(ui_theme.border),
        ..default()
    };
    let mut legend_style = frame_style.clone();
    legend_style.margin = UiRect::all(ui_theme.margin);
    legend_style.display = Display::Flex;
    legend_style.left = Val::Px(200.);

    commands
        .spawn((
            GameSessionCleanup,
            CleanupStrategy::DespawnRecursive,
            NodeBundle {
                style: legend_style,
                border_color: ui_theme.border_color,
                background_color: ui_theme.background_color,
                ..default()
            },
            first_pass_layer,
        ))
        .with_children(|p| {
            p.spawn((
                PassiveTreeLegend,
                first_pass_layer,
                TextBundle {
                    text: Text {
                        sections: vec![TextSection::new("", ui_theme.text_style_regular.clone())],
                        linebreak_behavior: BreakLineOn::WordBoundary,
                        ..default()
                    },
                    ..default()
                },
            ));
        });

    commands
        .spawn((
            GameSessionCleanup,
            CleanupStrategy::DespawnRecursive,
            NodeBundle {
                style: frame_style,
                background_color: ui_theme.frame_background_color,
                border_color: ui_theme.border_color,
                ..default()
            },
            first_pass_layer,
        ))
        .with_children(|p| {
            p.spawn((
                PassiveTreePopupHeader,
                first_pass_layer,
                TextBundle {
                    text: Text {
                        sections: vec![TextSection::new("", ui_theme.text_style_regular.clone())],
                        justify: JustifyText::Center,
                        linebreak_behavior: BreakLineOn::WordBoundary,
                    },
                    ..default()
                },
            ));

            p.spawn((
                PassiveTreePopupBody,
                first_pass_layer,
                TextBundle {
                    text: Text {
                        sections: vec![TextSection::new("", ui_theme.text_style_small.clone())],
                        linebreak_behavior: BreakLineOn::WordBoundary,
                        ..default()
                    },
                    ..default()
                },
            ));

            p.spawn((
                PassiveTreePopupFlavour,
                first_pass_layer,
                TextBundle {
                    text: Text {
                        sections: vec![TextSection::new("", ui_theme.text_style_small.clone())],
                        linebreak_behavior: BreakLineOn::WordBoundary,
                        ..default()
                    },
                    ..default()
                },
            ));
        });
}

pub fn toggle_passive_tree(
    mut controls: ResMut<Controls>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_passive_q: Query<&mut Camera, With<PassiveTreeCamera>>,
    mut camera_main_q: Query<&mut Camera, (With<Camera3d>, Without<PassiveTreeCamera>)>,
) {
    let mut camera_passive = camera_passive_q.single_mut();
    let mut camera_main = camera_main_q.single_mut();

    if keyboard_input.just_pressed(KeyCode::KeyS) {
        if camera_passive.is_active {
            controls.set_inhibited(false);
            camera_passive.is_active = false;
            camera_main.is_active = true;
        } else {
            camera_passive.is_active = true;
            controls.set_inhibited(true);
            camera_main.is_active = false;
        }
    }
}

pub fn set_view(
    time: Res<Time>,
    controls: Res<Controls>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut camera_passive_q: Query<
        (&mut Transform, &mut OrthographicProjection, &mut Camera),
        With<PassiveTreeCamera>,
    >,
) {
    let (mut transform, mut projection, camera_passive) = camera_passive_q.single_mut();

    if !camera_passive.is_active {
        return;
    }

    let dt = time.delta_seconds();
    let wheel_delta = controls.mouse_wheel_delta;
    if wheel_delta != 0.0 {
        let sign = if wheel_delta > 0. { -1. } else { 1. };
        let delta = 25.0 * dt * sign;

        let z = (projection.scale + delta).clamp(1.0, 20.);
        projection.scale = z;
    }

    if mouse_input.pressed(MouseButton::Right) && controls.mouse_motion != Vec2::ZERO {
        let dz = dt * projection.scale;
        let motion_delta = Vec3::new(
            -(controls.mouse_motion.x * 25. * dz),
            controls.mouse_motion.y * 25. * dz,
            dz,
        );

        // TODO I should probably use the viewport aspect ratio for clamping
        let t_x = (transform.translation.x + motion_delta.x).clamp(-24000.0, 24000.0);
        let t_y = (transform.translation.y + motion_delta.y).clamp(-24000.0, 24000.0);

        transform.translation.x = t_x;
        transform.translation.y = t_y;
    }
}

pub fn update_legend(
    camera_passive_q: Query<&mut Camera, With<PassiveTreeCamera>>,
    player_q: Query<&mut Unit, With<Player>>,
    mut passive_tree_legend_q: Query<&mut Text, With<PassiveTreeLegend>>,
) {
    let camera = camera_passive_q.single();
    if !camera.is_active {
        return;
    }

    let player = player_q.single();
    let mut legend = passive_tree_legend_q.single_mut();
    let passive_point_string = format!("{} Passive Skill Points", player.0.passive_skill_points);
    if legend.sections[0].value != passive_point_string {
        legend.sections[0].value = passive_point_string;
    }
}

pub fn display(
    renderables: Res<RenderResources>,
    metadata: Res<MetadataResources>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut player_q: Query<(&mut Unit, &mut PassiveTree), With<Player>>,
    mut camera_passive_q: Query<(&GlobalTransform, &mut Camera), With<PassiveTreeCamera>>,
    mut passive_node_q: Query<(&PassiveTreeNode, &mut Handle<ColorMaterial>)>,
    mut passive_connection_q: Query<
        (&PassiveTreeConnection, &mut Handle<ColorMaterial>),
        Without<PassiveTreeNode>,
    >,
    mut passive_tree_popup_header_q: Query<(&mut Text, &Parent), With<PassiveTreePopupHeader>>,
    mut passive_tree_popup_body_q: Query<
        &mut Text,
        (With<PassiveTreePopupBody>, Without<PassiveTreePopupHeader>),
    >,
    mut passive_tree_frame_q: Query<&mut Style, Without<PassiveTreePopup>>,
) {
    let (global_transform, camera_passive) = camera_passive_q.single_mut();

    if !camera_passive.is_active {
        return;
    }

    let window = window_q.single();

    let Some(position) = window.cursor_position() else {
        return;
    };
    let Some(point) = camera_passive.viewport_to_world_2d(global_transform, position) else {
        println!("could not convert viewport position to world");
        return;
    };

    let (mut player, mut passive_tree) = player_q.single_mut();

    // Set edge material to allocated material if not already set
    for (passive_connection, mut connection_material) in &mut passive_connection_q {
        if passive_tree
            .allocated_edges
            .iter()
            .any(|n| *n == **passive_connection)
            && *connection_material != renderables.color_materials["line_allocated"]
        {
            *connection_material = renderables.color_materials["line_allocated"].clone();
        }
    }

    // Update and display node popup if the cursor is in range of a node
    let mut popup_visibile = false;

    let (mut popup_header, popup_parent) = passive_tree_popup_header_q.single_mut();
    let mut popup_body = passive_tree_popup_body_q.single_mut();
    let mut popup_frame = passive_tree_frame_q.get_mut(popup_parent.get()).unwrap();

    for (passive_node, mut material_handle) in &mut passive_node_q {
        let node_descriptor = metadata
            .rpg
            .passive_tree
            .nodes
            .iter()
            .find(|n| n.id == passive_node.0)
            .unwrap();

        let allocated_node = passive_tree
            .allocated_nodes
            .iter()
            .find(|n| **n == node_descriptor.id);

        let mut node_state = match allocated_node {
            Some(_) => PassiveNodeState::Allocated,
            None => PassiveNodeState::Normal,
        };

        let node_size = node_descriptor.get_size(&metadata.rpg.passive_tree.node_info) * 100.;
        let distance = point.distance(node_descriptor.position * 100.);

        let node_hovered = distance <= node_size;
        if node_hovered {
            node_state = PassiveNodeState::Hovered;
        }

        update_node_material(
            node_descriptor,
            &renderables,
            &mut material_handle,
            node_state,
        );

        // At this point we only operate on the selected node
        if passive_node.0 != node_descriptor.id {
            continue;
        }

        if node_hovered {
            popup_visibile = true;

            if popup_frame.display != Display::Flex {
                popup_frame.display = Display::Flex;
            }

            popup_frame.top = Val::Px(position.y);
            popup_frame.left = Val::Px(position.x + 24.);

            // update the popup's text
            let header = node_descriptor.name.to_string();
            if popup_header.sections[0].value != header {
                popup_header.sections[0].value = header;
            }

            if let Some(stats) = &node_descriptor.modifiers {
                let mut body_value = String::new();
                for stat in stats.iter() {
                    let stat_descriptor = metadata
                        .rpg
                        .stat
                        .stats
                        .values()
                        .find(|s| s.id == stat.id)
                        .unwrap();

                    let is_percent = stat_descriptor.value_kind == ValueKind::F32;
                    body_value = format!(
                        "{body_value}\n{:2.1}{} {}",
                        stat.value,
                        if is_percent { "%" } else { "" },
                        stat_descriptor.name
                    );
                }
                popup_body.sections[0].value = body_value;
            } else {
                popup_body.sections[0].value.clear();
            }

            if player.passive_skill_points > 0 && mouse_input.just_pressed(MouseButton::Left) {
                let mut allocated_node_id: Option<NodeId> = None;

                if passive_tree.allocated_nodes.contains(&node_descriptor.id) {
                    println!("node already allocated");
                    break;
                };

                for node in &passive_tree.allocated_nodes {
                    let start = metadata.rpg.passive_tree.graph_indices[&node];
                    let dest = metadata.rpg.passive_tree.graph_indices[&node_descriptor.id];

                    let path = algo::astar(
                        &metadata.rpg.passive_tree.graph,
                        start,
                        |d| d == dest,
                        |e| *e.weight(),
                        |_| 0,
                    );

                    let Some((len, _)) = path else {
                        //println!("no path {len}");
                        continue;
                    };

                    println!("no path {len}");
                    if len == 1 {
                        allocated_node_id = Some(*node);
                        break;
                    }
                }

                if let Some(allocated_node_id) = allocated_node_id {
                    passive_tree.allocated_nodes.push(node_descriptor.id);
                    passive_tree.allocated_edges.push(EdgeNodes {
                        lhs: allocated_node_id,
                        rhs: node_descriptor.id,
                    });

                    player.passive_skill_points -= 1;
                    player.apply_passive_rewards(&metadata.rpg, &passive_tree);
                }
            }
            break;
        }
    }

    if !popup_visibile && popup_frame.display != Display::None {
        popup_frame.display = Display::None;
    }
}

fn update_node_material(
    node: &Node,
    renderables: &RenderResources,
    material: &mut Handle<ColorMaterial>,
    node_state: PassiveNodeState,
) {
    let key = match node_state {
        PassiveNodeState::Normal => match node.kind {
            NodeKind::Root => "root_node",
            NodeKind::Major => "major_node",
            NodeKind::Minor => "minor_node",
        },
        PassiveNodeState::Allocated => match node.kind {
            NodeKind::Root => "root_node_allocated",
            NodeKind::Major => "major_node_allocated",
            NodeKind::Minor => "minor_node_allocated",
        },
        PassiveNodeState::Hovered => match node.kind {
            NodeKind::Root => "root_node_hovered",
            NodeKind::Major => "major_node_hovered",
            NodeKind::Minor => "minor_node_hovered",
        },
    };

    let wanted_material = &renderables.color_materials[key];
    if material != wanted_material {
        *material = wanted_material.clone();
    }
}

fn get_line_transform(
    origin: &Vec3,
    origin_size: f32,
    destination: &Vec3,
    destination_size: f32,
) -> Transform {
    let dir = (*destination - *origin).normalize_or_zero();

    let line_origin =
        ((*origin + origin_size * dir) + (*destination + destination_size * -dir)) * 0.5;
    let mut line_transform = Transform::from_translation(line_origin);

    let angle = dir.y.atan2(dir.x);
    if angle.is_finite() {
        line_transform.rotate_local_z(angle);
    }

    let distance = origin.distance(*destination);
    let scale = (distance - origin_size - destination_size) / 100.;
    //println!("{origin} {origin_size} {destination} {destination_size} {distance} {scale}");

    line_transform.scale = Vec3::new(scale * 0.9, 1.0, 1.0);

    line_transform
}
