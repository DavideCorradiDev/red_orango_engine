use super::{AssetLoadError, AssetLoader};

use std::{collections::HashMap, path::PathBuf};

pub struct AssetManager<T, L: AssetLoader<T>> {
    asset_loader: L,
    assets_path: PathBuf,
    assets: HashMap<String, T>,
}

impl<T, L: AssetLoader<T>> AssetManager<T, L> {
    pub fn new(asset_loader: L, assets_path: PathBuf) -> Self {
        Self {
            asset_loader,
            assets_path,
            assets: HashMap::new(),
        }
    }

    pub fn get_asset_path(&self, asset_id: &str) -> PathBuf {
        let mut asset_path = self.assets_path.clone();
        asset_path.push(asset_id);
        asset_path
    }

    pub fn get(&self, asset_id: &str) -> Option<&T> {
        self.assets.get(asset_id)
    }

    pub fn load(&mut self, asset_id: &str) -> Result<Option<T>, AssetLoadError> {
        let asset = self.asset_loader.load(&self.get_asset_path(asset_id))?;
        Ok(self.assets.insert(asset_id.to_owned(), asset))
    }

    pub fn get_or_load(&mut self, asset_id: &str) -> Result<&T, AssetLoadError> {
        if let None = self.get(asset_id) {
            self.load(asset_id)?;
        }
        Ok(self.get(asset_id).unwrap())
    }

    pub fn remove(&mut self, asset_id: &str) -> Option<T> {
        self.assets.remove(asset_id)
    }

    pub fn clear(&mut self) {
        self.assets.clear()
    }
}
