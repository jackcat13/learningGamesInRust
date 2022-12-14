use sdl2::rect::Rect;
use specs::{System, SystemData, ReadExpect, ReadStorage, WriteStorage, Join, World, prelude::ResourceId};

use crate::resources::TimeDelta;
use crate::components::{BoundingBox, Velocity};

pub struct Movement {
    pub world_bounds: Rect,
}

#[derive(SystemData)]
pub struct MovementData<'a> {
    velocities: ReadStorage<'a, Velocity>,
    bounding_boxes: WriteStorage<'a, BoundingBox>,
    time_delta: ReadExpect<'a, TimeDelta>,
}

impl<'a> System<'a> for Movement {
    type SystemData = MovementData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let MovementData {velocities, mut bounding_boxes, time_delta} = data;
        let TimeDelta(time_elapsed) = *time_delta;

        for (&Velocity {speed, direction}, BoundingBox(bounds)) in (&velocities, &mut bounding_boxes).join() {
            if speed == 0 {
                continue;
            }
            let distance = speed * time_elapsed.as_micros() as i32 / 1_000_000;
            let new_pos = bounds.center() + direction.into_point() * distance;
            let new_bounds = Rect::from_center(new_pos, bounds.width(), bounds.height());
            *bounds = new_bounds;
            /*if self.world_bounds.contains_rect(new_bounds) {
            }*/
        }
    }
}