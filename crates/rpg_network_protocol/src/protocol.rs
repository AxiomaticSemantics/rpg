use bevy::{
    ecs::{bundle::Bundle, component::Component, entity::Entity},
    math::Vec3,
    prelude::{Deref, DerefMut},
    utils::{default, EntityHashSet},
};

use derive_more::{Add, Mul};
use lightyear::prelude::*;
use serde_derive::{Deserialize, Serialize};

use rpg_core::uid::Uid;
use rpg_world::zone::ZoneId;

// Player
#[derive(Bundle)]
pub struct NetworkPlayerBundle {
    pub id: NetworkClientId,
    pub direction: PlayerDirection,
    pub position: PlayerPosition,
    pub replicate: Replicate,
}

impl NetworkPlayerBundle {
    pub fn new(id: ClientId, position: Vec3, direction: Vec3) -> Self {
        Self {
            id: NetworkClientId(id),
            position: PlayerPosition(position),
            direction: PlayerDirection(direction),
            replicate: Replicate {
                // prediction_target: NetworkTarget::None,
                prediction_target: NetworkTarget::Only(vec![id]),
                interpolation_target: NetworkTarget::AllExcept(vec![id]),
                ..default()
            },
        }
    }
}

// Components

#[derive(Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NetworkClientId(pub ClientId);

#[derive(Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MonsterId(pub Uid);

#[derive(
    Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq, Deref, DerefMut, Add, Mul,
)]
pub struct PlayerPosition(pub Vec3);

#[derive(
    Component, Message, Serialize, Deserialize, Clone, Debug, PartialEq, Deref, DerefMut, Add, Mul,
)]
pub struct PlayerDirection(pub Vec3);

// This component, when replicated, needs to have the inner entity mapped from the Server world
// to the client World.
// This can be done by adding a `#[message(custom_map)]` attribute to the component, and then
// deriving the `MapEntities` trait for the component.
#[derive(Component, Message, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[message(custom_map)]
pub struct PlayerParent(pub Entity);

impl<'a> MapEntities<'a> for PlayerParent {
    fn map_entities(&mut self, entity_mapper: Box<dyn EntityMapper + 'a>) {
        self.0.map_entities(entity_mapper);
    }

    fn entities(&self) -> EntityHashSet<Entity> {
        EntityHashSet::from_iter(vec![self.0])
    }
}

#[component_protocol(protocol = "MyProtocol")]
pub enum Components {
    #[sync(once)]
    Id(NetworkClientId),
    #[sync(full)]
    PlayerPosition(PlayerPosition),
    #[sync(full)]
    PlayerDirection(PlayerDirection),
    //#[sync(once)]
    //PlayerColor(PlayerColor),
}

// Channels

#[derive(Channel)]
pub struct Channel1;

// Messages

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSHello;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCHello;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSConnectPlayer;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSConnectAdmin;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSJoinZone(pub ZoneId);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSMovePlayer;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSRotPlayer(pub Vec3);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCMovePlayer(pub Vec3);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCRotPlayer(pub Vec3);

#[message_protocol(protocol = "MyProtocol")]
pub enum Messages {
    // Server -> Client
    SCHello(SCHello),
    SCMovePlayer(SCMovePlayer),
    SCRotPlayer(SCRotPlayer),

    // Client -> Server
    CSConnectPlayer(CSConnectPlayer),
    CSConnectAdmin(CSConnectAdmin),
    CSJoinZone(CSJoinZone),
    CSRotPlayer(CSRotPlayer),
    CSMovePlayer(CSMovePlayer),
}

// Protocol

protocolize! {
    Self = MyProtocol,
    Message = Messages,
    Component = Components,
}

pub fn protocol() -> MyProtocol {
    let mut protocol = MyProtocol::default();
    protocol.add_channel::<Channel1>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
        direction: ChannelDirection::Bidirectional,
    });
    protocol
}
