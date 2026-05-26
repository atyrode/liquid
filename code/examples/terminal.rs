#[path = "../src/particle.rs"]
mod particle;

use crossterm::terminal;
use particle::{Particles, Vec2f64};
use std::env;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

const WORLD_WIDTH: f64 = 1080.0;
const WORLD_HEIGHT: f64 = 1080.0;
const DENSITY_FULL: f32 = 3.0;
const PALETTE: &[u8] = b" .:-=+*#%@";

#[derive(Debug)]
struct Config {
    cols: usize,
    rows: usize,
    auto_size: bool,
    color: ColorTheme,
    gravity_spin: f64,
    particles: usize,
    fps: u64,
    frames: Option<usize>,
}

impl Config {
    fn from_args() -> Result<Self, String> {
        let mut config = Config {
            cols: 100,
            rows: 50,
            auto_size: false,
            color: ColorTheme::Blue,
            gravity_spin: 1.0,
            particles: 2_000,
            fps: 30,
            frames: None,
        };

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--cols" => config.cols = parse_next(&mut args, "--cols")?,
                "--rows" => config.rows = parse_next(&mut args, "--rows")?,
                "--auto-size" => config.auto_size = true,
                "--color" => {
                    config.color = ColorTheme::parse(&parse_next::<String>(&mut args, "--color")?)?
                }
                "--no-color" => config.color = ColorTheme::Mono,
                "--gravity-spin" => config.gravity_spin = parse_next(&mut args, "--gravity-spin")?,
                "--particles" => config.particles = parse_next(&mut args, "--particles")?,
                "--fps" => config.fps = parse_next(&mut args, "--fps")?,
                "--frames" => config.frames = Some(parse_next(&mut args, "--frames")?),
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => return Err(format!("unknown argument: {arg}")),
            }
        }

        if config.cols == 0 || config.rows == 0 {
            return Err("grid dimensions must be greater than zero".to_string());
        }
        if config.fps == 0 {
            return Err("fps must be greater than zero".to_string());
        }
        if !config.gravity_spin.is_finite() {
            return Err("gravity spin must be a finite number".to_string());
        }

        Ok(config)
    }

    fn grid_size(&self) -> GridSize {
        if self.auto_size {
            terminal_grid_size().unwrap_or(GridSize {
                cols: self.cols,
                rows: self.rows,
            })
        } else {
            GridSize {
                cols: self.cols,
                rows: self.rows,
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ColorTheme {
    Mono,
    Blue,
    Cyan,
    DeepBlue,
}

impl ColorTheme {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "mono" | "none" | "off" => Ok(Self::Mono),
            "blue" => Ok(Self::Blue),
            "cyan" => Ok(Self::Cyan),
            "deep-blue" => Ok(Self::DeepBlue),
            _ => Err(format!(
                "unknown color theme: {value}; expected mono, blue, cyan, or deep-blue"
            )),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct GridSize {
    cols: usize,
    rows: usize,
}

struct DensityGrid {
    cols: usize,
    rows: usize,
    cells: Vec<f32>,
}

impl DensityGrid {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            cells: vec![0.0; cols * rows],
        }
    }

    fn size(&self) -> GridSize {
        GridSize {
            cols: self.cols,
            rows: self.rows,
        }
    }

    fn rasterize(&mut self, particles: &[Vec2f64], world_width: f64, world_height: f64) {
        self.cells.fill(0.0);

        for particle in particles {
            let col = project_axis(particle.x, world_width, self.cols);
            let row = self.rows - 1 - project_axis(particle.y, world_height, self.rows);

            self.add_density(col, row, 1.0);
            self.add_density(col.saturating_sub(1), row, 0.25);
            self.add_density(col + 1, row, 0.25);
            self.add_density(col, row.saturating_sub(1), 0.25);
            self.add_density(col, row + 1, 0.25);
        }
    }

    fn add_density(&mut self, col: usize, row: usize, amount: f32) {
        if col < self.cols && row < self.rows {
            self.cells[row * self.cols + col] += amount;
        }
    }

    fn render(&self, frame: usize, config: &Config) -> String {
        let mut output = String::with_capacity((self.cols + 1) * (self.rows + 2) * 2);
        output.push_str("\x1b[0m\x1b[H");
        output.push_str(&format!(
            "fluid_sim terminal | {}x{} | particles {} | fps {} | gravity spin {:.2} | color {:?} | frame {} | Ctrl-C exits\n",
            self.cols,
            self.rows,
            config.particles,
            config.fps,
            config.gravity_spin,
            config.color,
            frame
        ));

        for row in 0..self.rows {
            let mut current_color = None;
            for col in 0..self.cols {
                let density = self.cells[row * self.cols + col];
                let color = cell_color(config.color, density);
                if color != current_color {
                    if let Some((red, green, blue)) = color {
                        output.push_str(&format!("\x1b[38;2;{red};{green};{blue}m"));
                    } else {
                        output.push_str("\x1b[0m");
                    }
                    current_color = color;
                }
                output.push(density_char(density));
            }
            output.push_str("\x1b[0m\n");
        }

        output
    }
}

fn main() -> Result<(), String> {
    let config = Config::from_args()?;
    let mut particles = Particles::new(config.particles, WORLD_WIDTH, WORLD_HEIGHT);
    let size = config.grid_size();
    let mut grid = DensityGrid::new(size.cols, size.rows);
    let delta = 1.0 / config.fps as f64;
    let frame_duration = Duration::from_secs_f64(delta);
    let mut stdout = io::stdout();
    let mut gravity_time = particles.time;

    stdout
        .write_all(b"\x1b[2J\x1b[H")
        .map_err(|err| err.to_string())?;

    let mut frame = 0;
    loop {
        let started_at = Instant::now();
        let size = config.grid_size();
        if grid.size() != size {
            grid = DensityGrid::new(size.cols, size.rows);
            stdout
                .write_all(b"\x1b[2J\x1b[H")
                .map_err(|err| err.to_string())?;
        }

        grid.rasterize(
            &particles.pos,
            particles.board_width,
            particles.board_height,
        );
        stdout
            .write_all(grid.render(frame, &config).as_bytes())
            .map_err(|err| err.to_string())?;
        stdout.flush().map_err(|err| err.to_string())?;

        gravity_time += delta * config.gravity_spin;
        particles.time = gravity_time - delta;
        particles.frame(delta);
        frame += 1;

        if config.frames.is_some_and(|limit| frame >= limit) {
            break;
        }

        if let Some(remaining) = frame_duration.checked_sub(started_at.elapsed()) {
            thread::sleep(remaining);
        }
    }

    Ok(())
}

fn parse_next<T>(args: &mut impl Iterator<Item = String>, name: &str) -> Result<T, String>
where
    T: std::str::FromStr,
{
    args.next()
        .ok_or_else(|| format!("{name} needs a value"))?
        .parse()
        .map_err(|_| format!("invalid value for {name}"))
}

fn project_axis(value: f64, world_size: f64, cells: usize) -> usize {
    if cells == 1 {
        return 0;
    }

    let normalized = (value / world_size).clamp(0.0, 1.0);
    (normalized * (cells - 1) as f64).round() as usize
}

fn density_char(density: f32) -> char {
    let intensity = (density / DENSITY_FULL).clamp(0.0, 1.0);
    let index = (intensity * (PALETTE.len() - 1) as f32).round() as usize;
    PALETTE[index] as char
}

fn cell_color(theme: ColorTheme, density: f32) -> Option<(u8, u8, u8)> {
    if theme == ColorTheme::Mono || density <= 0.0 {
        return None;
    }

    let intensity = (density / DENSITY_FULL).clamp(0.0, 1.0);
    let mix = |low: u8, high: u8| -> u8 {
        (low as f32 + (high as f32 - low as f32) * intensity).round() as u8
    };

    Some(match theme {
        ColorTheme::Mono => unreachable!(),
        ColorTheme::Blue => (mix(20, 110), mix(70, 190), mix(150, 255)),
        ColorTheme::Cyan => (mix(10, 90), mix(110, 245), mix(140, 255)),
        ColorTheme::DeepBlue => (mix(0, 55), mix(20, 110), mix(90, 230)),
    })
}

fn terminal_grid_size() -> Option<GridSize> {
    let (cols, rows) = terminal::size().ok()?;
    let cols = usize::from(cols).max(1);
    let rows = usize::from(rows).saturating_sub(1).max(1);

    Some(GridSize { cols, rows })
}

fn print_help() {
    println!(
        "Usage: cargo run --example terminal -- [OPTIONS]\n\
\n\
Options:\n\
  --cols N        Terminal grid columns [default: 100]\n\
  --rows N        Terminal grid rows [default: 50]\n\
  --auto-size     Use terminal size and adapt when resized\n\
  --color THEME   Color theme: blue, cyan, deep-blue, mono [default: blue]\n\
  --no-color      Alias for --color mono\n\
  --gravity-spin N\n\
                  Gravity rotation speed multiplier [default: 1.0]\n\
  --particles N   Particle count [default: 2000]\n\
  --fps N         Target frames per second [default: 30]\n\
  --frames N      Stop after N frames, useful for smoke tests\n\
  -h, --help      Show this help"
    );
}
