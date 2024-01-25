use bevy::{ecs::entity::Entity, math::Vec3};

use lightyear::prelude::*;
use serde_derive::{Deserialize, Serialize};

// TODO split these up into multiple protocols once the basic design is settled
use rpg_account::{
    account::{Account, AccountInfo},
    character::{CharacterInfo, CharacterRecord, CharacterSlot},
};
use rpg_chat::chat::{ChannelId, Message as ChatMessage};
use rpg_core::{
    class::Class,
    skill::SkillId,
    stat::StatUpdate,
    uid::Uid,
    unit::{HeroGameMode, UnitKind, VillainInfo},
};
use rpg_lobby::lobby::{Lobby, LobbyId, LobbyMessage};
use rpg_world::zone::ZoneId;

// Components (Unused, everything is manually managed)
#[component_protocol(protocol = "RpgProtocol")]
pub enum Components {}

// Channels

#[derive(Channel)]
pub struct Channel1;

// Messages

// Client -> Server

// Account Messages
#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSConnectPlayer;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSConnectAdmin;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSLoadAccount {
    pub name: String,
    pub password: String,
}

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSLoadAdminAccount {
    pub name: String,
    pub password: String,
}

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSCreateAccount {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSCreateCharacter {
    pub name: String,
    pub slot: CharacterSlot,
    pub class: Class,
    pub game_mode: HeroGameMode,
}

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSLobbyCreate {
    pub game_mode: HeroGameMode,
    pub name: String,
}

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSLobbyJoin(pub LobbyId);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSLobbyLeave;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSLobbyMessage {
    pub id: LobbyId,
    pub message: String,
}

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSCreateGame {
    pub game_mode: HeroGameMode,
    pub slot: CharacterSlot,
}

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSJoinGame {
    pub game_mode: HeroGameMode,
    pub slot: CharacterSlot,
}

// Chat Messages
#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSChatJoin;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSChatLeave;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSChatChannelMessage(pub ChatMessage);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSChatChannelCreate(pub String);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSChatChannelJoin(pub String);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSChatChannelLeave(pub ChannelId);

// Game Messages
#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSPlayerReady;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSPlayerLeave;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSJoinZone(pub ZoneId);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSMovePlayer;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSMovePlayerEnd;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSRotPlayer(pub Vec3);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSSkillUseDirect(pub SkillId);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSSkillUseTargeted {
    pub skill_id: SkillId,
    pub target: Vec3,
}

// Server -> Client

// Account Messages
#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCHello(pub ClientId);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCCreateAccountSuccess(pub Account);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCCreateAccountError;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLoginAccountSuccess(pub Account);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLoginAccountError;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCCreateCharacterSuccess(pub CharacterRecord);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCCreateCharacterError;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCAccount(pub Account);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCAccountInfo(pub AccountInfo);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCCharacter(pub CharacterRecord);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCCharacterInfo(pub CharacterInfo);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyCreateSuccess(pub Lobby);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyCreateError;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyJoinSuccess(pub Lobby);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyJoinError;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyLeaveSuccess;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyLeaveError;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyMessage(pub LobbyMessage);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyMessageSuccess;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyMessageError;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCGameCreateSuccess;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCGameCreateError;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCGameJoinSuccess;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCGameJoinError;

// Chat Messages
#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatJoinSuccess(pub u64);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatJoinError;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatLeave;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatChannelJoinSuccess {
    pub recent_messages: Vec<ChatMessage>,
}

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatChannelJoinError;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatChannelLeave;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatChannelMessageSuccess;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatChannelMessageError;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatMessage(pub ChatMessage);

// Game Messages
#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCPlayerJoinSuccess;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCPlayerJoinError;

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCPlayerSpawn {
    pub position: Vec3,
    pub direction: Vec3,
}

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCMovePlayer(pub Vec3);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCRotPlayer(pub Vec3);

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCStatUpdates {
    pub updates: Vec<StatUpdate>,
}

#[derive(Message, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCSpawnVillain {
    pub position: Vec3,
    pub direction: Vec3,
    pub class: Class,
    pub uid: Uid,
    pub info: VillainInfo,
}

#[message_protocol(protocol = "RpgProtocol")]
pub enum Messages {
    // Server -> Client

    // Account Messages
    SCHello(SCHello),
    SCCreateAccountSuccess(SCCreateAccountSuccess),
    SCCreateAccountError(SCCreateAccountError),
    SCLoginAccountSuccess(SCLoginAccountSuccess),
    SCLoginAccountError(SCLoginAccountError),
    SCCreateCharacterSuccess(SCCreateCharacterSuccess),
    SCCreateCharacterError(SCCreateCharacterError),
    SCAccount(SCAccount),
    SCAccountInfo(SCAccountInfo),
    SCCharacter(SCCharacter),
    SCCharacterInfo(SCCharacterInfo),
    SCLobbyCreateSuccess(SCLobbyCreateSuccess),
    SCLobbyCreateError(SCLobbyCreateError),
    SCLobbyJoinSuccess(SCLobbyJoinSuccess),
    SCLobbyJoinError(SCLobbyJoinError),
    SCLobbyLeaveSuccess(SCLobbyLeaveSuccess),
    SCLobbyLeaveError(SCLobbyLeaveError),
    SCLobbyMessageSuccess(SCLobbyMessageSuccess),
    SCLobbyMessageError(SCLobbyMessageError),
    SCLobbyMessage(SCLobbyMessage),
    SCGameCreateSuccess(SCGameCreateSuccess),
    SCGameCreateError(SCGameCreateError),
    SCGameJoinSuccess(SCGameJoinSuccess),
    SCGameJoinError(SCGameJoinError),

    // Chat Messages
    SCChatJoinSuccess(SCChatJoinSuccess),
    SCChatJoinError(SCChatJoinError),
    SCChatLeave(SCChatLeave),
    SCChatChannelJoinSuccess(SCChatChannelJoinSuccess),
    SCChatChannelJoinError(SCChatChannelJoinError),
    SCChatChannelLeave(SCChatChannelLeave),
    SCChatChannelMessageSuccess(SCChatChannelMessageSuccess),
    SCChatChannelMessageError(SCChatChannelMessageError),
    SCChatMessage(SCChatMessage),

    // Game Messages
    SCPlayerJoinSuccess(SCPlayerJoinSuccess),
    SCPlayerJoinError(SCPlayerJoinError),
    SCPlayerSpawn(SCPlayerSpawn),
    SCMovePlayer(SCMovePlayer),
    SCRotPlayer(SCRotPlayer),
    SCStatUpdates(SCStatUpdates),
    SCSpawnVillain(SCSpawnVillain),

    // Client -> Server

    // Account Messages
    CSConnectPlayer(CSConnectPlayer),
    CSConnectAdmin(CSConnectAdmin),
    CSCreateAccount(CSCreateAccount),
    CSLoadAccount(CSLoadAccount),
    CSLoadAdminAccount(CSLoadAdminAccount),
    CSCreateCharacter(CSCreateCharacter),
    CSLobbyCreate(CSLobbyCreate),
    CSLobbyJoin(CSLobbyJoin),
    CSLobbyLeave(CSLobbyLeave),
    CSLobbyMessage(CSLobbyMessage),
    CSCreateGame(CSCreateGame),
    CSJoinGame(CSJoinGame),

    // Chat Messages
    CSChatJoin(CSChatJoin),
    CSChatLeave(CSChatLeave),
    CSChatChannelMessage(CSChatChannelMessage),
    CSChatChannelCreate(CSChatChannelCreate),
    CSChatChannelJoin(CSChatChannelJoin),
    CSChatChannelLeave(CSChatChannelLeave),

    // Game Messages
    CSPlayerReady(CSPlayerReady),
    CSPlayerLeave(CSPlayerLeave),
    CSJoinZone(CSJoinZone),
    CSRotPlayer(CSRotPlayer),
    CSMovePlayer(CSMovePlayer),
    CSMovePlayerEnd(CSMovePlayerEnd),
    CSSkillUseDirect(CSSkillUseDirect),
    CSSkillUseTargeted(CSSkillUseTargeted),
}

// Protocol

protocolize! {
    Self = RpgProtocol,
    Message = Messages,
    Component = Components,
}

pub fn protocol() -> RpgProtocol {
    let mut protocol = RpgProtocol::default();
    protocol.add_channel::<Channel1>(ChannelSettings {
        mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
        direction: ChannelDirection::Bidirectional,
    });
    protocol
}
