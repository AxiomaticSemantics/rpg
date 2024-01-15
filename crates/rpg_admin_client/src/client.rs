use rpg_network_protocol::{self, protocol::*, KEY, PROTOCOL_ID};

use bevy::{
    math::primitives::Plane3d, prelude::*, render::primitives::Aabb, window::PrimaryWindow,
};

use lightyear::prelude::client::*;
use lightyear::prelude::*;

use std::net::{Ipv4Addr, SocketAddr};

#[derive(Resource)]
pub struct CameraTarget(pub Entity);

#[derive(Resource, Deref, DerefMut)]
pub struct ConnectionTimer(pub Timer);

impl FromWorld for ConnectionTimer {
    fn from_world(_world: &mut World) -> Self {
        Self(Timer::from_seconds(5.0, TimerMode::Repeating))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NetworkClientConfig {
    pub(crate) server_addr: Ipv4Addr,
    pub(crate) server_port: u16,
}

pub struct NetworkClientPlugin {
    pub(crate) config: NetworkClientConfig,
    pub(crate) client_id: ClientId,
}

impl Plugin for NetworkClientPlugin {
    fn build(&self, app: &mut App) {
        let server_addr = SocketAddr::new(self.config.server_addr.into(), self.config.server_port);
        let auth = Authentication::Manual {
            server_addr,
            client_id: self.client_id,
            private_key: KEY,
            protocol_id: PROTOCOL_ID,
        };
        let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0);
        /* FIXME add as testing feature
        let link_conditioner = LinkConditionerConfig {
            incoming_latency: Duration::from_millis(200),
            incoming_jitter: Duration::from_millis(20),
            incoming_loss: 0.05,
        };
        */
        let transport = TransportConfig::UdpSocket(client_addr);
        let io = Io::from_config(IoConfig::from_transport(transport));
        // FIXME .with_conditioner(link_conditioner));

        let config = ClientConfig {
            shared: rpg_network_protocol::shared_config(),
            netcode: Default::default(),
            ping: PingConfig::default(),
            sync: SyncConfig::default(),
            prediction: PredictionConfig::default(),
            // we are sending updates every frame (60fps), let's add a delay of 6 network-ticks
            interpolation: InterpolationConfig::default()
                .with_delay(InterpolationDelay::default().with_send_interval_ratio(2.0)),
        };
        let plugin_config = PluginConfig::new(config, io, protocol(), auth);

        app.add_plugins(ClientPlugin::new(plugin_config))
            .init_resource::<ConnectionTimer>()
            .init_resource::<GizmoConfig>()
            .init_resource::<AmbientLight>()
            .insert_resource(CameraTarget(Entity::PLACEHOLDER))
            .add_systems(Startup, init)
            .add_systems(
                FixedUpdate,
                (
                    input.in_set(FixedUpdateSet::Main),
                    (receive_player_rotation, receive_player_movement)
                        .after(input)
                        .in_set(FixedUpdateSet::Main),
                ),
            )
            .add_systems(PreUpdate, reconnect)
            .add_systems(
                Update,
                (
                    (
                        receive_hello,
                        receive_entity_spawn,
                        receive_entity_despawn,
                        handle_predicted_spawn,
                        handle_interpolated_spawn,
                    )
                        .before(update_camera),
                    update_camera,
                ),
            );
    }
}

pub(crate) fn reconnect(
    time: Res<Time>,
    mut client: ResMut<Client>,
    mut connection_timer: ResMut<ConnectionTimer>,
) {
    let dt = time.delta();
    connection_timer.tick(dt);

    if client.is_connected() {
        if !connection_timer.paused() {
            connection_timer.pause();
        }
    } else {
        if connection_timer.just_finished() {
            client.connect();
        }
    }
}

// Startup system for the client
pub(crate) fn init(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::Y * 10.).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.spawn(TextBundle::from_section(
        "Client",
        TextStyle {
            font_size: 30.0,
            //font:
            color: Color::WHITE,
            ..default()
        },
    ));

    let bundle = (
        Aabb::from_min_max(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(0.5, 0.5, 0.5)),
        AabbGizmo {
            color: Some(Color::BLACK),
        },
        SpatialBundle::default(),
    );
    commands.spawn(bundle);
}

pub(crate) fn update_camera(
    camera_target: Res<CameraTarget>,
    player_q: Query<&Transform, (With<PlayerPosition>, Without<Camera3d>)>,
    mut camera_q: Query<&mut Transform, With<Camera3d>>,
) {
    let mut transform = camera_q.single_mut();

    if let Ok(player_transform) = player_q.get_single() {
        transform.translation = player_transform.translation + Vec3::Y * 10.;
        transform.look_at(player_transform.translation, Vec3::Y)
    }
}

pub(crate) fn input(
    mut client: ResMut<Client>,
    mousepress: Res<ButtonInput<MouseButton>>,
    keypress: Res<ButtonInput<KeyCode>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    player_q: Query<&Transform, With<PlayerDirection>>,
) {
    if keypress.pressed(KeyCode::KeyW) || keypress.pressed(KeyCode::ArrowUp) {
        client
            .send_message::<Channel1, CSMovePlayer>(CSMovePlayer)
            .unwrap();
    }
    /*
    if keypress.pressed(KeyCode::Delete) {
        client.add_input(Inputs::Delete);
    }
    if keypress.pressed(KeyCode::Space) {
        client.add_input(Inputs::Spawn);
    }
    */

    let window = window_q.single();
    let (camera, transform) = camera_q.single();

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Some(ray) = camera.viewport_to_world(transform, cursor_position) else {
        println!("could not convert viewport position to world");
        return;
    };

    let Some(ground_distance) = ray.intersect_plane(Vec3::new(0., 0., 0.), Plane3d::new(Vec3::Y))
    else {
        println!("could not determine ground distance");
        return;
    };
    let ground_point = ray.get_point(ground_distance);

    if let Ok(player_transform) = player_q.get_single() {
        let target_dir = (ground_point - player_transform.translation).normalize_or_zero();
        if target_dir != player_transform.forward() {
            //println!("{target_dir} {}", player_transform.forward());
            client
                .send_message::<Channel1, CSRotPlayer>(CSRotPlayer(target_dir))
                .unwrap();
        }
    }
}

pub(crate) fn receive_player_rotation(
    mut player_q: Query<(&mut Transform, &mut PlayerDirection), With<Confirmed>>,
    mut rotation_events: EventReader<MessageEvent<SCRotPlayer>>,
) {
    for rotation in rotation_events.read() {
        for (mut transform, mut direction) in player_q.iter_mut() {
            direction.0 = rotation.message().0;
            transform.look_to(direction.0, Vec3::Y);
            //println!("player rotated to {}", position.0);
        }
    }
}

pub(crate) fn receive_player_movement(
    mut position_q: Query<(&mut Transform, &mut PlayerPosition), With<Confirmed>>,
    mut movement_events: EventReader<MessageEvent<SCMovePlayer>>,
) {
    for movement in movement_events.read() {
        for (mut transform, mut position) in position_q.iter_mut() {
            position.0 = movement.message().0;
            transform.translation = position.0;
            //println!("player moved to {}", position.0);
        }
    }
}

pub(crate) fn receive_hello(
    mut client: ResMut<Client>,
    mut reader: EventReader<MessageEvent<SCHello>>,
) {
    for event in reader.read() {
        info!("Received: {:?}", event.message());

        client
            .send_message::<Channel1, CSLoadAccount>(CSLoadAccount {
                name: "TestAccount".into(),
                password: "".into(),
            })
            .unwrap();

        client
            .send_message::<Channel1, CSCreateAccount>(CSCreateAccount {
                name: "TestAccount".into(),
                email: "test@test.com".into(),
                password: "".into(),
            })
            .unwrap();

        reader.clear();
        return;
    }
}

pub(crate) fn receive_entity_spawn(
    mut commands: Commands,
    mut reader: EventReader<EntitySpawnEvent>,
    mut camera_target: ResMut<CameraTarget>,
    player_q: Query<&PlayerPosition>,
) {
    for event in reader.read() {
        info!("Received entity spawn: {:?}", event.entity());

        let bundle = (
            Aabb::from_min_max(Vec3::new(-0.5, -0.5, -0.5), Vec3::new(0.5, 0.5, 0.5)),
            AabbGizmo {
                color: Some(Color::RED),
            },
            SpatialBundle::default(),
        );

        camera_target.0 = *event.entity();
        commands.entity(*event.entity()).insert(bundle);
    }
}

pub(crate) fn receive_entity_despawn(mut reader: EventReader<EntityDespawnEvent>) {
    for event in reader.read() {
        info!("Received entity despawn: {:?}", event.entity());
    }
}

// When the predicted copy of the client-owned entity is spawned
// - keep track of it in the Global resource
pub(crate) fn handle_predicted_spawn(mut predicted: Query<&mut PlayerDirection, Added<Predicted>>) {
    for mut color in predicted.iter_mut() {
        color.0 = Vec3::ZERO;
    }
}

// When the predicted copy of the client-owned entity is spawned
// - keep track of it in the Global resource
pub(crate) fn handle_interpolated_spawn(
    mut interpolated: Query<&mut PlayerDirection, Added<Interpolated>>,
) {
    for mut direction in interpolated.iter_mut() {
        direction.0 = Vec3::ZERO;
    }
}
