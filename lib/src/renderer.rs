use crate::utils::Position;
use raylib::prelude::*;
use raylib::core::math::Rectangle;

use specs::{Component, VecStorage};

/// currently only handles rendering primitives, but raylib supports sprites.
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub enum Renderer {
    CircleRenderer { radius: f32, color: Color },
    RectangeRenderer { size: Vector2, color: Color },
    SpriteRenderer { img: Texture2D , size: Rectangle, tint: Color },
}

impl Renderer {
    pub fn image(path: &str, size: Vector2, tint: Color, rl: &mut RaylibHandle, rlth: &RaylibThread) -> Renderer{
        Renderer::SpriteRenderer{img: rl.load_texture_from_image(rlth, &Image::load_image(path).unwrap()).unwrap(), size: Rectangle{x: 0.0, y: 0.0, width: size.x, height: size.y}, tint: tint}
    }

    pub fn render(&self, d: &mut impl raylib::core::drawing::RaylibDraw, position: &Position) {
        match self {
            Renderer::CircleRenderer { radius, color } => {
                d.draw_circle_v(position.0 + (Vector2::one() * (*radius)), *radius, color);
            },
            Renderer::RectangeRenderer { size, color } => {
                d.draw_rectangle_v(position.0, size, color);
            },
            Renderer::SpriteRenderer{img, size, tint} => {
                 d.draw_texture_rec(img, size, position.0, tint)
            }
        }
    }
}
