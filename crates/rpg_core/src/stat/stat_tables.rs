use super::{
    stat::{StatDescriptor, StatId},
    value::Value,
};

use std::borrow::Cow;
use std::collections::HashMap;

use serde_derive::Deserialize as De;

#[derive(De)]
pub struct StatTable {
    pub stats: HashMap<Cow<'static, str>, StatDescriptor>,
    pub base_stats: Vec<Cow<'static, str>>,
    pub vital_stats: Vec<Cow<'static, str>>,
    pub base_stat_defaults: HashMap<Cow<'static, str>, Value>,
    pub class_str: HashMap<Cow<'static, str>, Value>,
    pub class_dex: HashMap<Cow<'static, str>, Value>,
    pub class_int: HashMap<Cow<'static, str>, Value>,
    pub class_str_dex: HashMap<Cow<'static, str>, Value>,
    pub class_dex_int: HashMap<Cow<'static, str>, Value>,
    pub class_int_str: HashMap<Cow<'static, str>, Value>,
    pub class_str_dex_int: HashMap<Cow<'static, str>, Value>,
}
