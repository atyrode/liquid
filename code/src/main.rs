mod particle;
use particle::Particles;

/*
fn main() {
	loop {
	}
}
*/
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 1080;
const HEIGHT: usize = 1080;

use std::cmp::{min, max};

fn main() {
	let mut particles = Particles::new(2000, WIDTH as f64, HEIGHT as f64);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // Clear screen
        }



		for i in 0..particles.pos.len() {
			draw_square(&mut buffer, particles.pos[i].x as i32, HEIGHT as i32 - particles.pos[i].y as i32, 2);
		}
		particles.frame(1. / 60.);
		println!("{:?}", particles.pos[0]);



        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}

fn draw_square(buffer: &mut Vec<u32>, x: i32, y: i32, extent: i32) {
	for i in -extent..extent {
		for j in -extent..extent {
			let fx = min(WIDTH as i32 - 1, max(0, x + i)) as usize;
			let fy = min(HEIGHT as i32 - 1, max(0, y + j)) as usize;
			buffer[fy * WIDTH + fx] = 0x00AAFF;
		}
	}
}
