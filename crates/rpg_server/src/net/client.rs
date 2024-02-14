use bevy::{ecs::entity::Entity, log::info};

use bevy_renet::renet::ClientId;

use rpg_account::account::AccountId;

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum ClientType {
    #[default]
    Unknown,
    Player,
    Admin,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Client {
    pub(crate) client_id: ClientId,
    pub(crate) entity: Entity,
    pub(crate) client_type: ClientType,
    pub(crate) account_id: Option<AccountId>,
}

impl Client {
    pub(crate) fn new(client_id: ClientId) -> Self {
        Self {
            client_id,
            entity: Entity::PLACEHOLDER,
            account_id: None,
            client_type: ClientType::Unknown,
        }
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.account_id.is_some() && (self.is_player() || self.is_admin())
    }

    pub(crate) fn is_player(&self) -> bool {
        ClientType::Player == self.client_type
    }

    pub(crate) fn is_admin(&self) -> bool {
        ClientType::Admin == self.client_type
    }

    pub(crate) fn is_authenticated_player(&self) -> bool {
        self.is_player() && self.is_authenticated()
    }

    pub(crate) fn is_authenticated_admin(&self) -> bool {
        self.is_admin() && self.is_authenticated()
    }
}
