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
};

#[derive(Component, Default, Debug, PartialEq, Copy, Clone)]
pub struct AnimationState {
    pub repeat: RepeatAnimation,
    pub paused: bool,
    pub index: usize,
}

impl AnimationState {
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
    actor_q: Query<(Entity, &AnimationState, &ActorKey), Changed<AnimationState>>,
    child_q: Query<&Children>,
    mut anim_q: Query<&mut AnimationPlayer>,
) {
    for (entity, anim, actor_key) in &actor_q {
        let clip_handles = &renderables.actors[actor_key.0].animations;

        for child in child_q.iter_descendants(entity) {
            let Ok(mut player) = anim_q.get_mut(child) else {
                continue;
            };

            if anim.paused {
                player.pause();
            } else {
                player.resume();
            }

            player.set_repeat(anim.repeat);

            //if curr.index != before.index {
            if !anim.paused {
                if anim.repeat == RepeatAnimation::Never {
                    player.start(clip_handles[anim.index].clone());
                } else {
                    player.play(clip_handles[anim.index].clone());
                }
            }
        }

        // println!("player {} {} {}", curr_clip.duration(), player.is_paused(), player.elapsed());
    }
}
