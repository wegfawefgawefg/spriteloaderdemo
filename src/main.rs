use draw::draw_entities;
use entity::{Entity, EntityType};
use glam::{UVec2, Vec2};
use graphics::Graphics;
use rand::Rng;
use raylib::ffi::{SetTraceLogLevel, TraceLogLevel};
use raylib::prelude::*;
use serde_json::Value;
use settings::SCREEN_DIMS;
use sprite::{Sprite, SpriteAnimator};
use state::State;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::EnumIter;

pub mod draw;
pub mod entity;
pub mod graphics;
pub mod settings;
pub mod sprite;
pub mod state;
pub mod step;

fn main() -> Result<(), String> {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_DIMS.x as i32, SCREEN_DIMS.y as i32)
        .title("Auto-Managed Textures Sprite System")
        .build();

    // set window position
    rl.set_window_position(500, 500);
    // set fps to 144
    rl.set_target_fps(144);
    // hide mouse
    rl.hide_cursor();

    let asset_folder = "./assets";
    let mut graphics = Graphics::new(&mut rl, &thread, asset_folder)?;

    let mut state = State::new();

    // reticle
    let reticle_entity = Entity {
        entity_type: EntityType::Reticle,
        position: Vec2::new(50.0, 200.0),
        velocity: Vec2::new(0.0, 0.0),
        size: Vec2::new(40.0, 40.0),
        sprite_animator: SpriteAnimator {
            sprite: Sprite::Reticle,
            current_frame: 0,
            current_time: 0.0,
            scale: 5.0,
        },
        follows: None,
        hp: 10.0,
    };
    let reticle_id = state.add_entity(reticle_entity);

    // apple
    let rng = &mut rand::thread_rng();
    let apple_id = state.add_entity(Entity {
        entity_type: EntityType::Apple,
        position: Vec2::new(
            rng.gen_range(0.0..SCREEN_DIMS.x as f32),
            rng.gen_range(0.0..SCREEN_DIMS.y as f32),
        ),
        velocity: Vec2::new(0.0, 0.0),
        size: Vec2::new(48.0, 36.0),
        sprite_animator: SpriteAnimator {
            sprite: Sprite::Apple,
            current_frame: 0,
            current_time: 0.0,
            scale: 6.0,
        },
        follows: None,
        hp: 10.0,
    });

    // trees
    // in random positions, only 20
    const NUM_TREES: usize = 20;
    let rng = &mut rand::thread_rng();
    for _ in 0..NUM_TREES {
        // let scale = rng.gen_range(10.0..15.0);
        let scale = 10.0;
        let mut entity = Entity {
            entity_type: EntityType::Tree,
            position: Vec2::new(
                rng.gen_range(0.0..SCREEN_DIMS.x as f32),
                rng.gen_range(0.0..SCREEN_DIMS.y as f32),
            ),
            velocity: Vec2::new(0.0, 0.0),
            size: Vec2::new(2.0 * scale, 5.0 * scale),
            sprite_animator: SpriteAnimator {
                sprite: Sprite::Tree,
                current_frame: 0,
                current_time: 0.0,
                scale,
            },
            follows: None,
            hp: 10.0,
        };
        entity.sprite_animator.randomize_frame(&graphics.sprites);
        state.add_entity(entity);
    }

    // mans
    let mut last_man: Option<usize> = None;
    const NUM_MANS: usize = 1;
    for i in 0..NUM_MANS {
        println!("Adding");
        let follows = if i == 0 { Some(apple_id) } else { last_man };
        let last_entity_id = state.add_entity(Entity {
            entity_type: EntityType::Man,
            position: SCREEN_DIMS.as_vec2() / 2.0,
            velocity: Vec2::new(0.0, 0.0),
            size: Vec2::new(16.0, 24.0),
            sprite_animator: SpriteAnimator {
                sprite: Sprite::ManIdle,
                current_frame: 0,
                current_time: 0.0,
                scale: 6.0,
            },
            follows,
            hp: 10.0,
        });
        last_man = Some(last_entity_id);
    }

    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            match graphics.reload(&mut rl, &thread, asset_folder) {
                Ok(_) => println!("Reloaded assets"),
                Err(e) => println!("Failed to reload assets: {}", e),
            }
        }

        // arrow keys to move entity 0
        let vel = 1000.0;
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            state.entities[0].velocity.x = vel;
        } else if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            state.entities[0].velocity.x = -vel;
        } else {
            state.entities[0].velocity.x = 0.0;
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) {
            state.entities[0].velocity.y = vel;
        } else if rl.is_key_down(KeyboardKey::KEY_UP) {
            state.entities[0].velocity.y = -vel;
        } else {
            state.entities[0].velocity.y = 0.0;
        }

        // set entity 0 position to mouse
        state.entities[0].position = Vec2::new(rl.get_mouse_x() as f32, rl.get_mouse_y() as f32);

        // make entity 2 follow entity 1
        // let target = state.entities[0].position;
        // let dir = target - state.entities[1].position;
        // // dir can be zero if target is the same as position, then normalize fails
        // if dir.length() > 0.0 {
        //     state.entities[1].velocity = dir.normalize() * vel * 0.5;
        // } else {
        //     state.entities[1].velocity = Vec2::ZERO;
        // }

        // make each entity follow its follow entity if the distance is between them is greater than 2
        let dist = 8.0;
        for i in 1..state.entities.len() {
            if let Some(follows) = state.entities[i].follows {
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
                if dir.length() > dist {
                    state.entities[i].velocity = dir.normalize() * vel;
                } else {
                    state.entities[i].velocity = Vec2::ZERO;
                }
            }
        }

        let dt = rl.get_frame_time();
        step::step(&mut rl, &mut state, &graphics.sprites, dt);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(150, 150, 200, 255));

        let man_walk = graphics.get_sprite_data(Sprite::ManWalk);
        d.draw_text(
            &format!("Man Walk frames: {}", man_walk.frames.len()),
            10,
            40,
            20,
            Color::BLACK,
        );

        draw_entities(&mut d, &graphics, &state);
    }

    Ok(())
}
