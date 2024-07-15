use glam::{UVec2, Vec2};
use rand::Rng;
use raylib::ffi::{SetTraceLogLevel, TraceLogLevel};
use raylib::prelude::*;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::EnumIter;

const SCREEN_DIMS: UVec2 = UVec2::new(1000, 1000);

#[derive(Copy, Clone, Debug, EnumIter, EnumCount, PartialEq, Eq, Hash)]
pub enum Sprite {
    ManWalk,
    ManIdle,
    Tree,
    Reticle,
    Apple,
}

impl Sprite {
    fn to_filename(self) -> &'static str {
        match self {
            Sprite::ManWalk => "man_walk",
            Sprite::ManIdle => "man_idle",
            Sprite::Tree => "tree",
            Sprite::Reticle => "reticle",
            Sprite::Apple => "apple",
        }
    }
}

#[derive(Debug)]
pub struct Frame {
    pub sample_position: UVec2,
    pub duration: f32,
}

#[derive(Debug)]
pub struct SpriteData {
    pub frames: Vec<Frame>,
    pub size: UVec2,
}

fn load_sprites(asset_folder: &str) -> Result<Vec<SpriteData>, String> {
    let mut sprites: Vec<SpriteData> = vec![];
    for sprite in Sprite::iter() {
        let filename = sprite.to_filename();
        let json_path = Path::new(asset_folder).join(format!("{}.json", filename));
        let sprite_data = load_sprite_data(&json_path)?;
        sprites.push(sprite_data);
    }
    Ok(sprites)
}

fn load_sprite_data(json_path: &Path) -> Result<SpriteData, String> {
    let file = File::open(json_path).map_err(|e| format!("Failed to open JSON file: {}", e))?;
    let reader = BufReader::new(file);
    let json: Value =
        serde_json::from_reader(reader).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let frames = json["frames"].as_object().ok_or("Invalid JSON structure")?;
    let mut sprite_frames = Vec::new();
    let mut size = UVec2::ZERO;

    for (_, frame_data) in frames {
        let frame = frame_data["frame"]
            .as_object()
            .ok_or("Invalid frame data")?;
        let x = frame["x"].as_u64().ok_or("Invalid x")? as u32;
        let y = frame["y"].as_u64().ok_or("Invalid y")? as u32;
        let duration = frame_data["duration"].as_f64().ok_or("Invalid duration")? as f32;

        sprite_frames.push(Frame {
            sample_position: UVec2::new(x, y),
            duration,
        });

        if size == UVec2::ZERO {
            let w = frame["w"].as_u64().ok_or("Invalid w")? as u32;
            let h = frame["h"].as_u64().ok_or("Invalid h")? as u32;
            size = UVec2::new(w, h);
        }
    }

    Ok(SpriteData {
        frames: sprite_frames,
        size,
    })
}

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

#[derive(Debug, PartialEq)]
pub enum EntityType {
    Man,
    Tree,
    Reticle,
    Apple,
}

#[derive(Debug)]
pub struct Entity {
    pub entity_type: EntityType,
    pub position: Vec2,
    pub velocity: Vec2,
    pub sprite_animator: SpriteAnimator,
    pub follows: Option<usize>,
}

#[derive(Debug)]
pub struct SpriteAnimator {
    pub sprite: Sprite,
    pub current_frame: usize,
    pub current_time: f32,
    pub scale: f32,
}

impl SpriteAnimator {
    pub fn new(sprite: Sprite) -> Self {
        Self {
            sprite,
            current_frame: 0,
            current_time: 0.0,
            scale: 1.0,
        }
    }

    pub fn set_sprite(&mut self, sprite: Sprite) {
        self.sprite = sprite;
        self.current_frame = 0;
        self.current_time = 0.0;
    }

    pub fn step(&mut self, sprites: &[SpriteData], dt: f32) {
        let sprite_data = &sprites[self.sprite as usize];
        let frame = &sprite_data.frames[self.current_frame];
        self.current_time += dt;
        if self.current_time >= frame.duration {
            self.current_time = 0.0;
            self.current_frame = (self.current_frame + 1) % sprite_data.frames.len();
        }
    }

    pub fn randomize_frame(&mut self, sprites: &[SpriteData]) {
        let sprite_data = &sprites[self.sprite as usize];
        self.current_frame = rand::thread_rng().gen_range(0..sprite_data.frames.len());
    }
}

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

    pub fn step_entities(&mut self, sprites: &[SpriteData], dt: f32) {
        // step positions
        for entity in &mut self.entities {
            entity.position += entity.velocity * dt;
        }

        // wrap around screen
        for entity in &mut self.entities {
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

        // set sprite based on velocity
        const MIN_WALK_SPEED: f32 = 100.0;
        for entity in &mut self.entities {
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

        // step sprites
        let dt_ms = dt * 1000.0;
        for entity in &mut self.entities {
            entity.sprite_animator.step(sprites, dt_ms);
        }

        // if entity 02, touches the apple [entity 01], spawn a new man at the end of the list, make him follow the last entity in the list
        // consider the apple scale
        let apple = &self.entities[1];
        let apple_sprite = &sprites[apple.sprite_animator.sprite as usize];
        let apple_tl = apple.position;
        let apple_br = apple.position + apple_sprite.size.as_vec2() * apple.sprite_animator.scale;

        for entity in &mut self.entities {
            if entity.entity_type != EntityType::Man {
                continue;
            }
            let man = entity;
            let man_sprite = &sprites[man.sprite_animator.sprite as usize];
            let man_tl = man.position;
            let man_br = man.position + man_sprite.size.as_vec2() * man.sprite_animator.scale;

            if intersects(apple_tl, apple_br, man_tl, man_br) {
                let last_entity = self.entities.len() - 1;
                let last_entity_pos = self.entities[last_entity].position;
                self.add_entity(Entity {
                    entity_type: EntityType::Man,
                    position: last_entity_pos,
                    velocity: Vec2::new(0.0, 0.0),
                    sprite_animator: SpriteAnimator {
                        sprite: Sprite::ManIdle,
                        current_frame: 0,
                        current_time: 0.0,
                        scale: 6.0,
                    },
                    follows: Some(last_entity),
                });
                // move the apple to a new random position
                let rng = &mut rand::thread_rng();
                let new_pos = Vec2::new(
                    rng.gen_range(0.0..SCREEN_DIMS.x as f32),
                    rng.gen_range(0.0..SCREEN_DIMS.y as f32),
                );
                self.entities[1].position = new_pos;
                break;
            }
        }
    }
}

pub fn intersects(a_tl: Vec2, a_br: Vec2, b_tl: Vec2, b_br: Vec2) -> bool {
    a_tl.x < b_br.x && a_br.x > b_tl.x && a_tl.y < b_br.y && a_br.y > b_tl.y
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

pub fn draw_entities(d: &mut RaylibDrawHandle, graphics: &Graphics, state: &State) {
    // Create a vector of mutable references to entities
    let mut sorted_entities: Vec<&Entity> = state.entities.iter().collect();

    // Sort the entities based on their y-position
    sorted_entities.sort_by(|a, b| {
        let a_sprite = graphics.get_sprite_data(a.sprite_animator.sprite);
        let a_true_y = a.position.y + a_sprite.size.y as f32 * a.sprite_animator.scale;

        let b_sprite = graphics.get_sprite_data(b.sprite_animator.sprite);
        let b_true_y = b.position.y + b_sprite.size.y as f32 * b.sprite_animator.scale;

        a_true_y
            .partial_cmp(&b_true_y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Draw shadows of the stored entities, greyed out essentially

    let shadow_shift = Vec2::new(20.0, 35.0);
    for entity in sorted_entities.iter() {
        let sprite_data = graphics.get_sprite_data(entity.sprite_animator.sprite);
        let frame = &sprite_data.frames[entity.sprite_animator.current_frame];
        // origin should be the bottom center of the sprite
        let origin = Vector2::new(sprite_data.size.x as f32 / 2.0, sprite_data.size.y as f32);
        d.draw_texture_pro(
            graphics.get_sprite_texture(entity.sprite_animator.sprite),
            Rectangle::new(
                frame.sample_position.x as f32,
                frame.sample_position.y as f32,
                sprite_data.size.x as f32,
                sprite_data.size.y as f32,
            ),
            Rectangle::new(
                entity.position.x + shadow_shift.x,
                entity.position.y + shadow_shift.y,
                sprite_data.size.x as f32 * entity.sprite_animator.scale,
                sprite_data.size.y as f32 * entity.sprite_animator.scale * 0.5,
            ),
            origin,
            30.0,
            Color::new(0, 0, 0, 100),
        );
    }

    // Draw the sorted entities
    for entity in sorted_entities.iter() {
        let sprite_data = graphics.get_sprite_data(entity.sprite_animator.sprite);
        let frame = &sprite_data.frames[entity.sprite_animator.current_frame];
        d.draw_texture_pro(
            graphics.get_sprite_texture(entity.sprite_animator.sprite),
            Rectangle::new(
                frame.sample_position.x as f32,
                frame.sample_position.y as f32,
                sprite_data.size.x as f32,
                sprite_data.size.y as f32,
            ),
            Rectangle::new(
                entity.position.x,
                entity.position.y,
                sprite_data.size.x as f32 * entity.sprite_animator.scale,
                sprite_data.size.y as f32 * entity.sprite_animator.scale,
            ),
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );
    }
}

fn main() -> Result<(), String> {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_DIMS.x as i32, SCREEN_DIMS.y as i32)
        .title("Auto-Managed Textures Sprite System")
        .build();

    // set window position
    rl.set_window_position(500, 500);
    // set fps to 144
    rl.set_target_fps(144);

    let asset_folder = "./assets";
    let mut graphics = Graphics::new(&mut rl, &thread, asset_folder)?;

    let mut state = State::new();

    // reticle
    let reticle_entity = Entity {
        entity_type: EntityType::Reticle,
        position: Vec2::new(50.0, 200.0),
        velocity: Vec2::new(0.0, 0.0),
        sprite_animator: SpriteAnimator {
            sprite: Sprite::Reticle,
            current_frame: 0,
            current_time: 0.0,
            scale: 5.0,
        },
        follows: None,
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
        sprite_animator: SpriteAnimator {
            sprite: Sprite::Apple,
            current_frame: 0,
            current_time: 0.0,
            scale: 5.0,
        },
        follows: None,
    });

    // trees
    // in random positions, only 20
    const NUM_TREES: usize = 20;
    let rng = &mut rand::thread_rng();
    for _ in 0..NUM_TREES {
        let mut entity = Entity {
            entity_type: EntityType::Tree,
            position: Vec2::new(
                rng.gen_range(0.0..SCREEN_DIMS.x as f32),
                rng.gen_range(0.0..SCREEN_DIMS.y as f32),
            ),
            velocity: Vec2::new(0.0, 0.0),
            sprite_animator: SpriteAnimator {
                sprite: Sprite::Tree,
                current_frame: 0,
                current_time: 0.0,
                // random number between 30 and 50
                scale: rng.gen_range(10.0..15.0),
            },
            follows: None,
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
            sprite_animator: SpriteAnimator {
                sprite: Sprite::ManIdle,
                current_frame: 0,
                current_time: 0.0,
                scale: 6.0,
            },
            follows,
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
        let dist = 10.0;
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
        state.step_entities(&graphics.sprites, dt);

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
