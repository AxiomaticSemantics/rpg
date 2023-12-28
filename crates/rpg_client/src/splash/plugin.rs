use crate::{assets::TextureAssets, loader::plugin::OutOfGameCamera, state::AppState};

use ui_util::style::UiTheme;
use util::cleanup::CleanupStrategy;

use bevy::{
    app::{App, Plugin, Update},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        schedule::{common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter, OnExit},
        system::{Commands, ParamSet, Query, Res, ResMut},
    },
    hierarchy::{BuildChildren, DespawnRecursiveExt},
    input::{keyboard::KeyCode, ButtonInput},
    math::{Vec2, Vec3},
    render::{color::Color, view::visibility::Visibility},
    sprite::{Sprite, SpriteBundle},
    text::{BreakLineOn, JustifyText, Text, Text2dBounds, Text2dBundle, TextSection, TextStyle},
    transform::components::Transform,
    utils::{default, Duration},
    window::{PrimaryWindow, Window},
};

use bevy_tweening::{lens::*, *};

use std::collections::VecDeque;

pub struct SplashScreenPlugin;

impl Plugin for SplashScreenPlugin {
    fn build(&self, app: &mut App) {
        println!("Initializing Splash Screen plugin.");

        app.add_systems(OnEnter(AppState::Splash), setup_splash_screen)
            .add_systems(
                Update,
                update_splash_screen.run_if(in_state(AppState::Splash)),
            )
            .add_systems(
                OnExit(AppState::Splash),
                util::cleanup::cleanup::<SplashScreen>,
            );
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum TweenState {
    #[default]
    Pending,
    Active,
    Complete,
}

pub struct TweenInfo<T> {
    pub state: TweenState,
    pub entity: Entity,
    pub tween: Option<Tween<T>>,
}

impl<T: Component> TweenInfo<T> {
    pub fn new(entity: Entity, tween: Option<Tween<T>>) -> Self {
        Self {
            state: TweenState::Pending,
            entity,
            tween,
        }
    }
}

pub enum TweenKind {
    Translation(TweenInfo<Transform>),
    Rotation(TweenInfo<Transform>),
    Scale(TweenInfo<Transform>),
    TextColor(TweenInfo<Text>),
    SpriteColor(TweenInfo<Sprite>),
}

impl TweenKind {
    fn state(&self) -> &TweenState {
        match self {
            Self::Translation(t) | Self::Rotation(t) | Self::Scale(t) => &t.state,
            Self::TextColor(t) => &t.state,
            Self::SpriteColor(t) => &t.state,
        }
    }

    fn entity(&self) -> Entity {
        match self {
            Self::Translation(t) | Self::Rotation(t) | Self::Scale(t) => t.entity,
            Self::TextColor(t) => t.entity,
            Self::SpriteColor(t) => t.entity,
        }
    }

    fn state_mut(&mut self) -> &mut TweenState {
        match self {
            Self::Translation(t) | Self::Rotation(t) | Self::Scale(t) => &mut t.state,
            Self::TextColor(t) => &mut t.state,
            Self::SpriteColor(t) => &mut t.state,
        }
    }

    fn set_state(&mut self, state: TweenState) {
        *self.state_mut() = state;
    }
}

pub struct Sequence {
    pub state: TweenState,
    pub tweens: Vec<TweenKind>,
}

impl Sequence {
    pub fn new(tweens: Vec<TweenKind>) -> Self {
        Self {
            state: TweenState::Pending,
            tweens,
        }
    }

    pub fn start(
        &mut self,
        commands: &mut Commands,
        target_q: &mut Query<(
            &mut Visibility,
            Option<&Animator<Transform>>,
            Option<&Animator<Sprite>>,
            Option<&Animator<Text>>,
        )>,
    ) {
        assert_eq!(self.state, TweenState::Pending);
        assert!(!self.tweens.is_empty());
        if self.state != TweenState::Pending {
            return;
        }

        assert!(self
            .tweens
            .iter()
            .all(|t| t.state() == &TweenState::Pending));

        for tween in &mut self.tweens {
            assert_eq!(tween.state(), &TweenState::Pending);

            let (mut visibility, _, _, _) = target_q.get_mut(tween.entity()).unwrap();
            if *visibility == Visibility::Hidden {
                println!("setting target visible");
                *visibility = Visibility::Visible;
            }

            match tween {
                TweenKind::Translation(t) | TweenKind::Rotation(t) | TweenKind::Scale(t) => {
                    let tween = t.tween.take().unwrap();
                    commands.entity(t.entity).insert(Animator::new(tween));
                }
                //TweenKind::TextTransform(t)
                //| TweenKind::TextRotation(t)
                //| TweenKind::TextScale(t)
                TweenKind::TextColor(t) => {
                    let tween = t.tween.take().unwrap();
                    commands.entity(t.entity).insert(Animator::new(tween));
                }
                TweenKind::SpriteColor(t) => {
                    let tween = t.tween.take().unwrap();
                    commands.entity(t.entity).insert(Animator::new(tween));
                }
            }
            tween.set_state(TweenState::Active);
        }

        self.state = TweenState::Active;
    }

    fn clear(
        &mut self,
        target_q: &mut Query<(
            &mut Visibility,
            Option<&Animator<Transform>>,
            Option<&Animator<Sprite>>,
            Option<&Animator<Text>>,
        )>,
    ) -> bool {
        if self.is_complete(target_q) {
            self.tweens.clear();
            self.state = TweenState::Complete;

            true
        } else {
            false
        }
    }

    fn is_complete(
        &self,
        target_q: &Query<(
            &mut Visibility,
            Option<&Animator<Transform>>,
            Option<&Animator<Sprite>>,
            Option<&Animator<Text>>,
        )>,
    ) -> bool {
        self.tweens.iter().all(|t| {
            let (_, transform, text, sprite) = target_q.get(t.entity()).unwrap();

            (transform.is_some_and(|t| t.tweenable().progress() >= 1.0) || transform.is_none())
                && (text.is_some_and(|t| t.tweenable().progress() >= 1.0) || text.is_none())
                && (sprite.is_some_and(|t| t.tweenable().progress() >= 1.0) || sprite.is_none())
        })
    }

    pub fn update(
        &mut self,
        target_q: &mut Query<(
            &mut Visibility,
            Option<&Animator<Transform>>,
            Option<&Animator<Sprite>>,
            Option<&Animator<Text>>,
        )>,
    ) {
        let complete = self.is_complete(target_q);
        if complete {
            self.state = TweenState::Complete;
            self.clear(target_q);
        }
    }
}

#[derive(Debug)]
enum SplashScreenState {
    Pending,
    Active,
    Complete,
}

#[derive(Component)]
pub struct SplashScreen {
    sequences: VecDeque<Sequence>,
    state: SplashScreenState,
}

/*
impl std::fmt::Debug for SplashScreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SplashScreen")
            .field("sequences", &self.sequences)
            .field("state", &self.state)
            .finish()
    }
}*/

impl SplashScreen {
    pub fn new(sequences: VecDeque<Sequence>) -> Self {
        Self {
            sequences,
            state: SplashScreenState::Pending,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.sequences.is_empty()
    }

    pub fn update(
        &mut self,
        commands: &mut Commands,
        target_q: &mut Query<(
            &mut Visibility,
            Option<&Animator<Transform>>,
            Option<&Animator<Sprite>>,
            Option<&Animator<Text>>,
        )>,
    ) {
        match self.state {
            SplashScreenState::Pending => {
                self.state = SplashScreenState::Active;
                if let Some(part) = self.sequences.front_mut() {
                    if part.state == TweenState::Pending {
                        part.start(commands, target_q);
                    }
                } else {
                    panic!("no sequence");
                }
            }
            SplashScreenState::Active => {
                let sequence = &mut self.sequences[0];
                match sequence.state {
                    TweenState::Complete => {
                        self.sequences.pop_front();
                    }
                    TweenState::Pending => {
                        sequence.start(commands, target_q);
                    }
                    TweenState::Active => {
                        sequence.update(target_q);
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Component)]
struct SplashScreenCleanup;

#[derive(Component)]
struct SplashScreenCamera;

fn setup_splash_screen(
    mut commands: Commands,
    ui_theme: Res<UiTheme>,
    texture_assets: Res<TextureAssets>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let bevy_logo = texture_assets.bevy_logo.clone_weak();

    let window = window_q.single();

    /*commands.spawn((
        SplashScreenCamera,
        CleanupStrategy::Despawn,
        Camera2dBundle::default(),
    ));*/

    let bevy_logo_entity = commands
        .spawn((
            SplashScreenCleanup,
            CleanupStrategy::DespawnRecursive,
            SpriteBundle {
                texture: bevy_logo,
                transform: Transform::from_scale(Vec3::splat(0.5)),
                visibility: Visibility::Hidden,
                ..default()
            },
        ))
        .id();

    let mut text_style = ui_theme.text_style.clone();
    text_style.color = ui_theme.text_color_light;
    text_style.font_size = ui_theme.font_size_xtra_large;

    let box_size = Vec2::new(window.width() / 2., window.height() / 8.);

    let mut top_text_entity = Entity::PLACEHOLDER;
    let mut bottom_text_entity = Entity::PLACEHOLDER;

    let credits_entity = commands
        .spawn((
            SplashScreenCleanup,
            CleanupStrategy::DespawnRecursive,
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(0.55, 0.25, 0.75, 0.5),
                    custom_size: Some(box_size),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::ZERO),
                visibility: Visibility::Hidden,
                ..default()
            },
        ))
        .with_children(|p| {
            let pos = Vec3::new(0., window.height() / 8. + box_size.y / 2., 0.);

            top_text_entity = p
                .spawn(Text2dBundle {
                    text: Text::from_section("ATrivialSolution Presents", text_style.clone())
                        .with_justify(JustifyText::Center),
                    transform: Transform::from_translation(pos),
                    text_2d_bounds: Text2dBounds { size: box_size },
                    visibility: Visibility::Inherited,
                    ..default()
                })
                .id();

            bottom_text_entity = p
                .spawn(Text2dBundle {
                    text: Text::from_section("Project: Unnamed SurvivalRPG", text_style)
                        .with_justify(JustifyText::Center),
                    transform: Transform::from_translation(-pos),
                    text_2d_bounds: Text2dBounds { size: box_size },
                    visibility: Visibility::Inherited,
                    ..default()
                })
                .id();
        })
        .id();

    let mut screens = VecDeque::new();

    screens.push_back(Sequence::new(vec![
        TweenKind::TextColor(TweenInfo::new(
            top_text_entity,
            Some(Tween::new(
                EaseFunction::SineIn,
                Duration::from_secs_f32(4.),
                TextColorLens {
                    start: Color::rgba(0., 0., 0., 0.),
                    end: Color::rgba(0.4, 0.2, 0.45, 1.),
                    section: 0,
                },
            )),
        )),
        TweenKind::TextColor(TweenInfo::new(
            bottom_text_entity,
            Some(Tween::new(
                EaseFunction::SineIn,
                Duration::from_secs_f32(4.),
                TextColorLens {
                    start: Color::rgba(0.0, 0.0, 0.0, 0.0),
                    end: Color::rgba(0.4, 0.2, 0.45, 1.0),
                    section: 0,
                },
            )),
        )),
        TweenKind::SpriteColor(TweenInfo::new(
            credits_entity,
            Some(Tween::new(
                EaseFunction::SineIn,
                Duration::from_secs_f32(8.),
                SpriteColorLens {
                    start: Color::rgba(0.0, 0.0, 0.0, 0.0),
                    end: Color::rgba(0.6, 0.2, 0.7, 1.0),
                },
            )),
        )),
        TweenKind::Translation(TweenInfo::new(
            credits_entity,
            Some(Tween::new(
                EaseFunction::CircularIn,
                Duration::from_secs_f32(4.),
                TransformPositionLens {
                    start: Vec3::new(0., -400., 0.),
                    end: Vec3::new(0., 0., 0.),
                },
            )),
        )),
    ]));

    screens.push_back(Sequence::new(vec![TweenKind::Translation(TweenInfo::new(
        bevy_logo_entity,
        Some(Tween::new(
            EaseFunction::CircularInOut,
            Duration::from_secs_f32(4.),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::Y * 50.,
            },
        )),
    ))]));

    commands.spawn((
        SplashScreenCleanup,
        CleanupStrategy::DespawnRecursive,
        SplashScreen::new(screens),
    ));
}

fn update_splash_screen(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    input: Res<ButtonInput<KeyCode>>,
    mut screen_q: Query<&mut SplashScreen>,
    mut tween_q: Query<(
        &mut Visibility,
        Option<&Animator<Transform>>,
        Option<&Animator<Sprite>>,
        Option<&Animator<Text>>,
    )>,
) {
    let mut screen = screen_q.single_mut();
    if input.just_pressed(KeyCode::Escape) {
        screen.sequences.clear();
    }

    if screen.is_complete() {
        println!("sequences complete");
        state.set(AppState::MenuLoad);
        return;
    }

    screen.update(&mut commands, &mut tween_q);
    //println!("primary {:?}", screen.sequences);
}
