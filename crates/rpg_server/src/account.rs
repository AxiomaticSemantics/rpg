use rpg_account::account::{Account, AdminAccount};

use bevy::{
    ecs::{bundle::Bundle, component::Component},
    prelude::{Deref, DerefMut},
};

#[derive(Debug, Deref, DerefMut, Component)]
pub(crate) struct AccountInstance(pub(crate) Account);

#[derive(Debug, Deref, DerefMut, Component)]
pub(crate) struct AdminAccountInstance(pub(crate) AdminAccount);

#[derive(Bundle)]
pub(crate) struct AccountInstanceBundle {
    pub account: AccountInstance,
}

#[derive(Bundle)]
pub(crate) struct AdminAccountInstanceBundle {
    pub account: AdminAccountInstance,
}
