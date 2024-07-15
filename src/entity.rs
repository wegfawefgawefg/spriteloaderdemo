use glam::Vec2;

use crate::sprite::{Sprite, SpriteAnimator};

#[derive(Debug, PartialEq)]
pub enum EntityType {
    Man,
    Tree,
    Reticle,
    Apple,
    Log,
}

#[derive(Debug)]
pub struct Entity {
    pub entity_type: EntityType,
    pub position: Vec2,
    pub size: Vec2,
    pub velocity: Vec2,
    pub sprite_animator: SpriteAnimator,
    pub follows: Option<usize>,
    pub hp: f32,
    pub friction: Option<f32>,
    pub expire_in: Option<f32>,
    pub active: bool,
}

pub struct Bounds {
    pub top_left: Vec2,
    pub bottom_right: Vec2,
}

impl Bounds {
    pub fn intersects(&self, other: &Bounds) -> bool {
        self.top_left.x < other.bottom_right.x
            && self.bottom_right.x > other.top_left.x
            && self.top_left.y < other.bottom_right.y
            && self.bottom_right.y > other.top_left.y
    }
}

impl Entity {
    pub fn new(entity_type: EntityType, position: Vec2, size: Vec2, sprite: Sprite) -> Self {
        Self {
            entity_type,
            position,
            size,
            velocity: Vec2::ZERO,
            sprite_animator: SpriteAnimator::new(sprite),
            follows: None,
            hp: 100.0,
            friction: None,
            expire_in: None,
            active: true,
        }
    }

    pub fn get_bounds(&self) -> Bounds {
        Bounds {
            top_left: Vec2::new(
                self.position.x - self.size.x / 2.0,
                self.position.y - self.size.y,
            ),
            bottom_right: Vec2::new(self.position.x + self.size.x / 2.0, self.position.y),
        }
    }
}
