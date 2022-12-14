//! The renderer cannot be a normal system because it holds values that must be used on the main
//! thread. It cannot be executed in parallel like other systems. Another complication is that it
//! returns a `Result` whereas normal systems do not return anything.

use specs::{SystemData, ReadStorage, Join, World, prelude::ResourceId};
use sdl2::{
    rect::{Point, Rect},
    render::{WindowCanvas, Texture},
};

use crate::components::{BoundingBox, Sprite, Player};

/// Data from the world required by the renderer
#[derive(SystemData)]
pub struct RendererData<'a> {
    players: ReadStorage<'a, Player>,
    bounding_boxes: ReadStorage<'a, BoundingBox>,
    sprites: ReadStorage<'a, Sprite>,
}

impl<'a> RendererData<'a> {
    pub fn render(&self, canvas: &mut WindowCanvas, textures: &[Texture]) -> Result<(), String> {
        let RendererData {
            players,
            bounding_boxes, 
            sprites
        } = self;

        // The screen coordinate system has (0, 0) in its top-left corner whereas the
        // world coordinate system is centered on the player.
        let (width, height) = canvas.output_size()?;
        let world_to_screen_offset = Point::new(width as i32 / 2, height as i32 / 2);
        let mut player_bounds = Rect::from_center((0,0), 0, 0);
        for (_, BoundingBox(temp_player_bounds)) in (players, bounding_boxes).join() {
            player_bounds = temp_player_bounds.clone(); //TODO: find a clean code way to retrieve this from storage
        }
        for (&BoundingBox(bounds), &Sprite {texture_id, region: sprite_rect}) in (bounding_boxes, sprites).join() {
            let screen_pos = bounds.center() - player_bounds.center() + world_to_screen_offset;
            let screen_rect = Rect::from_center(screen_pos, sprite_rect.width(), sprite_rect.height());
            canvas.copy(&textures[texture_id], sprite_rect, screen_rect)?;
        }

        Ok(())
    }
}