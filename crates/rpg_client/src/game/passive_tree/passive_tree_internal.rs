#![allow(clippy::too_many_arguments)]

use crate::{
    assets::TextureAssets,
    game::{
        actor::player::Player, assets::RenderResources, controls::Controls,
        metadata::MetadataResources, plugin::GameSessionCleanup,
    },
};
use rpg_core::{
    passive_tree::{EdgeNodes, Node, NodeId, NodeKind, UnitPassiveSkills as RpgUnitPassiveSkills},
    value::ValueKind,
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
    log::debug,
    math::{
        primitives::{Circle, Rectangle},
        Vec2, Vec3,
    },
    prelude::{Deref, DerefMut},
    render::{
        camera::{Camera, ClearColorConfig, OrthographicProjection},
        color::Color,
        mesh::Mesh,
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

#[derive(Component, Deref, DerefMut)]
pub struct UnitPassiveSkills(pub RpgUnitPassiveSkills);

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
    mut renderables: ResMut<RenderResources>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    textures: Res<TextureAssets>,
) {
    let first_pass_layer = RenderLayers::layer(2);

    let node_info = &metadata.rpg.passive_tree.node_info;

    let line_mesh = meshes.add(Rectangle::new(100., 12.5));
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

    /*
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
    */

    // FIXME there needs to be a system that applies the characters passive tree to the default
    // tree
    for node in &metadata.rpg.passive_tree.nodes {
        let (circle_material, circle_mesh) = match node.kind {
            NodeKind::Root => {
                //if node.id == class_root {
                //    (circle_material_root_allocated.clone(), circle_root.clone())
                //} else {
                (circle_material_root.clone(), circle_root.clone())
                //} FIXME
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
    /*
        let player = player_q.single();
        let class_root = PassiveSkillGraph::get_class_root(player.class);

        let root_pos = metadata
            .rpg
            .passive_tree
            .nodes
            .iter()
            .find(|n| n.id == class_root)
            .unwrap()
            .position;
    */
}

pub(crate) fn setup_ui(mut commands: Commands, ui_theme: Res<UiTheme>) {
    let first_pass_layer = RenderLayers::layer(2);

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
            transform: Transform::from_translation(Vec2::ZERO.extend(1.)),
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
    mut player_q: Query<(&mut Unit, &mut UnitPassiveSkills), With<Player>>,
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

                    let Some((len, path)) = metadata.rpg.passive_tree.get_node_path(start, dest)
                    else {
                        continue;
                    };

                    debug!("path length: {len}");
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
