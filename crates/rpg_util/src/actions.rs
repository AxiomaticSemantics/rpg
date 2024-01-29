use crate::unit::Unit;

use rpg_core::skill::SkillId;

use bevy::{
    ecs::{
        component::Component,
        system::{Query, Res},
    },
    math::Vec3,
    time::{Time, Timer},
};

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttackData {
    pub skill_id: SkillId,
    pub user: Vec3,
    pub origin: Vec3,
    pub target: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KnockbackData {
    pub direction: Vec3,
    pub start: f32,
    pub duration: f32,
    pub speed: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Move = 0x0000_0001,
    Look = 0x0000_0002,
    Knockback = 0x0000_0008,
    Attack = 0x0000_00010,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    Timer,
    Active,
    Finalize,
    Completed,
}

#[derive(Clone, Debug)]
pub struct Action {
    pub state: State,
    pub kind: Kind,
    pub data: ActionData,
    pub interruptible: bool,
    pub timer: Option<Timer>,
}

impl Action {
    pub fn new(data: ActionData, timer: Option<Timer>, interruptible: bool) -> Self {
        let kind = match data {
            ActionData::Move(_) => Kind::Move,
            ActionData::LookDir(_) | ActionData::LookPoint(_) => Kind::Look,
            ActionData::Attack(_) => Kind::Attack,
            ActionData::Knockback(_) => Kind::Knockback,
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
        if self.state != State::Timer {
            return;
        }

        if let Some(timer) = &mut self.timer {
            timer.tick(dt);
            if timer.just_finished() {
                self.state = State::Active;
            }
        } else {
            self.state = State::Completed;
        }

        //println!("remaining: {:?}", self.timer);
    }
}

#[derive(Default, Debug, Component)]
pub struct Actions {
    pub movement: Option<Action>,
    pub look: Option<Action>,
    pub knockback: Option<Action>,
    pub attack: Option<Action>,
}

impl Actions {
    pub fn is_set(&self, kind: Kind) -> bool {
        match kind {
            Kind::Look => self.look.is_some(),
            Kind::Move => self.movement.is_some(),
            Kind::Knockback => self.knockback.is_some(),
            Kind::Attack => self.attack.is_some(),
        }
    }

    pub fn set(&mut self, action: Action) {
        match action.kind {
            Kind::Look => self.look = Some(action),
            Kind::Move => self.movement = Some(action),
            Kind::Knockback => self.knockback = Some(action),
            Kind::Attack => self.attack = Some(action),
        }
    }

    pub fn request(&mut self, action: Action) {
        if self.knockback.is_some() {
            //println!("action blocked");
            return;
        }

        match action.kind {
            Kind::Look => self.look = Some(action),
            Kind::Move => self.movement = Some(action),
            Kind::Knockback => self.knockback = Some(action),
            Kind::Attack => {
                if self.attack.is_none() {
                    self.attack = Some(action);
                } else {
                    println!("attack in progress rejected");
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
        //println!("resetting actions");

        self.look = None;
        self.movement = None;
        self.knockback = None;
        self.attack = None;
    }
}

pub fn action_tick(time: Res<Time>, mut unit_q: Query<(&Unit, &mut Actions)>) {
    for (unit, mut actions) in &mut unit_q {
        if !unit.is_alive() && !actions.is_inactive() {
            actions.reset();
            continue;
        }

        if let Some(action) = &mut actions.attack {
            if action.state == State::Timer {
                action.update(time.delta());
            }
        }

        actions.clear_completed();
    }
}
