use glam::Vec2;
use rand::Rng;
use raylib::{ffi::MouseButton, RaylibHandle};

use crate::{
    entity::{Entity, EntityType},
    settings::SCREEN_DIMS,
    sprite::{Sprite, SpriteAnimator, SpriteData},
    state::State,
};

pub fn step_positions(state: &mut State, dt: f32) {
    for entity in &mut state.entities {
        entity.position += entity.velocity * dt;
    }
}

pub fn wrap_around_screen(state: &mut State) {
    for entity in &mut state.entities {
        if entity.position.x < 0.0 {
            entity.position.x += SCREEN_DIMS.x as f32;
        } else if entity.position.x >= SCREEN_DIMS.x as f32 {
            entity.position.x -= SCREEN_DIMS.x as f32;
        }
        if entity.position.y < 0.0 {
            entity.position.y += SCREEN_DIMS.y as f32;
        } else if entity.position.y >= SCREEN_DIMS.y as f32 {
            entity.position.y -= SCREEN_DIMS.y as f32;
        }
    }
}

pub fn set_man_sprite_based_on_velocity(state: &mut State) {
    // set sprite based on velocity
    const MIN_WALK_SPEED: f32 = 10.0;
    for entity in &mut state.entities {
        if entity.entity_type != EntityType::Man {
            continue;
        }
        let sprite = if entity.velocity.length() > MIN_WALK_SPEED {
            Sprite::ManWalk
        } else {
            Sprite::ManIdle
        };
        if entity.sprite_animator.sprite != sprite {
            entity.sprite_animator.set_sprite(sprite);
        }
    }
}

pub fn step_sprites(state: &mut State, sprites: &[SpriteData], dt: f32) {
    let dt_ms = dt * 1000.0;
    for entity in &mut state.entities {
        entity.sprite_animator.step(sprites, dt_ms);
    }
}

pub fn do_touch_apple(state: &mut State) {
    // if entity 02, touches the apple [entity 01], spawn a new man at the end of the list, make him follow the last entity in the list
    // consider the apple scale
    let apple = &state.entities[1];
    let apple_bounds = apple.get_bounds();
    for entity in &mut state.entities {
        if entity.entity_type != EntityType::Man {
            continue;
        }
        let man = entity;
        let man_bounds = man.get_bounds();

        if man_bounds.intersects(&apple_bounds) {
            let last_entity = state.entities.len() - 1;
            let last_entity_pos = state.entities[last_entity].position;
            state.add_entity(Entity {
                entity_type: EntityType::Man,
                position: last_entity_pos,
                velocity: Vec2::new(0.0, 0.0),
                size: Vec2::new(16.0, 24.0),
                sprite_animator: SpriteAnimator {
                    sprite: Sprite::ManIdle,
                    current_frame: 0,
                    current_time: 0.0,
                    scale: 6.0,
                },
                follows: Some(last_entity),
                hp: 10.0,
            });
            // move the apple to a new random position
            let rng = &mut rand::thread_rng();
            let new_pos = Vec2::new(
                rng.gen_range(0.0..SCREEN_DIMS.x as f32),
                rng.gen_range(0.0..SCREEN_DIMS.y as f32),
            );
            state.entities[1].position = new_pos;
            break;
        }
    }
}

/*
    determine reticle sprite
    // default reticle is reticle sprite
    // if reticle is on top of a tree, make it AxeIdle,
    // if currently clicking, make it AxeCutting,
*/
pub fn determine_reticle_sprite(rl: &mut RaylibHandle, state: &mut State) {
    let reticle = &state.entities[0];
    let reticle_bounds = reticle.get_bounds();
    let mut on_tree = false;
    for entity in &state.entities {
        if entity.entity_type != EntityType::Tree {
            continue;
        }
        let tree = entity;
        let tree_bounds = tree.get_bounds();

        if reticle_bounds.intersects(&tree_bounds) {
            on_tree = true;
            break;
        }
    }

    let reticle = &mut state.entities[0];
    let clicking = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);

    if on_tree {
        if clicking {
            reticle.sprite_animator.set_sprite(Sprite::AxeCutting);
        } else {
            reticle.sprite_animator.set_sprite(Sprite::AxeIdle);
        }
    } else {
        // default reticle sprite
        reticle.sprite_animator.set_sprite(Sprite::Reticle);
    }
}

pub fn step(rl: &mut RaylibHandle, state: &mut State, sprites: &[SpriteData], dt: f32) {
    step_positions(state, dt);
    wrap_around_screen(state);
    set_man_sprite_based_on_velocity(state);
    step_sprites(state, sprites, dt);
    do_touch_apple(state);
    determine_reticle_sprite(rl, state);
}
