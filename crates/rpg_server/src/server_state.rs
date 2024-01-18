use rpg_account::account::AccountId;
use rpg_chat::chat::MessageId;
use rpg_core::uid::NextUid;

use util::fs::open_read;

use bevy::ecs::{
    system::Resource,
    world::{FromWorld, World},
};

use serde_derive::{Deserialize as De, Serialize as Ser};

use std::{env, fs, path::Path};

#[derive(Debug, Ser, De)]
pub(crate) struct ServerMetadata {
    pub(crate) next_account_id: AccountId,
    pub(crate) next_message_id: MessageId,
    pub(crate) next_uid: NextUid,
}

#[derive(Resource)]
pub(crate) struct ServerMetadataResource(pub(crate) ServerMetadata);

impl FromWorld for ServerMetadataResource {
    fn from_world(world: &mut World) -> Self {
        let file_path = format!("{}/server/meta.json", env::var("RPG_SAVE_ROOT").unwrap());
        let path = Path::new(file_path.as_str());
        let file = open_read(path).unwrap();

        let metadata = serde_json::from_reader(file).unwrap();
        Self(metadata)
    }
}
