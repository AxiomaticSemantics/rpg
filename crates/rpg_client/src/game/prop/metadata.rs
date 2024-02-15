use std::borrow::Cow;
use std::collections::HashMap;

use serde_derive::Deserialize as De;

#[derive(De)]
pub(crate) struct PropDescriptor {
    pub(crate) key: Cow<'static, str>,
}

#[derive(De)]
pub(crate) struct PropTable {
    pub(crate) prop: HashMap<Cow<'static, str>, PropDescriptor>,
}

pub(crate) struct PropMetadata {
    pub(crate) prop: PropTable,
}
