use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{
    Sdl,
    rect::Rect,
    image::{self, InitFlag}
};

pub struct SDLGameContext {
    pub context: Sdl,
    pub canvas: Canvas<Window>,
    pub world_bounds: Rect,
    pub bardo_texture: usize,
    pub reaper_texture: usize,
    pub pink_tree_texture: usize,
    pub width: u32,
    pub height: u32
}

const WORLD_WIDTH: u32 = 900;
const WORLD_HEIGHT: u32 = 900;

pub fn sld_context() -> SDLGameContext {
    let sdl_context = sdl2::init().expect("Failed to load sdl2");
    let video_subsystem = sdl_context.video().expect("Failed to load video subsystem");
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG).expect("Failed to load image context");
    let window = video_subsystem.window("Minimal Game", WORLD_WIDTH, WORLD_HEIGHT)
        .position_centered()
        .build()
        .expect("Failed to build window");
    let canvas = window.into_canvas().build().expect("Failed to build canvas");
    let (width, height) = canvas.output_size().expect("Failed to setup canvas width and height");
    let world_bounds = {
        Rect::from_center((0, 0), width, height)
    };

    SDLGameContext{
        context: sdl_context,
        canvas: canvas,
        world_bounds: world_bounds,
        bardo_texture: 0,
        reaper_texture: 1,
        pink_tree_texture: 2,
        width: width,
        height: height
    }
}