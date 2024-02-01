use super::{
    modifier::{Affix, ModifierId, ModifierKind},
    StatId,
};
use crate::value::Value;

use serde_derive::{Deserialize as De, Serialize as Ser};

use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Ser, De, Debug, Hash, PartialEq, Eq)]
pub struct ModifierDescriptor {
    pub id: ModifierId,
    pub name: Cow<'static, str>,
    pub stat_id: StatId,
    pub kind: ModifierKind,
    pub affix: Affix,
    pub min: Value,
    pub max: Value,
}

#[derive(Ser, De, Debug)]
pub struct ModifierPool {
    pub resources: HashMap<ModifierId, ModifierDescriptor>,
    pub gems: HashMap<ModifierId, ModifierDescriptor>,
}
