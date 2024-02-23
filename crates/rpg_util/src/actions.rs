use crate::unit::Unit;

use rpg_core::skill::{SkillId, SkillTarget};

use bevy::{
    ecs::{
        component::Component,
        system::{Query, Res},
    },
    math::Vec3,
    time::{Time, Timer},
};

use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct AttackData {
    pub skill_id: SkillId,
    pub user: Vec3,
    pub skill_target: SkillTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KnockbackData {
    pub direction: Vec3,
    pub start: f32,
    pub duration: f32,
    pub speed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionKind {
    Move,
    Look,
    Knockback,
    Attack,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionData {
    Move(Vec3),
    LookDir(Vec3),
    LookPoint(Vec3),
    Knockback(KnockbackData),
    Attack(AttackData),
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub enum State {
    #[default]
    Pending,
    Active,
    Completed,
}

#[derive(Clone, Debug)]
pub struct Action {
    pub state: State,
    pub kind: ActionKind,
    pub data: ActionData,
    pub interruptible: bool,
    pub timer: Option<Timer>,
}

impl Action {
    pub fn new(data: ActionData, timer: Option<Timer>, interruptible: bool) -> Self {
        let kind = match data {
            ActionData::Move(_) => ActionKind::Move,
            ActionData::LookDir(_) | ActionData::LookPoint(_) => ActionKind::Look,
            ActionData::Attack(_) => ActionKind::Attack,
            ActionData::Knockback(_) => ActionKind::Knockback,
        };

        Self {
            data,
            timer,
            state: State::Pending,
            kind,
            interruptible,
        }
    }

    pub fn is_interruptible(&self) -> bool {
        self.interruptible
    }

    pub fn is_completed(&self) -> bool {
        self.state == State::Completed
    }

    pub fn update(&mut self, dt: Duration) {
        if let Some(timer) = &mut self.timer {
            timer.tick(dt);
            if timer.finished() {
                // TODO advance to the next state
                self.timer = None;
                match self.state {
                    State::Pending => self.state = State::Active,
                    State::Active => self.state = State::Completed,
                    _ => {
                        panic!("unexpected action timer")
                    }
                }
            }
        }

        // debug!("remaining: {:?}", self.timer);
    }
}

#[derive(Default, Debug, Component)]
pub struct UnitActions {
    movement: Option<Action>,
    look: Option<Action>,
    knockback: Option<Action>,
    attack: Option<Action>,
}

impl UnitActions {
    pub fn is_set(&self, kind: ActionKind) -> bool {
        match kind {
            ActionKind::Look => self.look.is_some(),
            ActionKind::Move => self.movement.is_some(),
            ActionKind::Knockback => self.knockback.is_some(),
            ActionKind::Attack => self.attack.is_some(),
        }
    }

    pub fn get(&self, kind: ActionKind) -> Option<&Action> {
        match kind {
            ActionKind::Move => self.movement.as_ref(),
            ActionKind::Look => self.look.as_ref(),
            ActionKind::Knockback => self.knockback.as_ref(),
            ActionKind::Attack => self.attack.as_ref(),
        }
    }

    pub fn get_mut(&mut self, kind: ActionKind) -> Option<&mut Action> {
        match kind {
            ActionKind::Move => self.movement.as_mut(),
            ActionKind::Look => self.look.as_mut(),
            ActionKind::Knockback => self.knockback.as_mut(),
            ActionKind::Attack => self.attack.as_mut(),
        }
    }

    pub fn set(&mut self, action: Action) {
        match action.kind {
            ActionKind::Look => self.look = Some(action),
            ActionKind::Move => self.movement = Some(action),
            ActionKind::Knockback => self.knockback = Some(action),
            ActionKind::Attack => self.attack = Some(action),
        }
    }

    pub fn request(&mut self, action: Action) -> bool {
        if self.knockback.is_some() || self.attack.is_some() {
            // debug!("action blocked");
            return false;
        }

        match action.kind {
            ActionKind::Look => {
                self.look = Some(action);
                true
            }
            ActionKind::Move => {
                if self.movement.is_none() {
                    self.movement = Some(action);
                    true
                } else {
                    false
                }
            }
            // Knockback may be overridden
            ActionKind::Knockback => {
                self.knockback = Some(action);
                true
            }
            ActionKind::Attack => {
                if self.attack.is_none() {
                    self.attack = Some(action);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn is_inactive(&self) -> bool {
        self.movement.is_none()
            && self.look.is_none()
            && self.attack.is_none()
            && self.knockback.is_none()
    }

    pub fn clear_completed(&mut self) {
        if let Some(action) = &mut self.look {
            if action.is_completed() {
                self.look = None;
            }
        }

        if let Some(action) = &mut self.movement {
            if action.is_completed() {
                self.movement = None;
            }
        }

        if let Some(action) = &mut self.knockback {
            if action.is_completed() {
                self.knockback = None;
            }
        }

        if let Some(action) = &mut self.attack {
            if action.is_completed() {
                self.attack = None;
            }
        }
    }

    pub fn reset(&mut self) {
        // debug!("resetting actions");

        self.look = None;
        self.movement = None;
        self.knockback = None;
        self.attack = None;
    }

    pub fn update(&mut self, dt: Duration) {
        if let Some(action) = &mut self.look {
            action.update(dt);
        } else if let Some(action) = &mut self.movement {
            action.update(dt);
        } else if let Some(action) = &mut self.knockback {
            action.update(dt);
        } else if let Some(action) = &mut self.attack {
            action.update(dt);
        }
    }
}

pub fn action_tick(time: Res<Time>, mut unit_q: Query<&mut UnitActions>) {
    for mut actions in &mut unit_q {
        if actions.is_inactive() {
            continue;
        }

        actions.update(time.delta());

        actions.clear_completed();
    }
}
