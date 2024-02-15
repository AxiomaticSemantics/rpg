use glam::Vec3;

use serde_derive::{Deserialize, Serialize};

// TODO split these up into multiple protocols once the basic design is settled
use rpg_account::{
    account::{Account, AccountInfo},
    character::{CharacterInfo, CharacterRecord, CharacterSlot},
};
use rpg_chat::chat::{ChannelId, Message as ChatMessage};
use rpg_core::{
    class::Class,
    combat::{CombatResult, DamageResult},
    game_mode::GameMode,
    item::ItemDrops,
    skill::{Skill, SkillId, SkillSlot, SkillTarget},
    stat::StatUpdate,
    uid::{InstanceUid, Uid},
    unit::VillainInfo,
};
use rpg_lobby::lobby::{Lobby, LobbyId, LobbyMessage};
use rpg_world::zone::ZoneId;

use bevy_ecs::event::Event;
use renet::{ChannelConfig, SendType};

use std::time::Duration;

// Channels

pub enum ClientChannel {
    Message,
}
pub enum ServerChannel {
    Message,
}

impl From<ClientChannel> for u8 {
    fn from(channel_id: ClientChannel) -> Self {
        match channel_id {
            ClientChannel::Message => 0,
        }
    }
}

impl ClientChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![ChannelConfig {
            channel_id: Self::Message.into(),
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::ZERO,
            },
        }]
    }
}

impl From<ServerChannel> for u8 {
    fn from(channel_id: ServerChannel) -> Self {
        match channel_id {
            ServerChannel::Message => 0,
        }
    }
}

impl ServerChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![ChannelConfig {
            channel_id: Self::Message.into(),
            max_memory_usage_bytes: 10 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(200),
            },
        }]
    }
}

// Messages

// Client -> Server

// Account Messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSConnectPlayer;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSConnectAdmin;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSLoadAccount {
    pub name: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSLoadAdminAccount {
    pub name: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSCreateAccount {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSCreateCharacter {
    pub name: String,
    pub slot: CharacterSlot,
    pub class: Class,
    pub game_mode: GameMode,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSLobbyCreate {
    pub game_mode: GameMode,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSLobbyJoin(pub LobbyId);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSLobbyLeave;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSLobbyMessage {
    pub id: LobbyId,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSCreateGame {
    pub game_mode: GameMode,
    pub slot: CharacterSlot,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSJoinGame {
    pub game_mode: GameMode,
    pub slot: CharacterSlot,
}

// Chat Messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSChatJoin;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSChatLeave;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSChatChannelMessage(pub ChatMessage);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSChatChannelCreate(pub String);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSChatChannelJoin(pub String);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSChatChannelLeave(pub ChannelId);

// Game Messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSClientReady;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSPlayerLeave;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSPlayerJoin;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSJoinZone(pub ZoneId);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSMovePlayer;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSMovePlayerEnd;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSRotPlayer(pub Vec3);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSSkillUseDirect(pub SkillId);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSSkillUseTargeted {
    pub skill_id: SkillId,
    pub target: Vec3,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSItemDrop(pub Uid);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSItemPickup(pub Uid);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CSPlayerRevive;

// Server -> Client

// Account Messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCHello;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCCreateAccountSuccess(pub Account);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCCreateAccountError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLoginAccountSuccess(pub Account);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLoginAccountError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCCreateCharacterSuccess(pub CharacterRecord);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCCreateCharacterError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCAccount(pub Account);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCAccountInfo(pub AccountInfo);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCCharacter(pub CharacterRecord);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCCharacterInfo(pub CharacterInfo);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyCreateSuccess(pub Lobby);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyCreateError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyJoinSuccess(pub Lobby);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyJoinError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyLeaveSuccess;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyLeaveError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyMessage(pub LobbyMessage);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyMessageSuccess;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCLobbyMessageError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCGameCreateSuccess(pub GameMode);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCGameCreateError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCGameJoinSuccess(pub GameMode);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCGameJoinError;

// Chat Messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatJoinSuccess(pub u64);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatJoinError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatLeave;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatChannelJoinSuccess {
    pub recent_messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatChannelJoinError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatChannelLeave;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatChannelMessageSuccess;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatChannelMessageError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCChatMessage(pub ChatMessage);

// Game Messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCPlayerJoinSuccess;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCPlayerJoinError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCPlayerLeave(pub Uid);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCPlayerSpawn {
    pub position: Vec3,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCPlayerRevive {
    pub position: Vec3,
    pub hp: u32,
    pub xp_total: u64,
    pub xp_loss: u64,
    pub deaths: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCMovePlayer(pub Vec3);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCMovePlayerEnd(pub Vec3);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCRotPlayer(pub Vec3);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCMoveUnit {
    pub uid: Uid,
    pub position: Vec3,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCMoveUnitEnd {
    pub uid: Uid,
    pub position: Vec3,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCRotUnit {
    pub uid: Uid,
    pub direction: Vec3,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCStatUpdate(pub StatUpdate);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCStatUpdates(pub Vec<StatUpdate>);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCSpawnSkill {
    pub instance_uid: InstanceUid,
    pub id: SkillId,
    pub owner_uid: Uid,
    pub target: SkillTarget,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCDespawnSkill(pub InstanceUid);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCSpawnItem {
    pub position: Vec3,
    pub items: ItemDrops,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCSpawnItems {
    pub position: Vec3,
    pub items: ItemDrops,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCDespawnItem(pub Uid);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCSpawnHero {
    pub position: Vec3,
    pub uid: Uid,
    pub name: String,
    pub class: Class,
    pub level: u8,
    pub deaths: Option<u32>,
    pub skills: Vec<Skill>,
    pub skill_slots: Vec<SkillSlot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCSpawnVillain {
    pub position: Vec3,
    pub direction: Vec3,
    pub uid: Uid,
    pub level: u8,
    pub info: VillainInfo,
    pub skills: Vec<Skill>,
    pub skill_slots: Vec<SkillSlot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCHeroDeath(pub Uid);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCVillainDeath(pub Uid);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCHeroRevive(pub Vec3);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCDespawnCorpse(pub Uid);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCCombatResult(pub CombatResult);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCUnitAnim {
    pub uid: Uid,
    pub anim: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCUnitAttack {
    pub uid: Uid,
    pub skill_id: SkillId,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCDamage {
    pub uid: Uid,
    pub damage: DamageResult,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCItemPickup(pub Uid);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCItemDrop(pub Uid);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCItemStore(pub Uid);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCZoneLoad(pub ZoneId);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SCZoneUnload(pub ZoneId);

/// Server -> Client
#[derive(Event, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ServerMessage {
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
    SCPlayerLeave(SCPlayerLeave),
    SCPlayerSpawn(SCPlayerSpawn),
    SCMovePlayer(SCMovePlayer),
    SCMovePlayerEnd(SCMovePlayerEnd),
    SCRotPlayer(SCRotPlayer),
    SCMoveUnit(SCMoveUnit),
    SCMoveUnitEnd(SCMoveUnitEnd),
    SCRotUnit(SCRotUnit),
    SCStatUpdate(SCStatUpdate),
    SCStatUpdates(SCStatUpdates),
    SCSpawnSkill(SCSpawnSkill),
    SCDespawnSkill(SCDespawnSkill),
    SCSpawnItem(SCSpawnItem),
    SCSpawnItems(SCSpawnItems),
    SCDespawnItem(SCDespawnItem),
    SCSpawnHero(SCSpawnHero),
    SCSpawnVillain(SCSpawnVillain),
    SCHeroDeath(SCHeroDeath),
    SCVillainDeath(SCVillainDeath),
    SCPlayerRevive(SCPlayerRevive),
    SCHeroRevive(SCHeroRevive),
    SCDespawnCorpse(SCDespawnCorpse),
    SCCombatResult(SCCombatResult),
    SCDamage(SCDamage),
    SCUnitAnim(SCUnitAnim),
    SCUnitAttack(SCUnitAttack),
    SCItemPickup(SCItemPickup),
    SCItemDrop(SCItemDrop),
    SCItemStore(SCItemStore),
    SCZoneLoad(SCZoneLoad),
    SCZoneUnload(SCZoneUnload),
}

/// Client -> Server
#[derive(Event, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ClientMessage {
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
    CSItemDrop(CSItemDrop),
    CSItemPickup(CSItemPickup),
    CSClientReady(CSClientReady),
    CSPlayerRevice(CSPlayerRevive),
    CSPlayerJoin(CSPlayerJoin),
    CSPlayerLeave(CSPlayerLeave),
    CSJoinZone(CSJoinZone),
    CSRotPlayer(CSRotPlayer),
    CSMovePlayer(CSMovePlayer),
    CSMovePlayerEnd(CSMovePlayerEnd),
    CSSkillUseDirect(CSSkillUseDirect),
    CSSkillUseTargeted(CSSkillUseTargeted),
}
