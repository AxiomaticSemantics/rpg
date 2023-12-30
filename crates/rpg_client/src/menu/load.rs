use crate::{
    game::{
        plugin::{GameState, PlayerOptions},
        state_saver::{SaveSlot, SaveSlotId, SaveSlots},
    },
    menu::main::MainRoot,
    state::AppState,
};

use ui_util::style::UiTheme;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{Changed, With},
        schedule::NextState,
        system::{ParamSet, Query, Res, ResMut, Resource},
    },
    hierarchy::{BuildChildren, ChildBuilder},
    prelude::{Deref, DerefMut},
    text::TextStyle,
    ui::{
        node_bundles::{ButtonBundle, NodeBundle, TextBundle},
        BackgroundColor, Display, Interaction, Style, Val,
    },
    utils::default,
};

#[derive(Debug, Default, Resource, Deref, DerefMut)]
pub struct SelectedSaveSlot(pub Option<SaveSlotId>);

#[derive(Component)]
pub struct LoadRoot;

#[derive(Component)]
pub struct CancelLoadButton;

#[derive(Component)]
pub struct LoadButton;

pub fn spawn_load(
    save_slots: &SaveSlots,
    builder: &mut ChildBuilder,
    ui_theme: &UiTheme,
    button: &ButtonBundle,
    frame: &Style,
    text_node_style: &Style,
    text_style: &TextStyle,
) {
    builder
        .spawn((
            LoadRoot,
            NodeBundle {
                style: frame.clone(),
                background_color: ui_theme.frame_background_color,
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(
                TextBundle::from_section("Load Character", text_style.clone())
                    .with_style(text_node_style.clone()),
            );

            let mut slot_style = ui_theme.frame_col_style.clone();
            slot_style.width = Val::Px(250.);
            slot_style.height = Val::Px(50.);

            for row in 0..6 {
                p.spawn(NodeBundle {
                    style: ui_theme.row_style.clone(),
                    ..default()
                })
                .with_children(|p| {
                    for column in 0..2 {
                        let slot_id = row * 2 + column;

                        let save_slot = &save_slots.slots[slot_id as usize];

                        let slot_string = if let Some(state) = &save_slot.state {
                            format!(
                                "{} level {} {}",
                                state.unit.name, state.unit.level, state.unit.class
                            )
                        } else {
                            "Empty Slot".into()
                        };

                        p.spawn((
                            SaveSlotId(slot_id),
                            Interaction::None,
                            NodeBundle {
                                style: slot_style.clone(),
                                background_color: ui_theme.button_theme.normal_background_color,
                                ..default()
                            },
                        ))
                        .with_children(|p| {
                            p.spawn(
                                TextBundle::from_section(slot_string, text_style.clone())
                                    .with_style(text_node_style.clone()),
                            );
                        });
                    }
                });
            }

            p.spawn(NodeBundle {
                style: ui_theme.vertical_spacer.clone(),
                ..default()
            });

            p.spawn((button.clone(), LoadButton)).with_children(|p| {
                p.spawn(TextBundle::from_section("Load", text_style.clone()));
            });

            p.spawn((button.clone(), CancelLoadButton))
                .with_children(|p| {
                    p.spawn(TextBundle::from_section("Cancel", text_style.clone()));
                });
        });
}

pub fn cancel_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<CancelLoadButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<LoadRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}

pub fn select_save_slot(
    mut selected_save_slot: ResMut<SelectedSaveSlot>,
    ui_theme: Res<UiTheme>,
    mut slot_q: Query<(&mut BackgroundColor, &Interaction, &SaveSlotId)>,
) {
    for (mut bg_color, interaction, slot) in &mut slot_q {
        match interaction {
            Interaction::Pressed => {
                selected_save_slot.0 = Some(*slot);
            }
            Interaction::Hovered => {
                *bg_color = ui_theme.button_theme.hovered_background_color;
            }
            Interaction::None => {
                if let Some(save_slot) = selected_save_slot.0 {
                    if save_slot == *slot {
                        *bg_color = ui_theme.button_theme.pressed_background_color;
                    } else {
                        *bg_color = ui_theme.button_theme.normal_background_color;
                    }
                } else {
                    *bg_color = ui_theme.button_theme.normal_background_color;
                }
            }
        }
    }
}

pub fn load_button(
    mut state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameState>,
    save_slots: Res<SaveSlots>,
    selected_slot: Res<SelectedSaveSlot>,
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<LoadButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<LoadRoot>>,
    )>,
) {
    if selected_slot.0.is_none() {
        return;
    }

    let slot_index = selected_slot.0.unwrap();

    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::None;
        menu_set.p1().single_mut().display = Display::None;

        game_state.player_config = Some(PlayerOptions {
            name: "Player".to_string(),
            class: save_slots.slots[slot_index.0 as usize]
                .state
                .as_ref()
                .unwrap()
                .unit
                .class,
        });

        state.set(AppState::GameSpawn);
    }
}
