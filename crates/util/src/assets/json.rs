use bevy::{reflect::TypePath, utils::BoxedFuture, asset::{AsyncReadExt, io::Reader, LoadContext, Asset, AssetLoader}};
use thiserror::Error;

use std::marker::PhantomData;

#[derive(Default)]
pub struct JsonAssetLoader<A> {
    _marker: PhantomData<A>,
}

impl<A> JsonAssetLoader<A> {
    pub fn new() -> Self {
        Self {
           _marker: PhantomData,
        }
    }
}

/// [`JsonAssetLoader`] error types
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum JsonAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load json: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Asset, Default, Debug, TypePath)]
pub struct JsonSource(pub Vec<u8>);

impl<A> AssetLoader for JsonAssetLoader<A>
where
    A: Asset
{
    type Asset = JsonSource;
    type Settings = ();
    type Error = JsonAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            
            Ok(JsonSource(bytes))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}
