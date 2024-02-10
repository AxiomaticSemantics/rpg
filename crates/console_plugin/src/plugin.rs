use crate::console::{Console, HistoryIndex, HistoryItem};
// use util::{to_color_rgb, to_color_rgba, to_vec3, to_vec4};

use ui_util::{
    style::UiTheme,
    widgets::{EditText, FocusedElement, List, ListPosition, NewlineBehaviour, ResizeableView},
};

use ab_glyph::GlyphId;

use bevy::{
    app::{App, Plugin, PreUpdate, Startup, Update},
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        entity::Entity,
        query::{Changed, With, Without},
        schedule::{common_conditions::*, Condition, IntoSystemConfigs, States},
        system::{Commands, ParamSet, Query, Res, ResMut, Resource},
    },
    hierarchy::BuildChildren,
    input::{keyboard::KeyCode, ButtonInput},
    log::info,
    render::{camera::Camera, color::Color},
    text::{FontAtlasSet, Text, TextLayoutInfo, TextSection, TextStyle},
    transform::components::Transform,
    ui::{
        node_bundles::{ImageBundle, NodeBundle, TextBundle},
        AlignContent, AlignItems, AlignSelf, BackgroundColor, Display, FlexDirection, FlexWrap,
        Interaction, Style, TargetCamera, UiImage, UiRect, Val, ZIndex,
    },
    utils::default,
};

use clap::{error::ErrorKind, CommandFactory};

#[derive(Component)]
pub struct ConsoleRoot;

#[derive(Component)]
pub struct ConsoleList;

#[derive(Component)]
pub struct ConsoleInput;

#[derive(Component)]
pub struct ConsoleHistoryItem;

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing console plugin.");

        app.add_systems(PreUpdate, setup.run_if(not(resource_exists::<Console>)))
            .add_systems(
                Update,
                (
                    (toggle_console.before(ui_util::widgets::edit_text)),
                    (console_action, handle_command, update_history)
                        .after(ui_util::widgets::edit_text)
                        .chain(),
                )
                    .run_if(
                        resource_exists::<Console>
                            .and_then(|r: Res<Console>| r.ui_root != Entity::PLACEHOLDER),
                    ),
            );

        /* match cmd = ConsoleCommands::command().try_get_matches_from(["light", "--dir", "3.0", "1.0", "4.0"]) {
            Ok(res) => { let light = res.subcommand_matches("light").unwrap(); }
        }; */
    }
}

fn setup(mut commands: Commands, ui_theme: Res<UiTheme>, camera_q: Query<Entity, With<Camera>>) {
    let mut container_hidden_style = ui_theme.frame_col_style.clone();
    container_hidden_style.display = Display::None;

    let Ok(camera) = camera_q.get_single() else {
        return;
    };

    let mut console = Console::new(Entity::PLACEHOLDER, Entity::PLACEHOLDER);

    let ui_root = commands
        .spawn((
            TargetCamera(camera),
            ConsoleRoot,
            NodeBundle {
                style: container_hidden_style.clone(),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                ResizeableView,
                Interaction::None,
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        //flex_grow: 1.0,
                        //justify_content: JustifyContent::Center,
                        min_width: Val::Px(800.),
                        min_height: Val::Px(600.),
                        align_self: AlignSelf::End,
                        align_items: AlignItems::Start,
                        border: UiRect::all(ui_theme.border),
                        ..default()
                    },
                    border_color: ui_theme.border_color,
                    background_color: ui_theme.frame_background_color,
                    ..default()
                },
            ))
            .with_children(|p| {
                p.spawn(NodeBundle {
                    style: Style {
                        align_self: AlignSelf::End,
                        align_items: AlignItems::Start,
                        //min_width: Val::Percent(100.),
                        ..default()
                    },
                    background_color: ui_theme.frame_background_color,
                    ..default()
                })
                .with_children(|p| {
                    p.spawn((
                        Interaction::None,
                        ConsoleList,
                        List {
                            position: ListPosition::Position(0.),
                            scrollable: true,
                        },
                        NodeBundle {
                            style: Style {
                                align_self: AlignSelf::Stretch,
                                flex_direction: FlexDirection::ColumnReverse,
                                min_width: Val::Percent(100.),
                                ..default()
                            },
                            background_color: Color::rgba(0.70, 0.10, 0.10, 0.9).into(),
                            ..default()
                        },
                    ))
                    .with_children(|p| {
                        for _ in 0..20 {
                            let text =
                                TextBundle::from_section("k", ui_theme.text_style_regular.clone())
                                    .with_style(Style {
                                        width: Val::Percent(100.),
                                        ..default()
                                    });

                            p.spawn(text);
                        }
                    });

                    let prefix = "$ ";
                    let mut edit_text = EditText::new(NewlineBehaviour::Consume);
                    edit_text.set_prefix(Some(prefix));

                    let text_bundle =
                        TextBundle::from_section(prefix, ui_theme.text_style_regular.clone());
                    let input = p
                        .spawn((
                            ConsoleInput,
                            HistoryIndex::None,
                            edit_text,
                            Interaction::default(),
                            text_bundle,
                        ))
                        .id();
                    console.input = input;
                });
            });
        })
        .id();

    console.ui_root = ui_root;
    /*
    let sprite = commands
        .spawn((
            CursorSprite,
            ImageBundle {
                ..default() //z_index: ZIndex::Global(10),
            },
        ))
        .id();
    */

    commands.insert_resource(console);
}

#[derive(Component)]
pub struct CursorSprite;

fn toggle_console(
    console: Res<Console>,
    mut focused: ResMut<FocusedElement>,
    input: Res<ButtonInput<KeyCode>>,
    mut console_q: Query<&mut Style, (With<ConsoleRoot>, Without<ConsoleList>)>,
    mut list_q: Query<&mut Style, (With<ConsoleList>, Without<ConsoleRoot>)>,
) {
    if input.just_pressed(KeyCode::F4) {
        let mut style = console_q.single_mut();
        if style.display == Display::None {
            //style.bottom = Val::Px(4320.);
            focused.0 = Some(console.input);
            style.display = Display::Flex;

            let mut list_style = list_q.single_mut();
            list_style.bottom = Val::Px(576.);
        } else {
            focused.0 = None::<Entity>;
            style.display = Display::None;
        }
    }
}

fn console_action(
    //time: Res<Time>,
    //font_atlas_sets: Res<Assets<FontAtlasSet>>,
    //font_assets: Res<FontAssets>,
    console: Res<Console>,
    input: Res<ButtonInput<KeyCode>>,
    mut console_ui_set: ParamSet<(
        //Query<(&mut Image, &mut Transform), (With<CursorSprite>, Without<Text>)>,
        Query<
            (
                &mut EditText,
                &mut Text,
                &Transform,
                &TextLayoutInfo,
                &mut HistoryIndex,
            ),
            Without<CursorSprite>,
        >,
    )>,
) {
    return;

    /*
    if console.command_history.history.is_empty() {
        return;
    }*/

    let (index, _translation, position) = {
        let text_q = console_ui_set.p0();
        let (edit_text, _input_text, input_transform, input_layout, input_index) = text_q.single();

        /*
        println!(
            "pos: {} layout: {:?}",
            input_transform.translation, input_layout
        );*/

        let mut translation = input_transform.translation;
        //translation.x = input_layout.logical_size.x;

        (input_index.clone(), translation, edit_text.cursor.position)
    };

    /*{
        let mut sprite_q = console_ui_set.p0();
        let (mut sprite, _sprite_transform) = sprite_q.single_mut();

        if let Some(set) = font_atlas_sets.get(&font_assets.fira_sans.cast_weak::<FontAtlasSet>()) {
            for atlas in set.iter() {
                for a in atlas.1.iter() {
                    //println!("{}", set.has_glyph(GlyphId(' ' as u16), ab_glyph::Point { x: 10., y: 16. }, 32.));
                }
            }
        }

        sprite.left = Val::Px(-12. + 12. * position as f32);
        sprite.width = Val::Px(12.);
    } */

    {
        let mut text_q = console_ui_set.p0();
        let (mut edit_text, mut input_text, _input_transform, _input_layout, mut input_index) =
            text_q.single_mut();
        if input.just_pressed(KeyCode::ArrowUp) {
            //println!("history pre-inc {history_index:?}");
            //input_index.increment(Some(console.command_history.max));
            //println!("history post-inc {history_index:?}");
        } else if input.just_pressed(KeyCode::ArrowDown) {
            //println!("history pre-dec {history_index:?}");
            input_index.decrement();
            //println!("history post-inc {history_index:?}");
        }

        if index == *input_index {
            return;
        }
        //println!("diff {curr_index:?} {input_index:?}");

        match *input_index {
            HistoryIndex::None => {
                input_text.sections[0].value = edit_text.prefix.unwrap_or_default().into();
                let len = input_text.sections[0].value.len();
                edit_text.cursor.set_max(Some(len + 1));
                edit_text.cursor.position = len + 1;
            }
            index => {
                if console.command_history.max.gt(index) {
                    let HistoryIndex::Some(index) = index else {
                        panic!("bad variant");
                    };
                    input_text.sections[0].value = format!(
                        "{}{}",
                        edit_text.prefix.unwrap_or_default(),
                        console.command_history.history[index].item,
                    );

                    let len = input_text.sections[0].value.len();
                    edit_text.cursor.set_max(Some(len + 1));
                    edit_text.cursor.position = len + 1;
                }
            }
        }
    }
}

pub fn update_history(
    mut item_q: Query<&mut Text, With<ConsoleHistoryItem>>,
    console: Res<Console>,
) {
    return;

    if !console.is_changed() {
        return;
    }

    let history_len = console.history.history.len();
    if history_len == 0 {
        return;
    }

    for (index, mut text) in item_q.iter_mut().enumerate() {
        if let Some(console_history_item) = console.history.history.get(index) {
            if text.sections[0].value != console_history_item.item {
                text.sections[0].value = console_history_item.item.clone();
            }
        }
    }
}

// TODO abstract more things so command handling is moved elsewhere
fn handle_command(
    mut console: ResMut<Console>,
    mut console_set: ParamSet<(
        Query<(&mut EditText, &mut Text, &mut HistoryIndex), With<ConsoleInput>>,
        Query<&mut Text, With<ConsoleHistoryItem>>,
    )>,
    /* mut command_set: ParamSet<(
        Query<(&mut DirectionalLight, &mut Transform)>,
        Query<&Transform, With<Unit>>,
    )> */
) {
    return;
    let mut console_q = console_set.p0();
    let (mut text_edit, mut text, mut history_index) = console_q.single_mut();
    if !text_edit.consume {
        return;
    }

    let min = text_edit.prefix.unwrap_or_default().len();
    if text.sections[0].value.len() <= min {
        text_edit.consume = false;
        return;
    }

    assert!(text.sections[0].value.len() > min);

    let item = text.sections[0].value.clone();
    // FIXME EditText should be handling this
    println!("input: `{item}`");
    text.sections[0].value.truncate(min);
    text_edit.consume = false;
    text_edit.cursor.reset_position();
    //text_edit.cursor.position = min + 1;
    *history_index = HistoryIndex::None;

    console.update_history(item.clone(), false);
    let cmds: Vec<&str> = item.split_terminator(' ').skip(1).collect();
    /*
    let cmd = ConsoleCommands::command().try_get_matches_from(cmds);

    //Err(clap::Error::new(ErrorKind::InvalidValue)),

    let item = match cmd {
        Ok(cmd) => {
            let cmd = cmd.subcommand().unwrap();

            let res = match cmd.0 {
                /*
                "light" => {
                    let mut light_q = command_set.p0();
                    let (mut light, mut transform) = light_q.single_mut();

                    if let Some(rot) = cmd.1.get_many::<f32>("rot") {
                        let len = rot.len();
                        let output = if len == 0 {
                            println!("ok rotation {:?}", transform.rotation.xyz());
                            transform.rotation.xyz().to_string()
                        } else if len == 3 {
                            let vec3 = to_vec3(rot.map(|v| *v).collect());

                            transform.rotation = Quat::from_xyzw(vec3.x, vec3.y, vec3.z, 1.);

                            vec3.to_string()
                        } else if len == 4 {
                            let vec4 = to_vec4(rot.map(|v| *v).collect());

                            transform.rotation = Quat::from_vec4(vec4);

                            vec4.to_string()
                        } else {
                            "Either 3 or 4 values are expected to set light rotation".into()
                        };

                        output
                    } else if let Some(pos) = cmd.1.get_many::<f32>("pos") {
                        let len = pos.len();
                        let output = if len == 0 {
                            println!("ok position {:?}", transform.translation);
                            transform.translation.to_string()
                        } else if len == 3 {
                            let vec3 = to_vec3(pos.map(|v| *v).collect::<Vec<f32>>());
                            transform.translation = vec3;

                            vec3.to_string()
                        } else {
                            "3 values are expected to set light position".into()
                        };

                        output
                    } else if let Some(color) = cmd.1.get_many::<f32>("color") {
                        let len = color.len();
                        if len == 0 {
                            format!("{:?}", light.color)
                        } else {
                            let vec: Vec<f32> = color.map(|v| *v).collect();
                            let color = if len == 3 {
                                to_color_rgb(vec)
                            } else if len == 4 {
                                to_color_rgba(vec)
                            } else {
                                None
                            };

                            if let Some(color) = color {
                                light.color = color;
                                println!("setting dir light color to {color:?}");

                                format!("{:?}", color)
                            } else {
                                format!("Either 3 or 4 values are expected to set the light color")
                            }
                        }
                    } else {
                        //let light_q = command_set.p0();
                        //let (light, transform, rotation, color) = light_q.single();
                        transform.translation.to_string()
                        //"light foo".into()
                    }
                }*/
                /*
                "time" => {
                    if let Some(mut set) = cmd.1.get_many::<f32>("set") {
                        let amount = set.next().unwrap();
                        game_time
                            .watch
                            .set_elapsed(std::time::Duration::from_secs_f32(*amount));

                        format!("Setting game time to: {}", amount)
                    } else {
                        format!("Game time is: {}", game_time.watch.elapsed_secs())
                    }
                } */
                /*
                "state" => {
                    if let Some(mut set) = cmd.1.get_many::<AppState>("set") {
                        let wanted_state = set.next().unwrap();
                        match wanted_state {
                            AppState::Game => {
                                if *state == AppState::GameOver {
                                    next_state.set(*wanted_state);

                                    format!("Setting state: {} -> {}", state.get(), wanted_state)
                                } else {
                                    format!(
                                        "You can't switch to {} from {}",
                                        wanted_state,
                                        state.get(),
                                    )
                                }
                            }
                            AppState::Menu => "not yet".into(),
                            AppState::Shutdown => {
                                next_state.set(*wanted_state);
                                "Shutting down".into()
                            }
                            _ => "not yet".into(),
                        }
                    } else {
                        format!("{}", state.get())
                    }
                }*/
                /*
                "volume" => {
                    if let Some(mut global) = cmd.1.get_many::<f32>("global") {
                        let curr_volume: f32 = global_volume.volume.get();
                        let wanted_volume = global.next().unwrap().clamp(0., 1.);

                        global_volume.volume = VolumeLevel::new(wanted_volume);
                        format!(
                            "Setting global volume from {:3.2} to {:3.2}",
                            curr_volume * 100.,
                            wanted_volume * 100.
                        )
                    } else {
                        format!("Global volume is {:3.2}", global_volume.volume.get() * 100.)
                    }
                }*/
                /*
                "pause" => {
                    if *state == AppState::Game {
                        match &mut game_state.state {
                            v if *v == PlayState::Paused(PauseState::User) => {
                                *v = PlayState::Play;
                                format!(
                                    "Setting pause state from {:?} to {:?}",
                                    PlayState::Paused(PauseState::User),
                                    PlayState::Play
                                )
                            }
                            v if *v == PlayState::Play => {
                                *v = PlayState::Paused(PauseState::User);
                                format!(
                                    "Setting pause state from {:?} to {:?}",
                                    PlayState::Play,
                                    PlayState::Paused(PauseState::User)
                                )
                            }
                            _ => "Unable to toggle pause stat during reward pause.".into(),
                        }
                    } else {
                        "Unable to pause game while not in game state.".into()
                    }
                }*/
                cmd => {
                    println!("unhandled command: {cmd}");

                    cmd.to_string()
                }
            };

            println!("cmd parse success: {cmd:?} {} {res}", cmd.0);

            res
        }
        Err(e) if e.kind() != ErrorKind::InvalidValue => {
            println!("cmd parse failure: {e:?}");

            let mut res = e.render().to_string();
            if res.ends_with('\n') {
                res.pop();
            }

            res
        }
        Err(e) => {
            let mut res = e.render().to_string();
            if res.ends_with('\n') {
                res.pop();
            }

            res
        }
    };*/

    //console.update_history(item.clone(), false);
}
