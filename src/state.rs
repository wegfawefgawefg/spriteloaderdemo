use glam::Vec2;
use rand::Rng;

use crate::{
    entity::{Entity, EntityType},
    settings::SCREEN_DIMS,
    sprite::{Sprite, SpriteAnimator, SpriteData},
};

pub struct State {
    pub entities: Vec<Entity>,
}

impl State {
    pub fn new() -> Self {
        Self { entities: vec![] }
    }

    pub fn add_entity(&mut self, entity: Entity) -> usize {
        // println!("Added entity: {:?}", entity);
        self.entities.push(entity);
        self.entities.len() - 1
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
