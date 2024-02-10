use bevy::ecs::{component::Component, entity::Entity, system::Resource};

use std::collections::VecDeque;

#[derive(Debug, Copy, Clone, Default, PartialEq, Component)]
pub enum HistoryIndex {
    #[default]
    None,
    Some(usize),
}

impl HistoryIndex {
    pub fn increment(&mut self, max: Option<Self>) {
        match self {
            Self::None => match max {
                Some(Self::None) | None => *self = Self::Some(0),
                Some(Self::Some(_)) => *self = Self::Some(0),
            },
            Self::Some(ref self_index) => match max {
                Some(Self::None) | None => *self = Self::Some(0),
                Some(Self::Some(ref max_index)) => {
                    if *self_index < *max_index {
                        *self = Self::Some(*self_index + 1);
                    }
                }
            },
        }
    }

    pub fn decrement(&mut self) {
        match self {
            Self::None => {}
            Self::Some(ref mut index) => {
                if *index == 0 {
                    *self = Self::None;
                } else {
                    *index -= 1;
                }
            }
        }
    }

    pub fn gt(&self, other: Self) -> bool {
        match self {
            Self::Some(index) => match other {
                Self::None => false,
                Self::Some(other_index) => *index > other_index,
            },
            Self::None => false,
        }
    }

    pub fn gte(&self, other: Self) -> bool {
        self.gt(other) || self.eq(other)
    }

    pub fn eq(&self, other: Self) -> bool {
        *self == other
    }

    //pub fn in_range(&self) -> bool {}
}

#[derive(Debug, Default)]
pub struct HistoryItem {
    pub item: String,
    pub is_cmd: bool,
    pub index: HistoryIndex,
}

impl HistoryItem {
    pub fn new(item: String, is_cmd: bool) -> Self {
        Self {
            item,
            is_cmd,
            index: HistoryIndex::None,
        }
    }
}

#[derive(Debug, Default)]
pub struct History {
    pub index: HistoryIndex,
    pub max: HistoryIndex,
    pub history: VecDeque<HistoryItem>,
}

impl History {
    pub fn increment(&mut self) {
        self.index.increment(Some(self.max));
    }

    pub fn decrement(&mut self) {
        self.index.decrement();
    }
}

#[derive(Resource, Debug)]
pub struct Console {
    pub ui_root: Entity,
    pub input: Entity,
    pub history: History,
    pub command_history: History,
}

impl Console {
    pub fn new(ui_root: Entity, input: Entity) -> Self {
        Self {
            ui_root,
            input,
            history: History::default(),
            command_history: History::default(),
        }
    }

    pub fn update_history(&mut self, item: String, is_command: bool) {
        // debug!("pushing history item {item:?}");

        if is_command {
            let (_, suffix) = item.split_at(2);
            self.command_history
                .history
                .push_front(HistoryItem::new(suffix.into(), true));

            self.command_history.increment();
            self.command_history.max = HistoryIndex::Some(self.command_history.history.len());
            self.command_history.index = HistoryIndex::None;
        }

        self.history
            .history
            .push_front(HistoryItem::new(item, false));

        self.history.increment();
        self.history.max = HistoryIndex::Some(self.history.history.len());
        self.history.index = HistoryIndex::None;

        assert!(if self.history.max == HistoryIndex::None {
            self.history.history.is_empty()
        } else {
            true
        });
    }
}
