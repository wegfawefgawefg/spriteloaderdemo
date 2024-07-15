use audio::{load_songs, load_sounds};
use raylib::{
    audio::{Music, RaylibAudio, Sound},
    color::Color,
    drawing::RaylibDraw,
    ffi::KeyboardKey,
};

use draw::draw_entities;
use entity::{Entity, EntityType};
use glam::Vec2;
use graphics::Graphics;
use rand::Rng;
use settings::SCREEN_DIMS;
use sprite::{Sprite, SpriteAnimator};
use state::State;

pub mod audio;
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

    rl.set_window_position(500, 500);
    rl.set_target_fps(144);
    rl.hide_cursor();

    let sprites_folder = "./assets/sprites";
    let mut graphics = Graphics::new(&mut rl, &thread, sprites_folder)?;
    let rl_audio_device = match RaylibAudio::init_audio_device() {
        Ok(rl_audio_device) => rl_audio_device,
        Err(e) => {
            println!("Error initializing audio device: {}", e);
            std::process::exit(1);
        }
    };
    let songs = audio::load_songs(&rl_audio_device);
    let sounds = audio::load_sounds(&rl_audio_device);
    let mut audio = audio::Audio::new(songs, sounds);
    let mut state = State::new();

    audio.play_song(audio::Song::Playing);

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
        active: true,
        friction: None,
        expire_in: None,
    };
    state.add_entity(reticle_entity);

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
        active: true,
        friction: None,
        expire_in: None,
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
            hp: 4.0,
            active: true,
            friction: None,
            expire_in: None,
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
            active: true,
            friction: None,
            expire_in: None,
        });
        last_man = Some(last_entity_id);
    }

    while !rl.window_should_close() {
        audio.update_current_song_stream_data();

        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            match graphics.reload(&mut rl, &thread, sprites_folder) {
                Ok(_) => println!("Reloaded assets"),
                Err(e) => println!("Failed to reload assets: {}", e),
            }
        }

        // arrow keys to move entity 0
        let vel = 100.0;
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

        let dt = rl.get_frame_time();
        step::step(&mut rl, &mut state, &mut audio, &mut graphics, dt);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(134, 163, 118, 255));

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
