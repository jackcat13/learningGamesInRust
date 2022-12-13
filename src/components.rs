use std::sync::Arc;
use std::time::{Instant, Duration};

use sdl2::rect::Rect;
use specs::{Component, VecStorage, NullStorage};

use crate::direction::Direction;

/// The position and dimensions of an entity in world coordinates
///
/// Assumes that the entity is positioned at the center of the bounding box.
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct BoundingBox(pub Rect);

/// Allows an entity to move with the given speed in the given direction
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct Velocity {
    /// The speed of the entity's movement
    pub speed: i32,
    /// The direction of the entity's movement
    pub direction: Direction,
}

/// The sprite to render for a given entity. The entity must also have a
/// `Position` component in order for it to be drawn on the screen.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
#[storage(VecStorage)]
pub struct Sprite {
    /// The texture containing the spritesheet to copy sprites from
    pub texture_id: usize,
    /// The region of the spritesheet to copy
    pub region: Rect,
}

/// A sequence of sprites that will be used to update an entity's `Sprite` component
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct Animation {
    /// The frames of the animation and their individual timings
    ///
    /// Stored using `Arc` to make cloning Animation cheaper
    pub frames: Arc<Vec<Frame>>,
    /// The current animation frame
    pub current_frame: usize,
    /// The amount of time elapsed since the animation frame changed
    pub frame_timer: Instant,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    /// The sprite to render for this frame
    pub sprite: Sprite,
    /// The duration of the animation frame. The next frame will begin once this amount of time has
    /// elapsed.
    pub duration: Duration,
}

/// Causes an entity's `Animation` component to be updated based on the direction in
/// its `Velocity` component.
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct MovementAnimations {
    /// The animation for when an entity is moving in the "up" direction
    pub walking_up: Animation,
    /// The animation for when an entity is moving in the "down" direction
    pub walking_down: Animation,
    /// The animation for when an entity is moving in the "left" direction
    pub walking_left: Animation,
    /// The animation for when an entity is moving in the "right" direction
    pub walking_right: Animation,
}

impl MovementAnimations {
    /// Generates a set of movement animations based on the standard convention for spritesheets
    /// in this project. Expects 4 rows of `frames_length` frames each with the animations ordered:
    /// walking down, walking left, walking right, walking up.
    ///
    /// The `top_left_frame` parameter provides the offset in the spritesheet as well as the
    /// width and height (in pixels) of each frame. The `step_delay` is the duration of each frame.
    pub fn standard_walking_animations(
        texture_id: usize,
        top_left_frame: Rect,
        frames_length: usize,
        step_delay: Duration,
    ) -> Self {
        let animation = |row| Animation {
            frames: Arc::new((0..frames_length as i32).map(|frame| Frame {
                sprite: Sprite {
                    texture_id,
                    region: {
                        let mut frame_region = top_left_frame.clone();
                        frame_region.offset(
                            frame * top_left_frame.width() as i32,
                            row * top_left_frame.height() as i32,
                        );
                        frame_region
                    },
                },
                duration: step_delay,
            }).collect()),
            current_frame: 0,
            frame_timer: Instant::now(),
        };

        Self {
            walking_up: animation(3),
            walking_down: animation(0),
            walking_left: animation(1),
            walking_right: animation(2),
        }
    }

    pub fn animation_for(&self, direction: Direction) -> &Animation {
        match direction {
            Direction::Up => &self.walking_up,
            Direction::Down => &self.walking_down,
            Direction::Left => &self.walking_left,
            Direction::Right => &self.walking_right,
        }
    }
}

/// Marks an entity as the keyboard controlled player
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct Player {
    /// The speed of the player when they are moving
    pub movement_speed: i32,
}

/// Marks an entity as an enemy that will cause damage to the player
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct Enemy {
    /// The amount of time elapsed since the direction was changed
    pub direction_timer: Instant,
    /// The amount of time to wait between direction changes
    pub direction_change_delay: Duration,
}

/// Marks an entity as the goal. If the player reaches this, they win the game.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[storage(NullStorage)]
pub struct Goal;