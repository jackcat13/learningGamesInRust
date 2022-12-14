mod direction;
mod components;
mod resources;
mod systems;
mod renderer;

use std::ops::ControlFlow;
use std::thread;
use std::error::Error;
use std::time::{Instant, Duration};

use rand::{Rng, thread_rng};
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
    image::{self, LoadTexture, InitFlag},
};
use specs::{World, WorldExt, Builder, DispatcherBuilder, SystemData};

use crate::direction::Direction;
use crate::resources::{TimeDelta, KeyboardEvent, GameStatus};
use crate::components::{
    BoundingBox,
    Velocity,
    Sprite,
    MovementAnimations,
    Player,
    Enemy,
    Goal,
};
use crate::renderer::RendererData;

fn main() -> Result<(), Box<dyn Error>> {

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem.window("Minimal Game", 900, 900)
        .position_centered()
        .build()?;
    let mut canvas = window.into_canvas().build()?;
    let (width, height) = canvas.output_size()?;
    let world_bounds = {
        Rect::from_center((0, 0), width, height)
    };

    let texture_creator = canvas.texture_creator();
    let textures = generate_textures(&texture_creator);
    let bardo_texture = 0;
    let reaper_texture = 1;
    let pink_trees_texture = 2;

    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::Keyboard, "Keyboard", &[])
        .with(systems::AI, "AI", &[])
        .with(systems::Movement {world_bounds}, "Movement", &["Keyboard", "AI"])
        .with(systems::WinLoseChecker, "WinLoseChecker", &["Movement"])
        .with(systems::Animator, "Animator", &["Keyboard", "AI"])
        .build();
    let mut world = World::new();
    dispatcher.setup(&mut world);
    RendererData::setup(&mut world);
    let mut rng = thread_rng();
    let random_x_position = rng.gen_range(-i32::try_from(width/2)?..i32::try_from(width/2)?);
    let y_position = -i32::try_from((height/2)-116)?;
    world.create_entity()
        .with(Goal)
        .with(BoundingBox(Rect::from_center((random_x_position, y_position), 92, 116)))
        .with(Sprite {
            texture_id: pink_trees_texture,
            region: Rect::new(0, 0, 128, 128),
        })
        .build();

    let player_animations = MovementAnimations::standard_walking_animations(
        bardo_texture,
        Rect::new(0, 0, 52, 72),
        3,
        Duration::from_millis(150),
    );

    let random_x_position = rng.gen_range(-i32::try_from(width/2)?..i32::try_from(width/2)?);
    world.create_entity()
        .with(Player {movement_speed: 200})
        .with(BoundingBox(Rect::from_center((random_x_position, 250), 32, 58)))
        .with(Velocity {speed: 0, direction: Direction::Down})
        .with(player_animations.animation_for(Direction::Down).frames[0].sprite.clone())
        .with(player_animations.animation_for(Direction::Down).clone())
        .with(player_animations)
        .build();

    // Generate enemies in random positions. To avoid overlap with anything else, an area of the
    // world coordinate system is divided up into a 2D grid. Each enemy gets a random position
    // within one of the cells of that grid.
    let enemy_animations = MovementAnimations::standard_walking_animations(
        reaper_texture,
        Rect::new(0, 0, 64, 72),
        3,
        Duration::from_millis(150),
    );

    for i in -1..2 {
        for j in -2..0 {
            let enemy_pos = Point::new(
                i * 200 + rng.gen_range(-80..80),
                j * 140 + 200 + rng.gen_range(-40..40),
            );
            let enemy_dir = match rng.gen_range(0..4) {
                0 => Direction::Up,
                1 => Direction::Down,
                2 => Direction::Left,
                3 => Direction::Right,
                _ => unreachable!(),
            };

            world.create_entity()
                .with(Enemy {
                    direction_timer: Instant::now(),
                    direction_change_delay: Duration::from_millis(200),
                })
                .with(BoundingBox(Rect::from_center(enemy_pos, 50, 58)))
                .with(Velocity {speed: 200, direction: enemy_dir})
                .with(enemy_animations.animation_for(enemy_dir).frames[0].sprite.clone())
                .with(enemy_animations.animation_for(enemy_dir).clone())
                .with(enemy_animations.clone())
                .build();
        }
    }

    world.insert(TimeDelta::default());
    world.insert(GameStatus::Running);

    // Begin game loop
    let frame_duration = Duration::from_nanos(1_000_000_000 / 60);
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        // HANDLE EVENTS
        let mut keyboard_event = None;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Up), repeat: false, .. } => {
                    keyboard_event = Some(KeyboardEvent::MoveInDirection(Direction::Up));
                },
                Event::KeyDown { keycode: Some(Keycode::Down), repeat: false, .. } => {
                    keyboard_event = Some(KeyboardEvent::MoveInDirection(Direction::Down));
                },
                Event::KeyDown { keycode: Some(Keycode::Left), repeat: false, .. } => {
                    keyboard_event = Some(KeyboardEvent::MoveInDirection(Direction::Left));
                },
                Event::KeyDown { keycode: Some(Keycode::Right), repeat: false, .. } => {
                    keyboard_event = Some(KeyboardEvent::MoveInDirection(Direction::Right));
                },
                Event::KeyUp { keycode: Some(Keycode::Left), repeat: false, .. } |
                Event::KeyUp { keycode: Some(Keycode::Right), repeat: false, .. } |
                Event::KeyUp { keycode: Some(Keycode::Up), repeat: false, .. } |
                Event::KeyUp { keycode: Some(Keycode::Down), repeat: false, .. } => {
                    keyboard_event = Some(KeyboardEvent::Stop);
                },
                _ => {}
            }
        }

        world.insert(keyboard_event);

        // UPDATE
        *world.write_resource() = TimeDelta(frame_duration);
        dispatcher.dispatch(&world);
        world.maintain();
        if let ControlFlow::Break(_) = check_win_or_lose(&world) {
            break;
        }

        // RENDER
        canvas.set_draw_color(Color::RGB(128, 128, 128));
        canvas.clear();
        let renderer_data: RendererData = world.system_data();
        renderer_data.render(&mut canvas, &textures)?;
        canvas.present();

        // LIMIT FRAMERATE

        // Manage the timing of the game so that the loop doesn't go too quickly or too slowly.
        //
        // Time stepping is a complex topic. We're simplifying things by just always assuming that
        // 1/60 seconds has passed in each iteration of the loop. 1/60th of a second is 60 FPS.
        // There are *many* downsides to the code as it is below, but it's good enough as a
        // starting point.
        //
        // For more information and some more robust approaches:
        // * http://web.archive.org/web/20190506122532/http://gafferongames.com/post/fix_your_timestep/
        // * https://www.gamasutra.com/blogs/BramStolk/20160408/269988/Fixing_your_time_step_the_easy_way_with_the_golden_48537_ms.php
        thread::sleep(frame_duration);
    }

    Ok(())
}

fn generate_textures(texture_creator: &TextureCreator<WindowContext>) -> [sdl2::render::Texture; 3] {
    let error = "Could not load properly textures";
    [
        texture_creator.load_texture("assets/bardo_2x.png").expect(error),
        texture_creator.load_texture("assets/reaper_blade_2x.png").expect(error),
        texture_creator.load_texture("assets/pinktrees_2x.png").expect(error),
    ]
}

fn check_win_or_lose(world: &World) -> ControlFlow<()> {
    match *world.read_resource() {
        GameStatus::Running => {}, // Keep going
        GameStatus::Win => {
            println!("You win!");
            return ControlFlow::Break(());
        },
        GameStatus::Lose => {
            println!("You lose!");
            return ControlFlow::Break(());
        },
    }
    ControlFlow::Continue(())
}