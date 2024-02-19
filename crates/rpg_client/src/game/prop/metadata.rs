use bevy::math::Vec3;

use std::borrow::Cow;
use std::collections::HashMap;

use serde_derive::Deserialize as De;

#[derive(De)]
pub(crate) enum PropAssetKind {
    GltfMesh,
    GltfScene,
    GlbMesh,
    GlbScene,
    ShapeMesh,
}

#[derive(De)]
pub(crate) struct PropDescriptor {
    pub(crate) key: Cow<'static, str>,
    pub(crate) kind: PropAssetKind,
    pub(crate) offset: Option<Vec3>,
    pub(crate) direction: Option<Vec3>,
}

#[derive(De)]
pub(crate) struct PropTable {
    pub(crate) prop: HashMap<Cow<'static, str>, PropDescriptor>,
}

pub(crate) struct PropMetadata {
    pub(crate) prop: PropTable,
}
