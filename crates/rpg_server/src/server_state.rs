use rpg_account::account::AccountId;
use rpg_chat::chat::MessageId;
use rpg_core::uid::NextUid;

use util::fs::{open_read, open_write};

use bevy::ecs::{
    system::Resource,
    world::{FromWorld, World},
};

use serde_derive::{Deserialize as De, Serialize as Ser};

use std::{env, path::Path};

#[derive(Debug, Ser, De)]
pub(crate) struct ServerMetadata {
    pub(crate) next_account_id: AccountId,
    pub(crate) next_message_id: MessageId,
    pub(crate) next_uid: NextUid,
    pub(crate) rng_seed: u64,
}

#[derive(Resource)]
pub(crate) struct ServerMetadataResource(pub(crate) ServerMetadata);

impl FromWorld for ServerMetadataResource {
    fn from_world(_world: &mut World) -> Self {
        let file_path = format!("{}/server/meta.bin", env::var("RPG_SAVE_ROOT").unwrap());
        let path = Path::new(file_path.as_str());

        if let Ok(file) = open_read(path) {
            Self(bincode::deserialize_from(file).unwrap())
        } else {
            let meta = ServerMetadata {
                next_account_id: AccountId(0),
                next_message_id: MessageId(0),
                next_uid: NextUid::default(),
                rng_seed: 0,
            };

            let file = open_write(path).unwrap();
            bincode::serialize_into(file, &meta).unwrap();

            Self(meta)
        }
    }
}
