use crate::particle::Particles;
use crate::raster::{DENSITY_FULL, DensityGrid, GridSize};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{
        self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

const WORLD_WIDTH: f64 = 1080.0;
const WORLD_HEIGHT: f64 = 1080.0;
const CLASSIC_CHARSET: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];
const DOT_CHARSET: &[char] = &[
    ' ', '.', '.', ':', ':', '\u{00b7}', '\u{00b7}', '\u{2022}', '\u{2022}', '\u{25cf}',
];
const BLOCK_CHARSET: &[char] = &[
    ' ', '\u{2591}', '\u{2591}', '\u{2592}', '\u{2592}', '\u{2593}', '\u{2593}', '\u{2588}',
    '\u{2588}', '\u{2588}',
];
const SOLID_BLOCK: char = '\u{2588}';
const TERMINAL_SETTING_KEYS: &[&str] = &[
    "LIQUID_COLS",
    "LIQUID_ROWS",
    "LIQUID_AUTO_SIZE",
    "LIQUID_COLOR",
    "LIQUID_CHARSET",
    "LIQUID_GRAVITY_SPIN",
    "LIQUID_PARTICLES",
    "LIQUID_FPS",
    "LIQUID_FRAMES",
    "LIQUID_STATUS",
];

#[derive(Debug, Clone)]
struct Config {
    cols: usize,
    rows: usize,
    auto_size: bool,
    color: ColorTheme,
    charset: Charset,
    gravity_spin: f64,
    particles: usize,
    fps: u64,
    frames: Option<usize>,
    show_status: bool,
}

impl Config {
    fn defaults() -> Self {
        Self {
            cols: 100,
            rows: 50,
            auto_size: false,
            color: ColorTheme::Blue,
            charset: Charset::Classic,
            gravity_spin: 1.0,
            particles: 2_000,
            fps: 60,
            frames: None,
            show_status: false,
        }
    }

    fn from_args() -> Result<(Self, bool), String> {
        let raw_args: Vec<String> = env::args().skip(1).collect();
        let mut setup = raw_args.is_empty();
        let mut config = Config {
            ..Config::defaults()
        };

        config.apply_settings_file()?;
        config.apply_env()?;

        let mut args = raw_args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--cols" => config.cols = parse_next(&mut args, "--cols")?,
                "--rows" => config.rows = parse_next(&mut args, "--rows")?,
                "--auto-size" => config.auto_size = true,
                "--fixed-size" => config.auto_size = false,
                "--color" => {
                    config.color = ColorTheme::parse(&parse_next::<String>(&mut args, "--color")?)?
                }
                "--no-color" => config.color = ColorTheme::Mono,
                "--charset" => {
                    config.charset = Charset::parse(&parse_next::<String>(&mut args, "--charset")?)?
                }
                "--gravity-spin" => config.gravity_spin = parse_next(&mut args, "--gravity-spin")?,
                "--particles" => config.particles = parse_next(&mut args, "--particles")?,
                "--fps" => config.fps = parse_next(&mut args, "--fps")?,
                "--frames" => config.frames = Some(parse_next(&mut args, "--frames")?),
                "--setup" => setup = true,
                "--status" => config.show_status = true,
                "--no-status" => config.show_status = false,
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => return Err(format!("unknown argument: {arg}")),
            }
        }

        config.validate()?;

        Ok((config, setup))
    }

    fn apply_settings_file(&mut self) -> Result<(), String> {
        let Some(path) = settings_path() else {
            return Ok(());
        };

        let Ok(contents) = fs::read_to_string(path) else {
            return Ok(());
        };

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            self.apply_setting(key.trim(), unquote(value.trim()))?;
        }

        Ok(())
    }

    fn apply_env(&mut self) -> Result<(), String> {
        for key in TERMINAL_SETTING_KEYS {
            if let Ok(value) = env::var(key) {
                self.apply_setting(key, value.as_str())?;
            }
        }

        Ok(())
    }

    fn apply_setting(&mut self, key: &str, value: &str) -> Result<(), String> {
        match key {
            "LIQUID_COLS" => self.cols = parse_setting(value, key)?,
            "LIQUID_ROWS" => self.rows = parse_setting(value, key)?,
            "LIQUID_AUTO_SIZE" => self.auto_size = parse_bool(value, key)?,
            "LIQUID_COLOR" => self.color = ColorTheme::parse(value)?,
            "LIQUID_CHARSET" => self.charset = Charset::parse(value)?,
            "LIQUID_GRAVITY_SPIN" => self.gravity_spin = parse_setting(value, key)?,
            "LIQUID_PARTICLES" => self.particles = parse_setting(value, key)?,
            "LIQUID_FPS" => self.fps = parse_setting(value, key)?,
            "LIQUID_STATUS" => self.show_status = parse_bool(value, key)?,
            "LIQUID_FRAMES" => {
                self.frames = if value.is_empty() {
                    None
                } else {
                    Some(parse_setting(value, key)?)
                };
            }
            _ => {}
        }

        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        if self.cols == 0 || self.rows == 0 {
            return Err("grid dimensions must be greater than zero".to_string());
        }
        if self.fps == 0 {
            return Err("fps must be greater than zero".to_string());
        }
        if !self.gravity_spin.is_finite() {
            return Err("gravity spin must be a finite number".to_string());
        }

        Ok(())
    }

    fn grid_size(&self) -> GridSize {
        if self.auto_size {
            terminal_grid_size(self.show_status).unwrap_or(GridSize {
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
enum Charset {
    Classic,
    Dots,
    Blocks,
    Solid,
}

impl Charset {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "classic" | "ascii" => Ok(Self::Classic),
            "dots" | "dot" => Ok(Self::Dots),
            "blocks" | "block" | "squares" | "square" => Ok(Self::Blocks),
            "solid" | "full" | "full-block" => Ok(Self::Solid),
            _ => Err(format!(
                "unknown charset: {value}; expected classic, dots, blocks, or solid"
            )),
        }
    }

    fn as_arg(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Dots => "dots",
            Self::Blocks => "blocks",
            Self::Solid => "solid",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Dots => "Dots",
            Self::Blocks => "Blocks",
            Self::Solid => "Solid block",
        }
    }

    fn chars(self) -> &'static [char] {
        match self {
            Self::Classic => CLASSIC_CHARSET,
            Self::Dots => DOT_CHARSET,
            Self::Blocks => BLOCK_CHARSET,
            Self::Solid => &[SOLID_BLOCK],
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Classic => Self::Dots,
            Self::Dots => Self::Blocks,
            Self::Blocks => Self::Solid,
            Self::Solid => Self::Classic,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Classic => Self::Solid,
            Self::Dots => Self::Classic,
            Self::Blocks => Self::Dots,
            Self::Solid => Self::Blocks,
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

    fn as_arg(self) -> &'static str {
        match self {
            Self::Mono => "mono",
            Self::Blue => "blue",
            Self::Cyan => "cyan",
            Self::DeepBlue => "deep-blue",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Mono => "Mono",
            Self::Blue => "Blue",
            Self::Cyan => "Cyan",
            Self::DeepBlue => "Deep blue",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Mono => Self::Blue,
            Self::Blue => Self::Cyan,
            Self::Cyan => Self::DeepBlue,
            Self::DeepBlue => Self::Mono,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Mono => Self::DeepBlue,
            Self::Blue => Self::Mono,
            Self::Cyan => Self::Blue,
            Self::DeepBlue => Self::Cyan,
        }
    }
}

impl DensityGrid {
    fn render(&self, frame: usize, config: &Config) -> String {
        let status_rows = usize::from(config.show_status);
        let mut output = String::with_capacity((self.cols + 1) * (self.rows + status_rows) * 2);
        output.push_str("\x1b[0m");
        if config.show_status {
            output.push_str(&format!(
                "fluid_sim terminal | {}x{} | particles {} | fps {} | gravity spin {:.2} | color {} | charset {} | frame {} | Q/Esc exits\x1b[K\r\n",
                self.cols,
                self.rows,
                config.particles,
                config.fps,
                config.gravity_spin,
                config.color.as_arg(),
                config.charset.as_arg(),
                frame
            ));
        }

        for row in 0..self.rows {
            let mut current_color = None;
            for col in 0..self.cols {
                let density = self.cells[row * self.cols + col];
                let color = cell_color(config.color, density);
                if color != current_color {
                    if let Some(color) = color {
                        output.push_str(color.sgr());
                    } else {
                        output.push_str("\x1b[0m");
                    }
                    current_color = color;
                }
                output.push(density_char(density, config.charset));
            }
            output.push_str("\x1b[0m\x1b[K");
            if row + 1 < self.rows {
                output.push_str("\r\n");
            }
        }

        output
    }

    fn write_frame(
        &self,
        stdout: &mut io::Stdout,
        frame: usize,
        config: &Config,
    ) -> Result<(), String> {
        queue!(stdout, ResetColor, SetAttribute(Attribute::Reset))
            .map_err(|err| err.to_string())?;

        let mut row_offset = 0;
        if config.show_status {
            queue!(
                stdout,
                MoveTo(0, 0),
                Print(format!(
                    "fluid_sim terminal | {}x{} | particles {} | fps {} | gravity spin {:.2} | color {} | charset {} | frame {} | Q/Esc exits",
                    self.cols,
                    self.rows,
                    config.particles,
                    config.fps,
                    config.gravity_spin,
                    config.color.as_arg(),
                    config.charset.as_arg(),
                    frame
                )),
                Clear(ClearType::UntilNewLine)
            )
            .map_err(|err| err.to_string())?;
            row_offset = 1;
        }

        for row in 0..self.rows {
            queue!(stdout, MoveTo(0, (row + row_offset) as u16)).map_err(|err| err.to_string())?;
            let mut current_color = None;
            for col in 0..self.cols {
                let density = self.cells[row * self.cols + col];
                let color = cell_color(config.color, density);
                if color != current_color {
                    if let Some(color) = color {
                        queue!(stdout, SetForegroundColor(color.crossterm()))
                            .map_err(|err| err.to_string())?;
                    } else {
                        queue!(stdout, ResetColor).map_err(|err| err.to_string())?;
                    }
                    current_color = color;
                }
                queue!(stdout, Print(density_char(density, config.charset)))
                    .map_err(|err| err.to_string())?;
            }
            queue!(stdout, ResetColor, Clear(ClearType::UntilNewLine))
                .map_err(|err| err.to_string())?;
        }

        Ok(())
    }
}

pub fn run_from_env() -> Result<(), String> {
    let (mut config, setup) = Config::from_args()?;
    if setup {
        let Some(setup_config) = run_setup(config.clone())? else {
            return Ok(());
        };
        config = setup_config;
    }

    let mut particles = Particles::new(config.particles, WORLD_WIDTH, WORLD_HEIGHT);
    let size = config.grid_size();
    let mut grid = DensityGrid::new(size.cols, size.rows);
    let delta = 1.0 / config.fps as f64;
    let frame_duration = Duration::from_secs_f64(delta);
    let render_guard = RenderTerminalGuard::enter()?;
    let mut stdout = io::stdout();
    let mut gravity_time = particles.time;

    let mut frame = 0;
    loop {
        let started_at = Instant::now();
        let size = config.grid_size();
        if grid.size() != size {
            grid = DensityGrid::new(size.cols, size.rows);
            queue!(stdout, Clear(ClearType::All), MoveTo(0, 0)).map_err(|err| err.to_string())?;
        }

        grid.rasterize(
            &particles.pos,
            particles.board_width,
            particles.board_height,
        );
        if render_guard.active {
            grid.write_frame(&mut stdout, frame, &config)?;
        } else {
            stdout
                .write_all(grid.render(frame, &config).as_bytes())
                .map_err(|err| err.to_string())?;
        }
        stdout.flush().map_err(|err| err.to_string())?;

        gravity_time += delta * config.gravity_spin;
        particles.time = gravity_time - delta;
        particles.frame(delta);
        frame += 1;

        if config.frames.is_some_and(|limit| frame >= limit) {
            break;
        }

        if render_guard.active && should_exit_render()? {
            break;
        }

        if let Some(remaining) = frame_duration.checked_sub(started_at.elapsed()) {
            thread::sleep(remaining);
        }
    }

    Ok(())
}

struct RenderTerminalGuard {
    active: bool,
}

impl RenderTerminalGuard {
    fn enter() -> Result<Self, String> {
        if !io::stdout().is_terminal() {
            return Ok(Self { active: false });
        }

        terminal::enable_raw_mode().map_err(|err| err.to_string())?;
        if let Err(err) = execute!(
            io::stdout(),
            EnterAlternateScreen,
            DisableLineWrap,
            Hide,
            Clear(ClearType::All),
            MoveTo(0, 0)
        ) {
            let _ = terminal::disable_raw_mode();
            return Err(err.to_string());
        }

        Ok(Self { active: true })
    }
}

impl Drop for RenderTerminalGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = execute!(
                io::stdout(),
                ResetColor,
                SetAttribute(Attribute::Reset),
                Show,
                EnableLineWrap,
                LeaveAlternateScreen
            );
            let _ = terminal::disable_raw_mode();
        }
    }
}

fn should_exit_render() -> Result<bool, String> {
    while event::poll(Duration::from_millis(0)).map_err(|err| err.to_string())? {
        let Event::Key(key) = event::read().map_err(|err| err.to_string())? else {
            continue;
        };

        if matches!(
            key.code,
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q')
        ) || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            return Ok(true);
        }
    }

    Ok(false)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum SetupItem {
    Start,
    Particles,
    Fps,
    Color,
    Charset,
    GravitySpin,
    Status,
    AutoSize,
    Cols,
    Rows,
    SaveStart,
    Quit,
}

const SETUP_ITEMS: &[SetupItem] = &[
    SetupItem::Start,
    SetupItem::Particles,
    SetupItem::Fps,
    SetupItem::Color,
    SetupItem::Charset,
    SetupItem::GravitySpin,
    SetupItem::Status,
    SetupItem::AutoSize,
    SetupItem::Cols,
    SetupItem::Rows,
    SetupItem::SaveStart,
    SetupItem::Quit,
];

struct SetupTerminalGuard;

impl SetupTerminalGuard {
    fn enter() -> Result<Self, String> {
        if !io::stdout().is_terminal() {
            return Err("setup screen needs an interactive terminal".to_string());
        }

        terminal::enable_raw_mode().map_err(|err| err.to_string())?;
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            Hide,
            Clear(ClearType::All)
        )
        .map_err(|err| err.to_string())?;

        Ok(Self)
    }
}

impl Drop for SetupTerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            ResetColor,
            SetAttribute(Attribute::Reset),
            Show,
            LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
    }
}

fn run_setup(mut config: Config) -> Result<Option<Config>, String> {
    let _guard = SetupTerminalGuard::enter()?;
    let mut stdout = io::stdout();
    let mut selected = 0;

    loop {
        render_setup(&mut stdout, &config, selected)?;

        let Event::Key(key) = event::read().map_err(|err| err.to_string())? else {
            continue;
        };

        match key.code {
            KeyCode::Up => selected = selected.saturating_sub(1),
            KeyCode::Down => selected = (selected + 1).min(SETUP_ITEMS.len() - 1),
            KeyCode::Left | KeyCode::Char('-') => {
                adjust_setup_value(&mut config, SETUP_ITEMS[selected], -1)
            }
            KeyCode::Right | KeyCode::Char('+') => {
                adjust_setup_value(&mut config, SETUP_ITEMS[selected], 1)
            }
            KeyCode::Enter => match SETUP_ITEMS[selected] {
                SetupItem::Start => return Ok(Some(config)),
                SetupItem::SaveStart => {
                    save_settings(&config)?;
                    return Ok(Some(config));
                }
                SetupItem::Quit => return Ok(None),
                SetupItem::Color | SetupItem::Charset | SetupItem::Status | SetupItem::AutoSize => {
                    adjust_setup_value(&mut config, SETUP_ITEMS[selected], 1);
                }
                _ => {}
            },
            KeyCode::Char('s') | KeyCode::Char('S') => {
                save_settings(&config)?;
                return Ok(Some(config));
            }
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return Ok(None),
            _ => {}
        }
    }
}

fn render_setup(stdout: &mut io::Stdout, config: &Config, selected: usize) -> Result<(), String> {
    queue!(
        stdout,
        MoveTo(0, 0),
        Clear(ClearType::All),
        SetForegroundColor(Color::Cyan),
        Print("Liquid terminal renderer\n"),
        ResetColor,
        Print("Enter starts. Up/down moves. Left/right changes values. S saves and starts. Q quits.\n\n")
    )
    .map_err(|err| err.to_string())?;

    for (index, item) in SETUP_ITEMS.iter().copied().enumerate() {
        render_setup_row(stdout, index, item, config, index == selected)?;
    }

    stdout.flush().map_err(|err| err.to_string())
}

fn render_setup_row(
    stdout: &mut io::Stdout,
    index: usize,
    item: SetupItem,
    config: &Config,
    selected: bool,
) -> Result<(), String> {
    if selected {
        queue!(stdout, SetAttribute(Attribute::Reverse)).map_err(|err| err.to_string())?;
    }

    let label = match item {
        SetupItem::Start => "Start",
        SetupItem::Particles => "Particles",
        SetupItem::Fps => "FPS",
        SetupItem::Color => "Color",
        SetupItem::Charset => "Character set",
        SetupItem::GravitySpin => "Gravity spin",
        SetupItem::Status => "Status line",
        SetupItem::AutoSize => "Auto size",
        SetupItem::Cols => "Columns",
        SetupItem::Rows => "Rows",
        SetupItem::SaveStart => "Save + start",
        SetupItem::Quit => "Quit",
    };

    let value = match item {
        SetupItem::Start => "launch now".to_string(),
        SetupItem::Particles => config.particles.to_string(),
        SetupItem::Fps => config.fps.to_string(),
        SetupItem::Color => config.color.label().to_string(),
        SetupItem::Charset => config.charset.label().to_string(),
        SetupItem::GravitySpin => format!("{:.1}", config.gravity_spin),
        SetupItem::Status => {
            if config.show_status {
                "on".to_string()
            } else {
                "off".to_string()
            }
        }
        SetupItem::AutoSize => {
            if config.auto_size {
                "on".to_string()
            } else {
                "off".to_string()
            }
        }
        SetupItem::Cols => {
            if config.auto_size {
                format!("{} (fixed mode)", config.cols)
            } else {
                config.cols.to_string()
            }
        }
        SetupItem::Rows => {
            if config.auto_size {
                format!("{} (fixed mode)", config.rows)
            } else {
                config.rows.to_string()
            }
        }
        SetupItem::SaveStart => settings_path()
            .map(|path| format!("write {}", path.display()))
            .unwrap_or_else(|| "settings path unavailable".to_string()),
        SetupItem::Quit => "exit without starting".to_string(),
    };

    queue!(
        stdout,
        MoveTo(0, (index + 3) as u16),
        Print(format!("{label:<14} {value:<60}")),
        SetAttribute(Attribute::Reset),
        Print("\n")
    )
    .map_err(|err| err.to_string())
}

fn adjust_setup_value(config: &mut Config, item: SetupItem, direction: i32) {
    match item {
        SetupItem::Particles => {
            config.particles = adjust_usize(config.particles, direction, 100, 20_000, 100);
        }
        SetupItem::Fps => {
            config.fps = adjust_u64(config.fps, direction, 1, 120, 1);
        }
        SetupItem::Color => {
            config.color = if direction >= 0 {
                config.color.next()
            } else {
                config.color.previous()
            };
        }
        SetupItem::Charset => {
            config.charset = if direction >= 0 {
                config.charset.next()
            } else {
                config.charset.previous()
            };
        }
        SetupItem::GravitySpin => {
            config.gravity_spin = (config.gravity_spin + direction as f64 * 0.1).clamp(-10.0, 10.0);
        }
        SetupItem::Status => config.show_status = !config.show_status,
        SetupItem::AutoSize => config.auto_size = !config.auto_size,
        SetupItem::Cols => {
            config.cols = adjust_usize(config.cols, direction, 1, 300, 5);
        }
        SetupItem::Rows => {
            config.rows = adjust_usize(config.rows, direction, 1, 200, 2);
        }
        SetupItem::Start | SetupItem::SaveStart | SetupItem::Quit => {}
    }
}

fn adjust_usize(value: usize, direction: i32, min: usize, max: usize, step: usize) -> usize {
    if direction >= 0 {
        value.saturating_add(step).min(max)
    } else {
        value.saturating_sub(step).max(min)
    }
}

fn adjust_u64(value: u64, direction: i32, min: u64, max: u64, step: u64) -> u64 {
    if direction >= 0 {
        value.saturating_add(step).min(max)
    } else {
        value.saturating_sub(step).max(min)
    }
}

fn save_settings(config: &Config) -> Result<(), String> {
    let path = settings_path().ok_or_else(|| "could not determine settings path".to_string())?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let mut contents = terminal_settings_contents(config);
    let preserved_led_settings = preserved_led_settings(&path)?;
    if !preserved_led_settings.is_empty() {
        contents.push_str("\n# LED matrix settings preserved by terminal setup.\n");
        for line in preserved_led_settings {
            contents.push_str(&line);
            contents.push('\n');
        }
    }

    fs::write(path, contents).map_err(|err| err.to_string())
}

fn terminal_settings_contents(config: &Config) -> String {
    format!(
        "\
# Local Liquid renderer settings.\n\
# This file is written by `liquid setup` and ignored by git.\n\
\n\
LIQUID_PARTICLES={}\n\
LIQUID_FPS={}\n\
LIQUID_COLOR={}\n\
LIQUID_CHARSET={}\n\
LIQUID_GRAVITY_SPIN={:.1}\n\
LIQUID_STATUS={}\n\
LIQUID_AUTO_SIZE={}\n\
LIQUID_COLS={}\n\
LIQUID_ROWS={}\n",
        config.particles,
        config.fps,
        config.color.as_arg(),
        config.charset.as_arg(),
        config.gravity_spin,
        if config.show_status { 1 } else { 0 },
        if config.auto_size { 1 } else { 0 },
        config.cols,
        config.rows
    )
}

fn preserved_led_settings(path: &Path) -> Result<Vec<String>, String> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Ok(Vec::new());
    };

    Ok(contents
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let (key, _) = trimmed.split_once('=')?;
            if key.trim().starts_with("LIQUID_LED_") {
                Some(trimmed.to_string())
            } else {
                None
            }
        })
        .collect())
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

fn parse_setting<T>(value: &str, name: &str) -> Result<T, String>
where
    T: std::str::FromStr,
{
    value
        .parse()
        .map_err(|_| format!("invalid value for {name}"))
}

fn parse_bool(value: &str, name: &str) -> Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(format!("invalid boolean value for {name}")),
    }
}

fn unquote(value: &str) -> &str {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return &value[1..value.len() - 1];
        }
    }

    value
}

fn settings_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("LIQUID_CONFIG") {
        return Some(PathBuf::from(path));
    }

    env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join("liquid/.liquid/settings.env"))
}

fn density_char(density: f32, charset: Charset) -> char {
    if density <= 0.0 {
        return ' ';
    }
    if charset == Charset::Solid {
        return SOLID_BLOCK;
    }

    let intensity = (density / DENSITY_FULL).clamp(0.0, 1.0);
    let chars = charset.chars();
    let index = (intensity * (chars.len() - 1) as f32).round() as usize;
    chars[index]
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CellColor {
    DarkBlue,
    Blue,
    DarkCyan,
    Cyan,
    White,
}

impl CellColor {
    fn sgr(self) -> &'static str {
        match self {
            Self::DarkBlue => "\x1b[34m",
            Self::Blue => "\x1b[94m",
            Self::DarkCyan => "\x1b[36m",
            Self::Cyan => "\x1b[96m",
            Self::White => "\x1b[97m",
        }
    }

    fn crossterm(self) -> Color {
        match self {
            Self::DarkBlue => Color::DarkBlue,
            Self::Blue => Color::Blue,
            Self::DarkCyan => Color::DarkCyan,
            Self::Cyan => Color::Cyan,
            Self::White => Color::White,
        }
    }
}

fn cell_color(theme: ColorTheme, density: f32) -> Option<CellColor> {
    if theme == ColorTheme::Mono || density <= 0.0 {
        return None;
    }

    let intensity = (density / DENSITY_FULL).clamp(0.0, 1.0);
    let band = (intensity * 3.0).ceil() as usize;

    Some(match theme {
        ColorTheme::Mono => unreachable!(),
        ColorTheme::Blue => match band {
            0 | 1 => CellColor::DarkBlue,
            2 => CellColor::Blue,
            _ => CellColor::Cyan,
        },
        ColorTheme::Cyan => match band {
            0 | 1 => CellColor::DarkCyan,
            2 => CellColor::Cyan,
            _ => CellColor::White,
        },
        ColorTheme::DeepBlue => match band {
            0 | 1 => CellColor::DarkBlue,
            2 => CellColor::Blue,
            _ => CellColor::Blue,
        },
    })
}

fn terminal_grid_size(show_status: bool) -> Option<GridSize> {
    let (cols, rows) = terminal::size().ok()?;
    let cols = usize::from(cols).max(1);
    let status_rows = usize::from(show_status);
    let rows = usize::from(rows).saturating_sub(status_rows).max(1);

    Some(GridSize { cols, rows })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_rendering_uses_portable_foreground_sgr() {
        let mut grid = DensityGrid::new(2, 1);
        grid.add_density(0, 0, 1.0);
        grid.add_density(1, 0, DENSITY_FULL);

        let config = Config {
            color: ColorTheme::DeepBlue,
            ..Config::defaults()
        };
        let output = grid.render(0, &config);

        assert!(output.contains("\x1b[34m"));
        assert!(output.contains("\x1b[94m"));
        assert!(!output.contains("\x1b[38;2;"));
        assert!(!output.contains("\x1b[48;"));
    }

    #[test]
    fn charset_rendering_changes_density_glyphs() {
        let mut grid = DensityGrid::new(2, 1);
        grid.add_density(0, 0, 0.25);
        grid.add_density(1, 0, DENSITY_FULL);

        let dots = grid.render(
            0,
            &Config {
                color: ColorTheme::Mono,
                charset: Charset::Dots,
                ..Config::defaults()
            },
        );
        assert!(dots.contains('.'));
        assert!(dots.contains('\u{25cf}'));

        let solid = grid.render(
            0,
            &Config {
                color: ColorTheme::Mono,
                charset: Charset::Solid,
                ..Config::defaults()
            },
        );
        assert!(solid.contains("\u{2588}\u{2588}"));
    }

    #[test]
    fn setup_save_preserves_led_settings() {
        let path = env::temp_dir().join(format!(
            "liquid-terminal-settings-{}-preserve-led.env",
            std::process::id()
        ));

        fs::write(
            &path,
            "\
LIQUID_PARTICLES=500
LIQUID_LED_PANEL_WIDTH=8
LIQUID_LED_CHAIN_COLS=3
IGNORED_VALUE=yes
LIQUID_LED_ORIGIN=bottom-left
",
        )
        .unwrap();

        let preserved = preserved_led_settings(&path).unwrap();
        let _ = fs::remove_file(path);

        assert_eq!(
            preserved,
            vec![
                "LIQUID_LED_PANEL_WIDTH=8".to_string(),
                "LIQUID_LED_CHAIN_COLS=3".to_string(),
                "LIQUID_LED_ORIGIN=bottom-left".to_string()
            ]
        );
    }
}

fn print_help() {
    println!(
        "Usage: cargo run --example terminal -- [OPTIONS]\n\
\n\
With no arguments, opens the interactive setup screen.\n\
\n\
Options:\n\
  --cols N        Terminal grid columns [default: 100]\n\
  --rows N        Terminal grid rows [default: 50]\n\
  --auto-size     Use terminal size and adapt when resized\n\
  --fixed-size    Use --cols and --rows instead of terminal size\n\
  --color THEME   Color theme: blue, cyan, deep-blue, mono [default: blue]\n\
  --no-color      Alias for --color mono\n\
  --charset NAME  Density characters: classic, dots, blocks, solid [default: classic]\n\
  --gravity-spin N\n\
                  Gravity rotation speed multiplier [default: 1.0]\n\
  --particles N   Particle count [default: 2000]\n\
  --fps N         Target frames per second [default: 60]\n\
  --frames N      Stop after N frames, useful for smoke tests\n\
  --setup         Open the interactive setup screen before rendering\n\
  --status        Show a changing status line above the animation\n\
  --no-status     Hide the status line [default]\n\
  -h, --help      Show this help"
    );
}
