use specs::{System, SystemData, Read, ReadStorage, WriteStorage, Join, World, prelude::ResourceId};

use crate::resources::KeyboardEvent;
use crate::components::{Player, Velocity};
use KeyboardEvent::*;

pub struct Keyboard;

#[derive(SystemData)]
pub struct KeyboardData<'a> {
    players: ReadStorage<'a, Player>,
    velocities: WriteStorage<'a, Velocity>,
    keyboard_event: Read<'a, Option<KeyboardEvent>>,
}

impl<'a> System<'a> for Keyboard {
    type SystemData = KeyboardData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let KeyboardData {players, mut velocities, keyboard_event} = data;
        match *keyboard_event {
            Some(MoveInDirection(direction)) => {
                for (&Player {movement_speed}, velocity) in (&players, &mut velocities).join() {
                    velocity.speed = movement_speed;
                    velocity.direction = direction;
                }
            },
            Some(Stop) => {
                for (_, velocity) in (&players, &mut velocities).join() {
                    velocity.speed = 0;
                }
            },
            None => {},
        }
    }
}