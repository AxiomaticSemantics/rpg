use crate::{style, widgets};

use bevy::{
    app::{App, Plugin, PreUpdate, Startup, Update},
    asset::{AssetServer, Handle},
    ecs::{
        schedule::IntoSystemConfigs,
        system::Resource,
        world::{FromWorld, World},
    },
    text::Font,
};

#[derive(Resource)]
pub struct UiFont {
    pub prime: Handle<Font>,
    pub fira_sans: Handle<Font>,
}

impl FromWorld for UiFont {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource_mut::<AssetServer>();

        UiFont {
            fira_sans: server.load("fonts/FiraNerd-Medium.ttf"),
            prime: server.load("fonts/courier_prime-regular.ttf"),
        }
    }
}

pub struct UiUtilPlugin;

impl Plugin for UiUtilPlugin {
    fn build(&self, app: &mut App) {
        println!("Initializing UI plugin.");

        app.init_resource::<UiFont>()
            .add_systems(Startup, (widgets::setup_focus, style::insert_theme))
            .add_systems(
                PreUpdate,
                widgets::edit_focus_update.after(bevy::ui::ui_focus_system),
            )
            .add_systems(
                Update,
                (
                    widgets::slider_update,
                    widgets::resize_view,
                    widgets::mouse_scroll,
                    widgets::edit_text,
                    style::button_style,
                ),
            );
    }
}
