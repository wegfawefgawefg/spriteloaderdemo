use glam::Vec2;
use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle},
    math::{Rectangle, Vector2},
};

use crate::{entity::Entity, graphics::Graphics, settings::SCREEN_DIMS, state::State};

pub fn draw_entities(d: &mut RaylibDrawHandle, graphics: &Graphics, state: &State) {
    // Create a vector of mutable references to entities
    let mut sorted_entities: Vec<&Entity> = state.entities.iter().collect();

    // // Sort the entities based on their y-foot-position
    // sorted_entities.sort_by(|a, b| {
    //     let a_sprite = graphics.get_sprite_data(a.sprite_animator.sprite);
    //     let a_true_y = a.position.y + a_sprite.size.y as f32 * a.sprite_animator.scale;

    //     let b_sprite = graphics.get_sprite_data(b.sprite_animator.sprite);
    //     let b_true_y = b.position.y + b_sprite.size.y as f32 * b.sprite_animator.scale;

    //     a_true_y
    //         .partial_cmp(&b_true_y)
    //         .unwrap_or(std::cmp::Ordering::Equal)
    // });

    // Sort entities based on their y-position
    sorted_entities.sort_by(|a, b| {
        a.position
            .y
            .partial_cmp(&b.position.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Draw shadows of the sorted entities
    // let shadow_angle = 4.0; // Angle of the shadow in degrees
    //get time
    let time = d.get_time();
    // let shadow_angle = ((time * 100.0) % 360.0) as f32; // Angle of the shadow in degrees
    // it needs to oscilate between -20 and 20
    let shadow_angle = 15.0 * (time * 4.0).sin() as f32;
    // let shadow_scale_y = 0.5; // Scale factor for shadow height
    // it needs to oscilate between 0.1 and 2.0
    let shadow_min_scale_y = 0.2;
    let shadow_max_scale_y = 0.8;
    let shadow_scale_y =
        shadow_min_scale_y + (shadow_max_scale_y - shadow_min_scale_y) * (time * 5.0).sin() as f32;
    for entity in sorted_entities.iter() {
        let sprite_data = graphics.get_sprite_data(entity.sprite_animator.sprite);
        let frame = &sprite_data.frames[entity.sprite_animator.current_frame];

        let scale = entity.sprite_animator.scale;
        let sprite_scaled_size = sprite_data.size.as_vec2() * scale;

        // Calculate shadow dimensions
        let shadow_width = sprite_scaled_size.x;
        let shadow_height = sprite_scaled_size.y * shadow_scale_y;

        // Calculate shadow position (at the entity's feet)
        let shadow_position = entity.position;

        // Origin is at the bottom center of the shadow
        let origin = Vector2::new(shadow_width / 2.0, shadow_height);

        d.draw_texture_pro(
            graphics.get_sprite_texture(entity.sprite_animator.sprite),
            Rectangle::new(
                frame.sample_position.x as f32,
                frame.sample_position.y as f32,
                sprite_data.size.x as f32,
                sprite_data.size.y as f32,
            ),
            Rectangle::new(
                shadow_position.x,
                shadow_position.y,
                shadow_width,
                shadow_height,
            ),
            origin,
            shadow_angle,
            Color::new(0, 0, 0, 100),
        );
    }

    // Draw the sorted entities
    for entity in sorted_entities.iter() {
        let sprite_data = graphics.get_sprite_data(entity.sprite_animator.sprite);
        let frame = &sprite_data.frames[entity.sprite_animator.current_frame];
        let position = entity.position;
        let scale = entity.sprite_animator.scale;
        let sprite_scaled_size = sprite_data.size.as_vec2() * scale;
        // we use feet style origin, so the origin is at the bottom center of the sprite
        let origin = Vec2::new(sprite_scaled_size.x / 2.0, sprite_scaled_size.y);
        // draw a debug blue rect at the origin of the sprite
        // d.draw_rectangle_lines_ex(
        //     Rectangle::new(position.x - origin.x, position.y - origin.y, 2.0, 2.0),
        //     2.0,
        //     Color::BLUE,
        // );

        let draw_position = position - origin;

        d.draw_texture_pro(
            graphics.get_sprite_texture(entity.sprite_animator.sprite),
            Rectangle::new(
                frame.sample_position.x as f32,
                frame.sample_position.y as f32,
                sprite_data.size.x as f32,
                sprite_data.size.y as f32,
            ),
            Rectangle::new(
                draw_position.x,
                draw_position.y,
                sprite_data.size.x as f32 * scale,
                sprite_data.size.y as f32 * scale,
            ),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
    }

    // draw boxes around entities
    // for entity in state.entities.iter() {
    //     // dont forget the position is actually the center bottom, the foot origin, so we have to shif the rect.
    //     let bounds = entity.get_bounds();
    //     let tl = bounds.top_left;
    //     let br = bounds.bottom_right;
    //     d.draw_rectangle_lines_ex(
    //         Rectangle::new(tl.x, tl.y, br.x - tl.x, br.y - tl.y),
    //         2.0,
    //         Color::RED,
    //     );
    // }
}
