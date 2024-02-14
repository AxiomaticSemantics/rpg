use super::client::Client;

use bevy::{
    ecs::{
        entity::Entity,
        system::{Commands, Resource},
    },
    hierarchy::DespawnRecursiveExt,
    log::info,
};

use rpg_account::account::AccountId;

use bevy_renet::renet::ClientId;

use std::collections::HashMap;

#[derive(Resource, Default)]
pub(crate) struct NetworkContext {
    pub(crate) clients: HashMap<ClientId, Client>,
}

impl NetworkContext {
    pub(crate) fn get_client_from_id(&self, id: ClientId) -> Option<&Client> {
        self.clients.get(&id)
    }

    pub(crate) fn get_client_from_account_id(&self, id: AccountId) -> Option<&Client> {
        self.clients.values().find(|a| {
            if let Some(aid) = a.account_id {
                aid == id
            } else {
                false
            }
        })
    }

    // TODO rework this to be move flexible, for now this is fine
    pub(crate) fn get_client_ids_for_account_ids(
        &self,
        account_ids: &Vec<AccountId>,
    ) -> Vec<ClientId> {
        let client_ids: Vec<_> = account_ids
            .iter()
            .map(|a| {
                *self
                    .clients
                    .iter()
                    .find(|(k, v)| v.is_authenticated() && v.account_id.unwrap() == *a)
                    .unwrap()
                    .0
            })
            .collect();

        assert_eq!(client_ids.len(), account_ids.len());

        client_ids
    }

    pub(crate) fn is_client_authenticated(&self, id: ClientId) -> bool {
        if let Some(client) = self.clients.get(&id) {
            client.is_authenticated()
        } else {
            false
        }
    }

    pub(crate) fn add_client(&mut self, id: ClientId) {
        assert!(!self.clients.contains_key(&id));

        self.clients.insert(id, Client::new(id));
    }

    pub(crate) fn remove_client(&mut self, commands: &mut Commands, id: ClientId) {
        if let Some(client) = self.clients.remove(&id) {
            if client.entity != Entity::PLACEHOLDER {
                info!("removed client {id}, despawning");
                // NOTE Attempting to create an EntityCommands for entity 0v1, which doesn't exist.
                commands.entity(client.entity).despawn_recursive();
            }
        }
    }
}
