use bevy::ecs::schedule::States;

#[derive(Default, Clone, PartialEq, Eq, Hash, Debug, States)]
pub(crate) enum AppState {
    #[default]
    Loading,
    Lobby,
    Chat,
    SpawnSimulation,
    Simulation,
}
