use std::borrow::Cow;
use std::collections::HashMap;

use serde_derive::Deserialize as De;

#[derive(De)]
pub(crate) struct PropDescriptor {
    pub key: Cow<'static, str>,
}

#[derive(Default, De)]
pub(crate) struct PropMetadata {
    pub prop: HashMap<Cow<'static, str>, PropDescriptor>,
}
