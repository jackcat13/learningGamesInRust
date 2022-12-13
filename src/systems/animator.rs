use std::time::Instant;

use specs::{System, SystemData, Entities, ReadStorage, WriteStorage, Join, World, prelude::ResourceId};

use crate::components::{Velocity, Animation, Sprite, MovementAnimations};

pub struct Animator;

#[derive(SystemData)]
pub struct AnimatorData<'a> {
    entities: Entities<'a>,
    velocities: ReadStorage<'a, Velocity>,
    movement_animations: ReadStorage<'a, MovementAnimations>,
    animations: WriteStorage<'a, Animation>,
    sprites: WriteStorage<'a, Sprite>,
}

impl<'a> System<'a> for Animator {
    type SystemData = AnimatorData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let AnimatorData {
            entities,
            velocities,
            movement_animations,
            mut animations,
            mut sprites,
        } = data;

        for (entity, &Velocity {speed, direction}, move_animations) in (&*entities, &velocities, &movement_animations).join() {
            let anim_frames = animations.get(entity).map(|anim| anim.frames.clone());
            if speed == 0 && anim_frames.is_some() {
                animations.remove(entity);
                continue;
            }
            let dir_anim = move_animations.animation_for(direction);
            let needs_update = match anim_frames {
                Some(anim_frames) => anim_frames != dir_anim.frames,
                None => true,
            };
            if needs_update {
                animations.insert(entity, dir_anim.clone())
                    .expect("failed to update animation");
            }
        }

        for (anim, sprite) in (&mut animations, &mut sprites).join() {
            if anim.frame_timer.elapsed() >= anim.frames[anim.current_frame].duration {
                anim.current_frame = (anim.current_frame + 1) % anim.frames.len();
                anim.frame_timer = Instant::now();
                *sprite = anim.frames[anim.current_frame].sprite.clone();
            }
        }
    }
}