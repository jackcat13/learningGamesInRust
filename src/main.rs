mod direction;
mod components;
mod resources;
mod systems;
mod renderer;
mod sdl_context;

use std::ops::ControlFlow;
use std::thread;
use std::error::Error;
use std::time::{Instant, Duration};

use rand::{Rng, thread_rng};
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
    image::LoadTexture,
};
use sdl_context::SDLGameContext;
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
    
    let sdl_context = sdl_context::sld_context();

    let texture_creator = sdl_context.canvas.texture_creator();
    let error = String::from("Could not load properly textures");
    let textures = vec!(
        texture_creator.load_texture("assets/bardo_2x.png").expect(error.as_str()),
        texture_creator.load_texture("assets/reaper_blade_2x.png").expect(error.as_str()),
        texture_creator.load_texture("assets/pinktrees_2x.png").expect(error.as_str()),
    );

    let mut dispatcher = DispatcherBuilder::new()
        .with(systems::Keyboard, "Keyboard", &[])
        .with(systems::AI, "AI", &[])
        .with(systems::Movement {world_bounds: sdl_context.world_bounds}, "Movement", &["Keyboard", "AI"])
        .with(systems::WinLoseChecker, "WinLoseChecker", &["Movement"])
        .with(systems::Animator, "Animator", &["Keyboard", "AI"])
        .build();

    let mut world = World::new();
    dispatcher.setup(&mut world);
    RendererData::setup(&mut world);
    
    generate_goal_in_world(&mut world, &sdl_context);
    generate_player_in_world(&mut world, &sdl_context);
    generate_enemies_in_world(&mut world, &sdl_context);

    world.insert(TimeDelta::default());
    world.insert(GameStatus::Running);

    game_loop(sdl_context, world, dispatcher, textures)?;

    Ok(())
}

fn game_loop(mut sdl_context: SDLGameContext, mut world: World, mut dispatcher: specs::Dispatcher, textures: Vec<sdl2::render::Texture>) -> Result<(), Box<dyn Error>> {
    let frame_duration = Duration::from_nanos(1_000_000_000 / 60);
    let mut event_pump = sdl_context.context.event_pump()?;
    Ok('running: loop {
        // Handle events
        let keyboard_event = handle_game_events(&mut event_pump);
        if keyboard_event == Some(KeyboardEvent::Escape){
            break 'running;
        }
        world.insert(keyboard_event);

        // Update world
        if let ControlFlow::Break(_) = update_world(&mut world, frame_duration, &mut dispatcher) {
            break;
        }

        // Render game
        render_game(&mut sdl_context, &world, &textures)?;

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
    })
}

/// RENDER GAME IN WINDOW
fn render_game(sdl_context: &mut SDLGameContext, world: &World, textures: &Vec<sdl2::render::Texture>) -> Result<(), Box<dyn Error>> {
    sdl_context.canvas.set_draw_color(Color::RGB(128, 128, 128));
    sdl_context.canvas.clear();
    let renderer_data: RendererData = world.system_data();
    renderer_data.render(&mut sdl_context.canvas, textures)?;
    sdl_context.canvas.present();
    Ok(())
}

/// UPDATE GAME
fn update_world(world: &mut World, frame_duration: Duration, dispatcher: &mut specs::Dispatcher) -> ControlFlow<()> {
    *world.write_resource() = TimeDelta(frame_duration);
    dispatcher.dispatch(&*world);
    world.maintain();
    if let ControlFlow::Break(_) = check_win_or_lose(&*world) {
        return ControlFlow::Break(());
    }
    ControlFlow::Continue(())
}

/// HANDLE GAME EVENTS
fn handle_game_events(event_pump: &mut sdl2::EventPump) -> Option<KeyboardEvent> {
    let mut keyboard_event = None;
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                keyboard_event = Some(KeyboardEvent::Escape)
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
    keyboard_event
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

fn generate_goal_in_world(world: &mut World, sdl_context: &SDLGameContext){
    let mut rng = thread_rng();
    let position_error = "Error generating positions of goal";
    let random_x_position = rng.gen_range(-i32::try_from(sdl_context.width/2).expect(position_error)..i32::try_from(sdl_context.width/2).expect(position_error));
    let y_position = -i32::try_from((sdl_context.height/2)-116).expect(position_error);
    world.create_entity()
        .with(Goal)
        .with(BoundingBox(Rect::from_center((random_x_position, y_position), 92, 116)))
        .with(Sprite {
            texture_id: sdl_context.pink_tree_texture,
            region: Rect::new(0, 0, 128, 128),
        })
        .build();
}

fn generate_player_in_world(world: &mut World, sdl_context: &SDLGameContext){
    let mut rng = thread_rng();
    let position_error = "Error generating positions of player";
    let player_animations = MovementAnimations::standard_walking_animations(
        sdl_context.bardo_texture,
        Rect::new(0, 0, 52, 72),
        3,
        Duration::from_millis(150),
    );
    let random_x_position = rng.gen_range(-i32::try_from(sdl_context.width/2).expect(position_error)..i32::try_from(sdl_context.width/2).expect(position_error));
    world.create_entity()
        .with(Player {movement_speed: 200})
        .with(BoundingBox(Rect::from_center((random_x_position, 250), 32, 58)))
        .with(Velocity {speed: 0, direction: Direction::Down})
        .with(player_animations.animation_for(Direction::Down).frames[0].sprite.clone())
        .with(player_animations.animation_for(Direction::Down).clone())
        .with(player_animations)
        .build();
}

/// Generate enemies in random positions. To avoid overlap with anything else, an area of the
/// world coordinate system is divided up into a 2D grid. Each enemy gets a random position
/// within one of the cells of that grid.
fn generate_enemies_in_world(world: &mut World, sdl_context: &SDLGameContext){
    let mut rng = thread_rng();
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
            generate_ennemy_in_world(world, enemy_pos, enemy_dir, &sdl_context);
        }
    }
}

fn generate_ennemy_in_world(world: &mut World, enemy_pos: Point, enemy_dir: Direction, sdl_context: &SDLGameContext) {
    let enemy_animations = MovementAnimations::standard_walking_animations(
        sdl_context.reaper_texture,
        Rect::new(0, 0, 64, 72),
        3,
        Duration::from_millis(150),
    );
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