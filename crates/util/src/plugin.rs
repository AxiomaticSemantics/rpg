use bevy::{
    app::{App, Plugin},
    asset::AssetApp,
};

use crate::assets::json::{JsonAssetLoader, JsonSource};

pub struct UtilityPlugin;

impl Plugin for UtilityPlugin {
    fn build(&self, app: &mut App) {
        println!("Initializing utility plugin.");

        app.init_asset::<JsonSource>()
            .init_asset_loader::<JsonAssetLoader<JsonSource>>();
    }
}
