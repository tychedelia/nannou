use bevy::asset::io::Reader;
use bevy::asset::{ron, AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use bevy::utils::{BoxedFuture, HashMap};

pub struct ScriptAssetPlugin;

impl Plugin for ScriptAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Script>()
            .init_asset_loader::<ScriptLoader>();
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct Script {
    pub(crate) code: String,
}

#[derive(Default)]
struct ScriptLoader;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ScriptLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// An [std::string::FromUtf8Error]
    #[error("Could not convert bytes to string: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
}

impl AssetLoader for ScriptLoader {
    type Asset = Script;
    type Settings = ();
    type Error = ScriptLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let code = String::from_utf8(bytes)?;
            Ok(Script { code })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["js"]
    }
}
