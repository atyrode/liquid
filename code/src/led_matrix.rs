use crate::particle::Particles;
use crate::raster::{DENSITY_FULL, DensityGrid};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use smart_leds::{RGB8, SmartLedsWrite, brightness, gamma};
use std::env;
use std::thread;
use std::time::{Duration, Instant};
use ws2812_spi::hosted::Ws2812;

const WORLD_WIDTH: f64 = 1080.0;
const WORLD_HEIGHT: f64 = 1080.0;

#[derive(Debug, Clone)]
struct Config {
    panel_width: usize,
    panel_height: usize,
    chain_cols: usize,
    chain_rows: usize,
    serpentine: bool,
    origin: Origin,
    color: LedColorTheme,
    brightness: u8,
    gravity_spin: f64,
    particles: usize,
    fps: u64,
    frames: Option<usize>,
    spi_hz: u32,
    test: bool,
    orbit_test: bool,
}

impl Config {
    fn defaults() -> Self {
        Self {
            panel_width: 8,
            panel_height: 8,
            chain_cols: 1,
            chain_rows: 1,
            serpentine: true,
            origin: Origin::TopLeft,
            color: LedColorTheme::DeepBlue,
            brightness: 16,
            gravity_spin: 1.0,
            particles: 500,
            fps: 30,
            frames: None,
            spi_hz: 3_000_000,
            test: false,
            orbit_test: false,
        }
    }

    fn from_args() -> Result<Self, String> {
        let mut config = Config {
            ..Config::defaults()
        };

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--panel-width" | "--width" => {
                    config.panel_width = parse_next(&mut args, &arg)?;
                }
                "--panel-height" | "--height" => {
                    config.panel_height = parse_next(&mut args, &arg)?;
                }
                "--chain-cols" => config.chain_cols = parse_next(&mut args, "--chain-cols")?,
                "--chain-rows" => config.chain_rows = parse_next(&mut args, "--chain-rows")?,
                "--serpentine" => config.serpentine = true,
                "--linear" => config.serpentine = false,
                "--origin" => {
                    config.origin = Origin::parse(&parse_next::<String>(&mut args, "--origin")?)?;
                }
                "--color" => {
                    config.color =
                        LedColorTheme::parse(&parse_next::<String>(&mut args, "--color")?)?;
                }
                "--brightness" => config.brightness = parse_next(&mut args, "--brightness")?,
                "--gravity-spin" => config.gravity_spin = parse_next(&mut args, "--gravity-spin")?,
                "--particles" => config.particles = parse_next(&mut args, "--particles")?,
                "--fps" => config.fps = parse_next(&mut args, "--fps")?,
                "--frames" => config.frames = Some(parse_next(&mut args, "--frames")?),
                "--spi-hz" => config.spi_hz = parse_next(&mut args, "--spi-hz")?,
                "--test" => config.test = true,
                "--orbit-test" => config.orbit_test = true,
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => return Err(format!("unknown argument: {arg}")),
            }
        }

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.panel_width == 0 || self.panel_height == 0 {
            return Err("panel dimensions must be greater than zero".to_string());
        }
        if self.chain_cols == 0 || self.chain_rows == 0 {
            return Err("chain dimensions must be greater than zero".to_string());
        }
        if self.matrix_width() == 0 || self.matrix_height() == 0 {
            return Err("matrix dimensions must be greater than zero".to_string());
        }
        if self.fps == 0 {
            return Err("fps must be greater than zero".to_string());
        }
        if !self.gravity_spin.is_finite() {
            return Err("gravity spin must be a finite number".to_string());
        }
        if !(2_000_000..=3_800_000).contains(&self.spi_hz) {
            return Err("spi hz must be between 2000000 and 3800000 for WS2812B".to_string());
        }

        Ok(())
    }

    fn matrix_width(&self) -> usize {
        self.panel_width * self.chain_cols
    }

    fn matrix_height(&self) -> usize {
        self.panel_height * self.chain_rows
    }

    fn layout(&self) -> MatrixLayout {
        MatrixLayout {
            width: self.matrix_width(),
            height: self.matrix_height(),
            serpentine: self.serpentine,
            origin: self.origin,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Origin {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Origin {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "top-left" | "tl" => Ok(Self::TopLeft),
            "top-right" | "tr" => Ok(Self::TopRight),
            "bottom-left" | "bl" => Ok(Self::BottomLeft),
            "bottom-right" | "br" => Ok(Self::BottomRight),
            _ => Err(format!(
                "unknown origin: {value}; expected top-left, top-right, bottom-left, or bottom-right"
            )),
        }
    }

    fn as_arg(self) -> &'static str {
        match self {
            Self::TopLeft => "top-left",
            Self::TopRight => "top-right",
            Self::BottomLeft => "bottom-left",
            Self::BottomRight => "bottom-right",
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum LedColorTheme {
    White,
    Blue,
    Cyan,
    DeepBlue,
}

impl LedColorTheme {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "white" | "mono" | "none" => Ok(Self::White),
            "blue" => Ok(Self::Blue),
            "cyan" => Ok(Self::Cyan),
            "deep-blue" => Ok(Self::DeepBlue),
            _ => Err(format!(
                "unknown color theme: {value}; expected white, blue, cyan, or deep-blue"
            )),
        }
    }

    fn as_arg(self) -> &'static str {
        match self {
            Self::White => "white",
            Self::Blue => "blue",
            Self::Cyan => "cyan",
            Self::DeepBlue => "deep-blue",
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct MatrixLayout {
    width: usize,
    height: usize,
    serpentine: bool,
    origin: Origin,
}

impl MatrixLayout {
    fn pixel_count(self) -> usize {
        self.width * self.height
    }

    fn pixel_index(self, x: usize, y: usize) -> usize {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        let (origin_x, origin_y) = match self.origin {
            Origin::TopLeft => (x, y),
            Origin::TopRight => (self.width - 1 - x, y),
            Origin::BottomLeft => (x, self.height - 1 - y),
            Origin::BottomRight => (self.width - 1 - x, self.height - 1 - y),
        };

        let row_start = origin_y * self.width;
        if self.serpentine && origin_y % 2 == 1 {
            row_start + (self.width - 1 - origin_x)
        } else {
            row_start + origin_x
        }
    }
}

pub fn run_from_env() -> Result<(), String> {
    let config = Config::from_args()?;
    if config.orbit_test {
        run_orbit_test(&config)
    } else if config.test {
        run_hardware_test(&config)
    } else {
        run_simulation(&config)
    }
}

fn run_hardware_test(config: &Config) -> Result<(), String> {
    let layout = config.layout();
    let mut output = LedOutput::new(layout, config.spi_hz)?;

    println!(
        "Testing {}x{} WS2812B matrix, brightness {}, origin {}, {} wiring.",
        layout.width,
        layout.height,
        config.brightness,
        config.origin.as_arg(),
        if config.serpentine {
            "serpentine"
        } else {
            "linear"
        }
    );

    for color in [
        RGB8 { r: 255, g: 0, b: 0 },
        RGB8 { r: 0, g: 255, b: 0 },
        RGB8 { r: 0, g: 0, b: 255 },
    ] {
        output.fill(color);
        output.write(config.brightness)?;
        thread::sleep(Duration::from_millis(600));
    }

    for index in 0..layout.pixel_count() {
        output.clear_frame();
        output.frame[index] = RGB8 {
            r: 255,
            g: 255,
            b: 255,
        };
        output.write(config.brightness)?;
        thread::sleep(Duration::from_millis(80));
    }

    for y in 0..layout.height {
        output.clear_frame();
        for x in 0..layout.width {
            let index = layout.pixel_index(x, y);
            output.frame[index] = RGB8 { r: 0, g: 0, b: 255 };
        }
        output.write(config.brightness)?;
        thread::sleep(Duration::from_millis(200));
    }

    output.clear()
}

fn run_orbit_test(config: &Config) -> Result<(), String> {
    let layout = config.layout();
    let mut output = LedOutput::new(layout, config.spi_hz)?;
    let delta = 1.0 / config.fps as f64;
    let frame_duration = Duration::from_secs_f64(delta);
    let mut progress = 0.0_f64;

    println!(
        "Running green orbit test on {}x{} WS2812B matrix, brightness {}, origin {}, {} wiring.",
        layout.width,
        layout.height,
        config.brightness,
        config.origin.as_arg(),
        if config.serpentine {
            "serpentine"
        } else {
            "linear"
        }
    );

    let mut frame = 0;
    loop {
        let started_at = Instant::now();
        output.render_orbit(progress);
        output.write(config.brightness)?;

        progress = (progress + 0.01) % 1.0;
        frame += 1;

        if config.frames.is_some_and(|limit| frame >= limit) {
            break;
        }

        if let Some(remaining) = frame_duration.checked_sub(started_at.elapsed()) {
            thread::sleep(remaining);
        }
    }

    output.clear()
}

fn run_simulation(config: &Config) -> Result<(), String> {
    let layout = config.layout();
    let mut output = LedOutput::new(layout, config.spi_hz)?;
    let mut particles = Particles::new(config.particles, WORLD_WIDTH, WORLD_HEIGHT);
    let mut grid = DensityGrid::new(layout.width, layout.height);
    let delta = 1.0 / config.fps as f64;
    let frame_duration = Duration::from_secs_f64(delta);
    let mut gravity_time = particles.time;

    println!(
        "Rendering Liquid to {}x{} WS2812B matrix, color {}, brightness {}, {} FPS.",
        layout.width,
        layout.height,
        config.color.as_arg(),
        config.brightness,
        config.fps
    );

    let mut frame = 0;
    loop {
        let started_at = Instant::now();
        grid.rasterize(
            &particles.pos,
            particles.board_width,
            particles.board_height,
        );
        output.render_density(&grid, config.color);
        output.write(config.brightness)?;

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

struct LedOutput {
    writer: Ws2812<Spi>,
    layout: MatrixLayout,
    frame: Vec<RGB8>,
}

impl LedOutput {
    fn new(layout: MatrixLayout, spi_hz: u32) -> Result<Self, String> {
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, spi_hz, Mode::Mode0)
            .map_err(|err| format!("could not open SPI0 SS0: {err}"))?;
        let writer = Ws2812::new(spi);

        Ok(Self {
            writer,
            layout,
            frame: vec![RGB8 { r: 0, g: 0, b: 0 }; layout.pixel_count()],
        })
    }

    fn fill(&mut self, color: RGB8) {
        self.frame.fill(color);
    }

    fn clear_frame(&mut self) {
        self.fill(RGB8 { r: 0, g: 0, b: 0 });
    }

    fn clear(&mut self) -> Result<(), String> {
        self.clear_frame();
        self.write(0)
    }

    fn render_density(&mut self, grid: &DensityGrid, theme: LedColorTheme) {
        for y in 0..self.layout.height {
            for x in 0..self.layout.width {
                let density = grid.cells()[y * self.layout.width + x];
                let index = self.layout.pixel_index(x, y);
                self.frame[index] = density_color(theme, density);
            }
        }
    }

    fn render_orbit(&mut self, progress: f64) {
        self.clear_frame();

        let angle = 2.0 * std::f64::consts::PI * progress;
        let board_x = (angle.cos() + 1.0) * (self.layout.width - 1) as f64 / 2.0;
        let board_y = (angle.sin() + 1.0) * (self.layout.height - 1) as f64 / 2.0;

        for y in [board_y.floor(), board_y.ceil()] {
            for x in [board_x.floor(), board_x.ceil()] {
                self.add_orbit_sample(board_x, board_y, x as isize, y as isize);
            }
        }
    }

    fn add_orbit_sample(&mut self, board_x: f64, board_y: f64, x: isize, y: isize) {
        if x < 0 || y < 0 || x as usize >= self.layout.width || y as usize >= self.layout.height {
            return;
        }

        let distance = ((board_x - x as f64).powi(2) + (board_y - y as f64).powi(2)).sqrt();
        let brightness = (0.75 - distance).clamp(0.0, 1.0);
        if brightness <= 0.0 {
            return;
        }

        let index = self.layout.pixel_index(x as usize, y as usize);
        let green = (brightness * 255.0).round() as u8;
        self.frame[index].g = self.frame[index].g.max(green);
    }

    fn write(&mut self, brightness_value: u8) -> Result<(), String> {
        self.writer
            .write(brightness(
                gamma(self.frame.iter().copied()),
                brightness_value,
            ))
            .map_err(|err| format!("could not write LED frame: {err}"))
    }
}

fn density_color(theme: LedColorTheme, density: f32) -> RGB8 {
    if density <= 0.0 {
        return RGB8 { r: 0, g: 0, b: 0 };
    }

    let intensity = (density / DENSITY_FULL).clamp(0.0, 1.0);
    let scaled = (intensity * 255.0).round() as u8;
    match theme {
        LedColorTheme::White => RGB8 {
            r: scaled,
            g: scaled,
            b: scaled,
        },
        LedColorTheme::Blue => RGB8 {
            r: 0,
            g: (intensity * 120.0).round() as u8,
            b: scaled,
        },
        LedColorTheme::Cyan => RGB8 {
            r: 0,
            g: scaled,
            b: (intensity * 220.0).round() as u8,
        },
        LedColorTheme::DeepBlue => RGB8 {
            r: 0,
            g: (intensity * 55.0).round() as u8,
            b: (intensity * 180.0).round() as u8,
        },
    }
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

fn print_help() {
    println!(
        "Usage: cargo run --release --no-default-features --features led-matrix --example led_matrix -- [OPTIONS]\n\
\n\
Options:\n\
  --test              Run RGB fill, physical chase, and row tests\n\
  --orbit-test        Run the green rotating-pixel Arduino parity test\n\
  --panel-width N     Width of one panel [default: 8]\n\
  --panel-height N    Height of one panel [default: 8]\n\
  --chain-cols N      Number of panels chained horizontally [default: 1]\n\
  --chain-rows N      Number of panels chained vertically [default: 1]\n\
  --serpentine        Use zig-zag wiring order [default]\n\
  --linear            Use left-to-right wiring on every row\n\
  --origin NAME       First LED corner: top-left, top-right, bottom-left, bottom-right [default: top-left]\n\
  --color THEME       Color theme: deep-blue, blue, cyan, white [default: deep-blue]\n\
  --brightness N      Smart-leds brightness limit, 0-255 [default: 16]\n\
  --spi-hz N          SPI clock in Hz, 2000000-3800000 [default: 3000000]\n\
  --gravity-spin N    Gravity rotation speed multiplier [default: 1.0]\n\
  --particles N       Particle count [default: 500]\n\
  --fps N             Target frames per second [default: 30]\n\
  --frames N          Stop after N frames, useful for smoke tests\n\
  -h, --help          Show this help"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serpentine_top_left_mapping_alternates_rows() {
        let layout = MatrixLayout {
            width: 4,
            height: 2,
            serpentine: true,
            origin: Origin::TopLeft,
        };

        assert_eq!(layout.pixel_index(0, 0), 0);
        assert_eq!(layout.pixel_index(3, 0), 3);
        assert_eq!(layout.pixel_index(0, 1), 7);
        assert_eq!(layout.pixel_index(3, 1), 4);
    }

    #[test]
    fn origin_flips_display_coordinates_before_serpentine_mapping() {
        let layout = MatrixLayout {
            width: 4,
            height: 2,
            serpentine: true,
            origin: Origin::BottomRight,
        };

        assert_eq!(layout.pixel_index(3, 1), 0);
        assert_eq!(layout.pixel_index(0, 1), 3);
        assert_eq!(layout.pixel_index(3, 0), 7);
        assert_eq!(layout.pixel_index(0, 0), 4);
    }

    #[test]
    fn density_color_clamps_to_theme_range() {
        assert_eq!(density_color(LedColorTheme::DeepBlue, 0.0).b, 0);
        assert_eq!(density_color(LedColorTheme::DeepBlue, DENSITY_FULL).b, 180);
        assert_eq!(
            density_color(LedColorTheme::White, DENSITY_FULL * 2.0).r,
            255
        );
    }
}
