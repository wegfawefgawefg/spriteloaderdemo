use glam::Vec2;
use rand::Rng;
use raylib::{ffi::MouseButton, RaylibHandle};

use crate::{
    audio::{Audio, SoundEffect},
    entity::{Entity, EntityType},
    graphics::Graphics,
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

pub fn do_touch_apple(state: &mut State, audio: &mut Audio) {
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
            // play sound
            audio.play_sound_effect(SoundEffect::UiConfirm);

            // find the latest man entity
            let man_entity: Option<usize> = state
                .entities
                .iter()
                .enumerate()
                .filter(|(_, entity)| entity.entity_type == EntityType::Man)
                .map(|(i, _)| i)
                .last();

            let pos = if let Some(man_entity) = man_entity {
                state.entities[man_entity].position
            } else {
                Vec2::new(SCREEN_DIMS.x as f32 / 2.0, SCREEN_DIMS.y as f32 / 2.0)
            };

            let rng = &mut rand::thread_rng();
            let max_scale = 10.0;
            let scale = rng.gen_range(4.0..max_scale);
            let base_size = Vec2::new(2.0, 4.0);
            state.add_entity(Entity {
                entity_type: EntityType::Man,
                position: pos,
                velocity: Vec2::new(0.0, 0.0),
                size: base_size * scale,
                sprite_animator: SpriteAnimator {
                    sprite: Sprite::ManIdle,
                    current_frame: 0,
                    current_time: 0.0,
                    scale,
                },
                follows: man_entity,
                hp: 10.0,
                active: true,
                friction: None,
                expire_in: None,
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
pub fn determine_reticle_sprite(rl: &mut RaylibHandle, state: &mut State, audio: &mut Audio) {
    let reticle = &state.entities[0];
    let reticle_bounds = reticle.get_bounds();
    let mut on_tree = false;
    let mut trees: Vec<usize> = vec![];
    for (i, entity) in state.entities.iter().enumerate() {
        if entity.entity_type != EntityType::Tree {
            continue;
        }
        let tree_bounds = entity.get_bounds();

        if reticle_bounds.intersects(&tree_bounds) {
            on_tree = true;
            trees.push(i);
        }
    }

    let reticle = &mut state.entities[0];
    let clicking = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);

    if on_tree {
        if clicking {
            reticle.sprite_animator.set_sprite(Sprite::AxeCutting);

            // if frame 3, hurt tree and play chop sound
            if reticle.sprite_animator.current_frame == 2 {
                if state.chop_cooldown > 0.0 {
                    return;
                }
                state.chop_cooldown = State::CHOP_COOLDOWN;
                audio.play_sound_effect(SoundEffect::BaseballBatSwing);

                for tree in trees {
                    if state.entities[tree].hp > 0.0 {
                        state.entities[tree].hp -= 1.0;
                    }

                    // spawn a 2-3 log entities at the position of the tree
                    let tree_position = state.entities[tree].position;
                    let mut rng = rand::thread_rng();
                    let num = rng.gen_range(2..=3);
                    let x_vel_max = 30;
                    let y_vel_max = 5;
                    for _ in 0..num {
                        let vel = Vec2::new(
                            rng.gen_range(-x_vel_max..x_vel_max) as f32,
                            rng.gen_range(-y_vel_max..y_vel_max) as f32,
                        );
                        let new_log = Entity {
                            entity_type: EntityType::Log,
                            position: tree_position,
                            velocity: vel,
                            size: Vec2::new(16.0, 16.0),
                            sprite_animator: SpriteAnimator {
                                sprite: Sprite::Log,
                                current_frame: 0,
                                current_time: 0.0,
                                scale: 6.0,
                            },
                            follows: None,
                            hp: 10.0,
                            expire_in: Some(5.0),
                            friction: Some(0.5),
                            active: true,
                        };

                        state.add_entity(new_log);
                    }
                }
            }
        } else {
            reticle.sprite_animator.set_sprite(Sprite::AxeIdle);
        }
    } else {
        // default reticle sprite
        reticle.sprite_animator.set_sprite(Sprite::Reticle);
    }

    if state.chop_cooldown > 0.0 {
        state.chop_cooldown -= rl.get_frame_time();
    }
}

pub fn become_chopped_if_dead_tree(state: &mut State) {
    for entity in state.entities.iter_mut() {
        if entity.entity_type != EntityType::Tree {
            continue;
        }
        if entity.hp <= 0.0 {
            entity.sprite_animator.set_sprite(Sprite::TreeStump);
        }
    }
}

pub fn apply_friction(state: &mut State, dt: f32) {
    for entity in state.entities.iter_mut() {
        if let Some(friction) = entity.friction {
            entity.velocity *= 1.0 - friction * dt;

            // if vel under 1, stop
            if entity.velocity.length() < 1.0 {
                entity.velocity = Vec2::ZERO;
            }
        }
    }
}

pub fn step_expiring_entities(state: &mut State, dt: f32) {
    for entity in state.entities.iter_mut() {
        // guard clause: skips entities that don't have an expire_in value
        if entity.expire_in.is_none() {
            continue;
        }

        if let Some(expire_in) = entity.expire_in {
            entity.expire_in = Some(expire_in - dt);
            if expire_in <= 0.0 {
                entity.active = false;
                entity.expire_in = None;
            }
        }
    }
}

pub fn prune_inactive_entities(state: &mut State) {
    state.entities.retain(|entity| entity.active);
}

pub fn do_following(state: &mut State) {
    let follow_dist = 10.0;
    let vel = 1000.0;
    for i in 1..state.entities.len() {
        if let Some(follows) = state.entities[i].follows {
            if follows >= state.entities.len() {
                state.entities[i].follows = None;
                continue;
            }

            let target = state.entities[follows].position;

            // seperate axis logic
            // let x_diff = target.x - state.entities[i].position.x;
            // let y_diff = target.y - state.entities[i].position.y;
            // if x_diff > dist {
            //     state.entities[i].velocity.x = vel;
            // } else if x_diff < -dist {
            //     state.entities[i].velocity.x = -vel;
            // } else {
            //     state.entities[i].velocity.x = 0.0;
            // }
            // if y_diff > dist {
            //     state.entities[i].velocity.y = vel;
            // } else if y_diff < -dist {
            //     state.entities[i].velocity.y = -vel;
            // } else {
            //     state.entities[i].velocity.y = 0.0;
            // }

            // unified axis logic
            let dir = target - state.entities[i].position;
            if dir.length() > follow_dist {
                state.entities[i].velocity = dir.normalize() * vel;
            } else {
                state.entities[i].velocity = Vec2::ZERO;
            }
        }
    }
}

pub fn step(
    rl: &mut RaylibHandle,
    state: &mut State,
    audio: &mut Audio,
    graphics: &mut Graphics,
    dt: f32,
) {
    apply_friction(state, dt);
    step_positions(state, dt);
    wrap_around_screen(state);
    set_man_sprite_based_on_velocity(state);
    step_sprites(state, &graphics.sprites, dt);
    do_touch_apple(state, audio);
    do_following(state);
    determine_reticle_sprite(rl, state, audio);
    become_chopped_if_dead_tree(state);

    step_expiring_entities(state, dt);
    prune_inactive_entities(state);
}
