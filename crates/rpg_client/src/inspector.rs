use bevy::{
    app::{App, Startup, Update},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        entity::Entity,
        schedule::IntoSystemConfigs,
        system::{Commands, Res, ResMut, Resource},
    },
    input::{
        common_conditions::{input_just_pressed, input_toggle_active},
        keyboard::KeyCode,
    },
    log::info,
    render::{
        camera::{Camera, RenderTarget},
        view::RenderLayers,
    },
    utils::default,
    window::{Window, WindowRef},
};

use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(Component)]
pub(crate) struct InspectorWindow;

#[derive(Default, Resource)]
pub(crate) struct InspectorTarget {
    pub(crate) window: Option<Entity>,
    pub(crate) camera: Option<Entity>,
}

pub(crate) fn inspector_plugin(app: &mut App) {
    app.init_resource::<InspectorTarget>()
        .add_plugins(
            WorldInspectorPlugin::<InspectorWindow>::new()
                .run_if(input_toggle_active(true, KeyCode::F1)),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            remove_inspector.run_if(input_just_pressed(KeyCode::F8)),
        );
}

fn setup(mut commands: Commands, mut inspector_target: ResMut<InspectorTarget>) {
    // Spawn the inspector window
    let window = commands
        .spawn((
            InspectorWindow,
            Window {
                title: "Inspector".to_owned(),
                ..default()
            },
        ))
        .id();

    let camera = commands
        .spawn((
            RenderLayers::layer(4),
            Camera2dBundle {
                camera: Camera {
                    target: RenderTarget::Window(WindowRef::Entity(window)),
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    inspector_target.window = Some(window);
    inspector_target.camera = Some(camera);
}

fn remove_inspector(mut commands: Commands, inspector_target: Res<InspectorTarget>) {
    if inspector_target.window.is_none() && inspector_target.camera.is_none() {
        info!("no inspector target");
        return;
    }

    if let Some(camera) = inspector_target.camera {
        info!("removing inspector camera");
        commands.entity(camera).despawn();
    }

    if let Some(window) = inspector_target.window {
        info!("removing inspector window");
        commands.entity(window).despawn();
    }

    if inspector_target.window.is_some() && inspector_target.camera.is_some() {
        info!("removing inspector");
        commands.remove_resource::<InspectorTarget>();
    }
}
