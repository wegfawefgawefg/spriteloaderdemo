use std::path::Path;

use raylib::{texture::Texture2D, RaylibHandle, RaylibThread};
use strum::{EnumCount, IntoEnumIterator};

use crate::sprite::{load_sprites, Sprite, SpriteData};

fn load_textures(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    asset_folder: &str,
) -> Result<Vec<Texture2D>, String> {
    let mut textures = Vec::with_capacity(Sprite::COUNT);
    for sprite in Sprite::iter() {
        let filename = sprite.to_filename();
        let png_path = Path::new(asset_folder).join(format!("{}.png", filename));
        let texture = rl
            .load_texture(thread, png_path.to_str().unwrap())
            .map_err(|e| format!("Failed to load texture {}: {}", filename, e))?;
        textures.push(texture);
    }
    Ok(textures)
}

pub struct Graphics {
    pub sprites: Vec<SpriteData>,
    pub textures: Vec<Texture2D>,
}

impl Graphics {
    pub fn new(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        asset_folder: &str,
    ) -> Result<Self, String> {
        let sprites = load_sprites(asset_folder)?;
        let textures = load_textures(rl, thread, asset_folder)?;
        Ok(Self { sprites, textures })
    }

    pub fn reload(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        asset_folder: &str,
    ) -> Result<(), String> {
        self.sprites = load_sprites(asset_folder)?;
        // The old textures will be automatically unloaded when replaced
        self.textures = load_textures(rl, thread, asset_folder)?;
        Ok(())
    }

    pub fn get_sprite_texture(&self, sprite: Sprite) -> &Texture2D {
        &self.textures[sprite as usize]
    }

    pub fn get_sprite_data(&self, sprite: Sprite) -> &SpriteData {
        &self.sprites[sprite as usize]
    }
}
