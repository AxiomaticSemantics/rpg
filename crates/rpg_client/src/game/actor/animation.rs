use super::ActorKey;
use crate::game::assets::RenderResources;

use bevy::{
    animation::{AnimationPlayer, RepeatAnimation},
    ecs::{
        component::Component,
        entity::Entity,
        query::Changed,
        system::{Query, Res},
    },
    hierarchy::{Children, HierarchyQueryExt},
    log::warn,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) struct AnimationIndex(pub(crate) usize);

pub(crate) const ANIM_ATTACK_IDX: AnimationIndex = AnimationIndex(0);
pub(crate) const ANIM_DEATH_IDX: AnimationIndex = AnimationIndex(1);
pub(crate) const ANIM_DEFEND_IDX: AnimationIndex = AnimationIndex(2);
pub(crate) const ANIM_WALK_IDX: AnimationIndex = AnimationIndex(3);

pub(crate) const ANIM_WALK: AnimationState =
    AnimationState::new(RepeatAnimation::Forever, false, ANIM_WALK_IDX);
pub(crate) const ANIM_IDLE: AnimationState =
    AnimationState::new(RepeatAnimation::Forever, true, ANIM_WALK_IDX);
pub(crate) const ANIM_ATTACK: AnimationState =
    AnimationState::new(RepeatAnimation::Never, false, ANIM_ATTACK_IDX);
pub(crate) const ANIM_DEFEND: AnimationState =
    AnimationState::new(RepeatAnimation::Never, false, ANIM_DEFEND_IDX);
pub(crate) const ANIM_DEATH: AnimationState =
    AnimationState::new(RepeatAnimation::Never, false, ANIM_DEATH_IDX);

#[derive(Component, Debug, PartialEq, Clone)]
pub struct AnimationState {
    pub repeat: RepeatAnimation,
    pub paused: bool,
    pub index: AnimationIndex,
    pub player: Entity,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            repeat: RepeatAnimation::Forever,
            paused: true,
            index: ANIM_WALK_IDX,
            player: Entity::PLACEHOLDER,
        }
    }
}

impl AnimationState {
    pub const fn new(repeat: RepeatAnimation, paused: bool, index: AnimationIndex) -> Self {
        Self {
            repeat,
            paused,
            index,
            player: Entity::PLACEHOLDER,
        }
    }

    pub fn set_state(&mut self, state: AnimationState) -> bool {
        if state != *self {
            *self = state;
            true
        } else {
            false
        }
    }
}

pub fn animator(
    renderables: Res<RenderResources>,
    mut actor_q: Query<(Entity, &mut AnimationState, &ActorKey), Changed<AnimationState>>,
    child_q: Query<&Children>,
    mut anim_q: Query<(Entity, &mut AnimationPlayer)>,
) {
    for (entity, mut anim, actor_key) in &mut actor_q {
        let clip_handles = &renderables.actors[actor_key.0].animations;

        if anim.player == Entity::PLACEHOLDER {
            for child in child_q.iter_descendants(entity) {
                let Ok((player_entity, _)) = anim_q.get(child) else {
                    continue;
                };

                anim.player = player_entity;

                break;
            }
        }

        if anim.player == Entity::PLACEHOLDER {
            warn!("unable to find animation player for actor");
            continue;
        }

        let (_, mut player) = anim_q.get_mut(anim.player).unwrap();

        if anim.paused {
            player.pause();
        } else {
            player.resume();
        }

        player.set_repeat(anim.repeat);

        if !anim.paused {
            if anim.repeat == RepeatAnimation::Never {
                player.start(clip_handles[anim.index.0].clone());
            } else {
                player.play(clip_handles[anim.index.0].clone());
            }
        } else {
            player.seek_to(0.);
        }
        // debug!("player {} {} {}", curr_clip.duration(), player.is_paused(), player.elapsed());
    }
}
