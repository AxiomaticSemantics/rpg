use bevy::{reflect::TypePath, utils::BoxedFuture, asset::{AsyncReadExt, io::Reader, LoadContext, Asset, AssetLoader}};
use thiserror::Error;

use std::marker::PhantomData;

#[derive(Default)]
pub struct JsonAssetLoader<A> {
    extensions: [&'static str; 1],
    _marker: PhantomData<A>,
}

impl<A> JsonAssetLoader<A> {
    pub fn new(name: &'static str) -> Self {
        Self {
           extensions: [ name ],
           _marker: PhantomData,
        }
    }
}

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum JsonAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
    // /// A [RON](ron) Error
    //#[error("Could not parse RON: {0}")]
    //RonSpannedError(#[from] ron::error::SpannedError),
}

#[derive(Asset, Default, Debug, TypePath)]
pub struct JsonSource(pub Vec<u8>);

impl<A> AssetLoader for JsonAssetLoader<A>
where
    A: Asset
    //for<'de> A: De<'de> + Asset,
{
    type Asset = JsonSource;
    type Settings = ();
    type Error = JsonAssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            //println!("{bytes:?}");
            //let mut asset: JsonSource = serde_json::from_slice(&bytes).expect("unable to decode asset");

            //load_context.set_default_asset(LoadedAsset::new(JsonSource(bytes.to_vec())));
            let asset: JsonSource = JsonSource(bytes);

            Ok(asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["json"] //self.extensions
    }
}
