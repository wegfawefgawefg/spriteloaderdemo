use glam::Vec2;
use rand::Rng;

use crate::{
    entity::{Entity, EntityType},
    settings::SCREEN_DIMS,
    sprite::{Sprite, SpriteAnimator, SpriteData},
};

pub struct State {
    pub entities: Vec<Entity>,
    pub chop_cooldown: f32,
}

impl State {
    pub const CHOP_COOLDOWN: f32 = 0.2;
    pub fn new() -> Self {
        Self {
            entities: vec![],
            chop_cooldown: 0.0,
        }
    }

    pub fn add_entity(&mut self, entity: Entity) -> usize {
        // println!("Added entity: {:?}", entity);
        self.entities.push(entity);
        self.entities.len() - 1
    }

    pub fn get_entity(&self, id: usize) -> Option<&Entity> {
        self.entities.get(id)
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
