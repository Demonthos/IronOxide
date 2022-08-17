use crate::utils::Position;
use raylib::prelude::*;

use specs::{Component, VecStorage};

/// Handles rendering entities.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub enum Renderer {
    CircleRenderer {
        radius: f32,
        color: Color,
    },
    RectangeRenderer {
        size: Vector2,
        color: Color,
    },
    SpriteRenderer {
        img: Texture2D,
        scale: f32,
        tint: Color,
    },
}

impl Renderer {
    ///     create a image renderer
    pub fn image(
        path: &str,
        scale: f32,
        tint: Color,
        rl: &mut RaylibHandle,
        rlth: &RaylibThread,
    ) -> Renderer {
        Renderer::SpriteRenderer {
            img: rl
                .load_texture_from_image(rlth, &Image::load_image(path).unwrap())
                .unwrap(),
            scale,
            tint: tint,
        }
    }

    /// Render the entity at a location.
    pub fn render(&self, d: &mut impl raylib::core::drawing::RaylibDraw, position: &Position) {
        match self {
            Renderer::CircleRenderer { radius, color } => {
                d.draw_circle_v(position.0 + (Vector2::one() * (*radius)), *radius, color);
            }
            Renderer::RectangeRenderer { size, color } => {
                d.draw_rectangle_v(position.0, size, color);
            }
            Renderer::SpriteRenderer { img, scale, tint } => {
                d.draw_texture_ex(img, position.0, *scale, 0.0, tint)
            }
        }
    }
}
