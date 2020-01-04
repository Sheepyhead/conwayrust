use core::sync::atomic::{AtomicU64, Ordering};
use ggez::event::{self, EventHandler};
use ggez::graphics::{Canvas, DrawParam, Image};
use ggez::input::keyboard;
use ggez::input::keyboard::KeyCode;
use ggez::input::mouse;
use ggez::input::mouse::{MouseButton, MouseCursor};
use ggez::{conf::WindowMode, graphics, Context, ContextBuilder, GameResult};
use rand;
use rand::Rng;
use rayon::prelude::*;
use std::{thread, time::Duration};

fn main() {
    // Make a Context.
    let (mut ctx, mut event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
        .window_mode(WindowMode {
            width: 800.0,
            height: 800.0,
            ..Default::default()
        })
        .build()
        .expect("aieee, could not create ggez context!");

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let mut my_game = MyGame::new(&mut ctx);

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}

struct MyGame {
    image: Image,
    generation: u64,
    started: bool,
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        // Load/create resources such as images here.
        let mut buffer = Canvas::with_window_size(_ctx)
            .expect("Failed to create initial canvas!")
            .into_inner()
            .to_rgba8(_ctx)
            .expect("Failed to read initial buffer");
        let width = graphics::drawable_size(_ctx).0 as u64;
        let height = graphics::drawable_size(_ctx).1 as u64;
        println!("{}, {}", width, height);
        const PIXEL_SIZE: u64 = 4;

        for x in 0..width {
            for y in 0..height {
                let index = width * PIXEL_SIZE * y + x * PIXEL_SIZE;

                buffer[(index + 1) as usize] = if y == 80 && x < 120 {
                    match x {
                        80 | 81 | 82 | 83 | 84 | 85 | 86 | 87 | 89 | 90 | 91 | 92 | 93 | 97
                        | 98 | 99 | 106 | 107 | 108 | 109 | 110 | 111 | 112 | 114 | 115 | 116
                        | 117 | 118 => 255,
                        _ => 0,
                    }
                } else {
                    0
                };
                buffer[index as usize] = 0;
                buffer[(index + 2) as usize] = 0;
                buffer[(index + 3) as usize] = 255;
            }
        }
        MyGame {
            image: Image::from_rgba8(_ctx, width as u16, height as u16, &buffer)
                .expect("Failed to create initial image"),
            generation: 0,
            started: false,
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if self.started {
            if !mouse::cursor_hidden(_ctx) {
                mouse::set_cursor_hidden(_ctx, true);
            }
            let width = graphics::drawable_size(_ctx).0 as u64;
            let height = graphics::drawable_size(_ctx).1 as u64;
            let read_buffer = self
                .image
                .to_rgba8(_ctx)
                .expect("Failed to dump image to bytes");
            const PIXEL_SIZE: u64 = 4;
            let write_buffer = read_buffer
                .par_iter()
                .enumerate()
                .map(|(index, byte)| {
                    match index % 4 {
                        0 => *byte, // Red byte, start of new pixel
                        1 => {
                            // Green byte
                            let current_pixel_index = index as u64 - index as u64 % PIXEL_SIZE;
                            let x = (current_pixel_index % (width * PIXEL_SIZE)) / PIXEL_SIZE;
                            let y = current_pixel_index / (width * PIXEL_SIZE);
                            let left_neighbor_index = width * PIXEL_SIZE * y
                                + if x == 0 { width - 1 } else { x - 1 } * PIXEL_SIZE
                                + 1;
                            let top_left_neighbor_index =
                                width * PIXEL_SIZE * if y == 0 { height - 1 } else { y - 1 }
                                    + if x == 0 { width - 1 } else { x - 1 } * PIXEL_SIZE
                                    + 1;
                            let top_neighbor_index =
                                width * PIXEL_SIZE * if y == 0 { height - 1 } else { y - 1 }
                                    + x * PIXEL_SIZE
                                    + 1;
                            let top_right_neighbor_index =
                                width * PIXEL_SIZE * if y == 0 { height - 1 } else { y - 1 }
                                    + if x == width - 1 { 0 } else { x + 1 } * PIXEL_SIZE
                                    + 1;
                            let right_neighbor_index = width * PIXEL_SIZE * y
                                + if x == width - 1 { 0 } else { x + 1 } * PIXEL_SIZE
                                + 1;
                            let bottom_right_neighbor_index =
                                width * PIXEL_SIZE * if y == height - 1 { 0 } else { y + 1 }
                                    + if x == width - 1 { 0 } else { x + 1 } * PIXEL_SIZE
                                    + 1;
                            let bottom_neighbor_index =
                                width * PIXEL_SIZE * if y == height - 1 { 0 } else { y + 1 }
                                    + x * PIXEL_SIZE
                                    + 1;
                            let bottom_left_neighbor_index =
                                width * PIXEL_SIZE * if y == height - 1 { 0 } else { y + 1 }
                                    + if x == 0 { width - 1 } else { x - 1 } * PIXEL_SIZE
                                    + 1;
                            let left_neighbor_alive =
                                read_buffer[left_neighbor_index as usize] == 255;
                            let top_left_neighbor_alive =
                                read_buffer[top_left_neighbor_index as usize] == 255;
                            let top_neighbor_alive =
                                read_buffer[top_neighbor_index as usize] == 255;
                            let top_right_neighbor_alive =
                                read_buffer[top_right_neighbor_index as usize] == 255;
                            let right_neighbor_alive =
                                read_buffer[right_neighbor_index as usize] == 255;
                            let bottom_right_neighbor_alive =
                                read_buffer[bottom_right_neighbor_index as usize] == 255;
                            let bottom_neighbor_alive =
                                read_buffer[bottom_neighbor_index as usize] == 255;
                            let bottom_left_neighbor_alive =
                                read_buffer[bottom_left_neighbor_index as usize] == 255;
                            let number_of_living_neighbors = left_neighbor_alive as u64 + top_left_neighbor_alive as u64
                                + top_neighbor_alive as u64 + top_right_neighbor_alive as u64
                                + right_neighbor_alive as u64 + bottom_right_neighbor_alive as u64
                                + bottom_neighbor_alive as u64 + bottom_left_neighbor_alive as u64;

                            if *byte == 0 {
                                // current pixel is dead
                                if number_of_living_neighbors == 3 {
                                    255
                                } else {
                                    0
                                }
                            } else {
                                // current pixel is alive
                                if !(number_of_living_neighbors == 2
                                    || number_of_living_neighbors == 3)
                                {
                                    0
                                } else {
                                    255
                                }
                            }
                        }
                        2 => *byte, // Blue byte
                        3 => *byte, // Alpha byte
                        _ => *byte, // Should not be possible
                    }
                })
                .collect::<Vec<u8>>();
            self.generation += 1;
            println!(
                "Generation {}",
                self.generation,
            );
            self.image = Image::from_rgba8(_ctx, width as u16, height as u16, &write_buffer)
                .expect("Failed to update image");
            if keyboard::is_key_pressed(_ctx, KeyCode::R) {
                let mut buffer = Canvas::with_window_size(_ctx)
                    .expect("Failed to create initial canvas!")
                    .into_inner()
                    .to_rgba8(_ctx)
                    .expect("Failed to read initial buffer");
                let width = graphics::drawable_size(_ctx).0 as u64;
                let height = graphics::drawable_size(_ctx).1 as u64;
                println!("{}, {}", width, height);
                const PIXEL_SIZE: u64 = 4;

                for x in 0..width {
                    for y in 0..height {
                        let index = width * PIXEL_SIZE * y + x * PIXEL_SIZE;
                        buffer[(index + 1) as usize] = 0;
                        buffer[index as usize] = 0;
                        buffer[(index + 2) as usize] = 0;
                        buffer[(index + 3) as usize] = 255;
                    }
                }
                self.image = Image::from_rgba8(_ctx, width as u16, height as u16, &buffer)
                    .expect("Failed to create initial image");
                self.generation = 0;
                self.started = false;
            }
            //thread::sleep(Duration::from_millis(100));
        } else {
            mouse::set_cursor_type(_ctx, MouseCursor::Crosshair);
            if mouse::cursor_hidden(_ctx) {
                mouse::set_cursor_hidden(_ctx, false);
            }
            if keyboard::is_key_pressed(_ctx, KeyCode::Space) {
                self.started = true;
            }

            if mouse::button_pressed(_ctx, MouseButton::Left) {
                const PIXEL_SIZE: u64 = 4;
                let mut buffer = self.image.to_rgba8(_ctx).expect("Failed to load buffer");
                let mouse_position = mouse::position(_ctx);
                let width = graphics::drawable_size(_ctx).0 as u64;
                let height = graphics::drawable_size(_ctx).1 as u64;
                if mouse_position.x < 0.0
                    || mouse_position.x >= width as f32
                    || mouse_position.y < 0.0
                    || mouse_position.y >= height as f32
                {
                    return Ok(());
                }
                let mut draw_indexes = vec![0, 0, 0, 0, 0];
                draw_indexes[0] = width * PIXEL_SIZE * mouse_position.y as u64
                    + mouse_position.x as u64 * PIXEL_SIZE
                    + 1;
                draw_indexes[1] = draw_indexes[0] - PIXEL_SIZE;
                draw_indexes[2] = draw_indexes[0] + PIXEL_SIZE;
                draw_indexes[3] = draw_indexes[0] - width * PIXEL_SIZE;
                draw_indexes[4] = draw_indexes[0] + width * PIXEL_SIZE;
                for index in draw_indexes {
                    if index < buffer.len() as u64 {
                        buffer[index as usize] = 255;
                    }
                }
                self.image = Image::from_rgba8(_ctx, width as u16, height as u16, &buffer)
                    .expect("Failed to update image");
            }
        }
        graphics::clear(_ctx, graphics::BLACK);
        graphics::draw(_ctx, &self.image, DrawParam::new()).expect("Draw failed!");
        graphics::present(_ctx)
    }
}
