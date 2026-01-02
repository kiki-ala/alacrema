use anathema::backend::tui::Style;
use anathema::component::*;
use anathema::default_widgets::Canvas;
use anathema::prelude::*;
use portable_pty::PtyPair;
use portable_pty::PtySystem;
use std::cmp::max;
use std::collections::VecDeque;
use std::time::Duration;
use std::time::Instant;

extern crate rand;
extern crate rand_chacha;
use rand::{Rng, SeedableRng};

// use anyhow::anyhow;
// use futures::prelude::*;
// use std::ffi::OsString;

use portable_pty::PtySize;
use portable_pty::native_pty_system;
// use portable_pty::{CommandBuilder, PtySize};

const BUBBLE: &str = "·⋅◌⊙⊚⦾⁜";

enum UserRequestType {
    NewPTY,
}

#[derive(Default)]
struct CommandQueue {
    vd: VecDeque<UserRequestType>,
}

#[derive(State)]
struct UIMainState {
    #[anathema(ignore)]
    command_queue: CommandQueue,
    fps: Value<i32>,
}

struct PseudoTerminalLoom {
    pty_system: Option<Box<dyn PtySystem + Send>>,
    ptys: Vec<PtyPair>,
}

impl PseudoTerminalLoom {
    fn _new() -> Self {
        Self {
            pty_system: None,
            ptys: vec![],
        }
    }

    #[allow(dead_code)]
    fn init_pty_system(&mut self) {
        self.pty_system = Some(native_pty_system());
    }

    #[allow(dead_code)]
    async fn spawn_pty(&mut self) -> anyhow::Result<()> {
        if self.pty_system.is_none() {
            self.init_pty_system();
        }

        let pty_system = self.pty_system.as_ref().unwrap();
        self.ptys.push(pty_system.openpty(PtySize {
            rows: 8,
            cols: 16,
            pixel_width: 0,
            pixel_height: 0,
        })?);
        Ok(())
    }
}

impl UIMainState {
    fn new() -> Self {
        Self {
            command_queue: CommandQueue::default(),
            fps: 24.into(),
        }
    }

    pub fn request_new_pty(&mut self) {
        self.command_queue.vd.push_front(UserRequestType::NewPTY);
    }
}

struct UIMain {}

impl UIMain {
    fn new() -> Self {
        Self {}
    }
}

impl Component for UIMain {
    type Message = ();
    type State = UIMainState;

    fn on_tick(
        &mut self,
        state: &mut Self::State,
        mut interior: Children<'_, '_>,
        _context: Context<'_, '_, Self::State>,
        _dt: Duration,
    ) {
        // process user command request queue,
        // haphazardly simply taking the back one
        if let Some(cmd) = state.command_queue.vd.pop_back() {
            match cmd {
                UserRequestType::NewPTY => state.request_new_pty(),
            };
        }

        // not strictly necessary, this was prototype code
        // just to manage global fps of canvasfx components
        let mut elements = interior.elements();
        elements
            .by_attribute("id", "canvasfx")
            .each(|_e, attributes| {
                attributes.set("fps", state.fps.copy_value());
            });
    }

    fn on_key(
        &mut self,
        key: KeyEvent,
        state: &mut Self::State,
        mut _interior: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
        match key.code {
            KeyCode::Char('j') => {
                let current = *state.fps.to_mut();
                if current > 1 {
                    *state.fps.to_mut() -= 1;
                }
            }
            KeyCode::Char('n') => state.request_new_pty(),
            KeyCode::Char('q') => context.stop_runtime(),
            KeyCode::Char('k') => {
                let current = *state.fps.to_mut();
                if current < 30 {
                    *state.fps.to_mut() += 1;
                }
            }
            _ => {}
        }
    }
}

#[derive(State)]
struct StatusLineState {}

impl StatusLineState {
    fn new() -> Self {
        Self {}
    }
}

struct StatusLine {}

impl StatusLine {
    fn new() -> Self {
        Self {}
    }
}

impl Component for StatusLine {
    type Message = ();
    type State = StatusLineState;

    fn on_tick(
        &mut self,
        _state: &mut Self::State,
        mut _interior: Children<'_, '_>,
        _context: Context<'_, '_, Self::State>,
        _dt: Duration,
    ) {
        // unused for now
    }

    fn on_key(
        &mut self,
        _key: KeyEvent,
        _state: &mut Self::State,
        mut _interior: Children<'_, '_>,
        _context: Context<'_, '_, Self::State>,
    ) {
        // unused for now
    }
}

#[derive(State)]
struct StatusFeedState {}

impl StatusFeedState {
    fn new() -> Self {
        Self {}
    }
}

struct StatusFeed {}

impl StatusFeed {
    fn new() -> Self {
        Self {}
    }
}

impl Component for StatusFeed {
    type Message = ();
    type State = StatusFeedState;

    fn on_tick(
        &mut self,
        _state: &mut Self::State,
        mut _interior: Children<'_, '_>,
        _context: Context<'_, '_, Self::State>,
        _dt: Duration,
    ) {
        // unused for now
    }

    fn on_key(
        &mut self,
        _key: KeyEvent,
        _state: &mut Self::State,
        mut _interior: Children<'_, '_>,
        _context: Context<'_, '_, Self::State>,
    ) {
        // unused for now
    }
}

#[derive(State)]
struct CanvasFXState {}

impl CanvasFXState {
    fn new() -> Self {
        Self {}
    }
}

struct CanvasFX {
    app_start: Instant,
    time_secs: f64,     // this value is only updated on an animation tick
    anim_tick: usize,   // this value goes up every n seconds
    anim_tick_per: f64, // seconds per animation tick
}

impl CanvasFX {
    fn new() -> Self {
        Self {
            app_start: Instant::now(),
            time_secs: 0.0f64,
            anim_tick: 0usize,
            anim_tick_per: 1.0 / 24.0,
        }
    }
}

pub fn safe_neighbor(sz: (u16, u16), coord: (u16, u16), delta: (i8, i8)) -> (u16, u16) {
    let mut test_coord = (
        coord.0 as i32 + delta.0 as i32,
        coord.1 as i32 + delta.1 as i32,
    );
    if test_coord.0 <= 0 {
        if test_coord.1 <= 0 {
            test_coord = (0, 0);
        } else if test_coord.1 >= sz.1 as i32 {
            test_coord = (0, sz.1 as i32);
        }
    } else if test_coord.0 >= sz.0 as i32 {
        if test_coord.1 <= 0 {
            test_coord = (sz.0 as i32, 0);
        } else if test_coord.1 >= sz.1 as i32 {
            test_coord = (sz.0 as i32, sz.1 as i32);
        }
    }
    (test_coord.0 as u16, test_coord.1 as u16)
}

impl Component for CanvasFX {
    type Message = ();
    type State = CanvasFXState;

    fn on_tick(
        &mut self,
        _state: &mut Self::State,
        mut interior: Children<'_, '_>,
        _context: Context<'_, '_, Self::State>,
        _dt: Duration,
    ) {
        let time_now_secs = self.app_start.elapsed().as_secs_f64();

        if self.time_secs + self.anim_tick_per > time_now_secs {
            return;
        }

        self.time_secs = time_now_secs;
        self.anim_tick += 1;
        let mut elements = interior.elements();
        elements.by_attribute("id", "canvasfx").first(|e, a| {
            // here we retrieve any configured animation fps value.
            let fps = a.get("fps").unwrap().as_int().unwrap();
            self.anim_tick_per = 1.0 / (max(fps, 1) as f64);

            let sz = e.size();
            let w = sz.width;
            let h = sz.height;
            // let eib = e.inner_bounds();
            // let w = eib.to.x - eib.from.x;
            // let h = eib.to.y - eib.from.y;

            let canvas = e.to::<Canvas>();
            let style = Style::new();

            // let output = a
            //     .get("output")
            //     .unwrap()
            //     .clone()
            //     .as_str()
            //     .unwrap()
            //     .to_string();

            if self.anim_tick.is_multiple_of(4) {
                let mut x_index = 0;
                let mut y_index = 0;
                // rectangles are weird huh
                // this puts the output text on the canvas
                let output = "text placeholder text placeholder text placeholder text placeholder\ntext placeholder text placeholder text placeholder text placeholder\ntext placeholder text placeholder text placeholder text placeholder\ntext placeholder text placeholder text placeholder text placeholder\ntext placeholder text placeholder text placeholder text placeholder\ntext placeholder text placeholder text placeholder text placeholder\ntext placeholder text placeholder text placeholder text placeholder\ntext placeholder text placeholder text placeholder text placeholder\ntext placeholder text placeholder text placeholder text placeholder\n";
                for char in output.chars() {
                    if char == '\n' {
                        y_index += 1;
                        x_index = 0;
                        continue;
                    }
                    canvas.put(char, style, (x_index, y_index));
                    x_index += 1;
                }
            }

            let mut rng = rand_chacha::ChaCha8Rng::from_os_rng();
            for y in 0..h {
                for x in 0..w {
                    let coord = (x, y);
                    let at_coord = canvas.get(coord);

                    if (at_coord.is_none() || (at_coord.is_some() && at_coord.unwrap().0 == ' '))
                        && rng.random::<f32>() < 0.01
                    {
                        canvas.put(BUBBLE.chars().nth(0).unwrap(), style, coord);
                    } else if at_coord.is_some() {
                        let c = at_coord.unwrap();
                        for (idx, anim_frame) in BUBBLE.chars().enumerate() {
                            if c.0 == anim_frame {
                                // if the bubble is reaching it, it gets erased
                                canvas.erase(coord);

                                let next_char = BUBBLE.chars().nth(idx + 1);
                                // let next_next_char = BUBBLE.chars().nth(idx + 2);
                                let coord_above = safe_neighbor((w, h), coord, (0, -1));

                                if let Some(nx) = next_char {
                                    canvas.put(nx, style, coord_above);
                                }
                                if let Some(_above_char) = canvas.get(coord_above) && idx > (BUBBLE.len() / 2) {
                                    canvas.erase(safe_neighbor((w, h), coord, (1, -1)));
                                    canvas.erase(safe_neighbor((w, h), coord, (-1, -1)));
                                }

                            }
                        }
                    }
                }
            } // end y loop
        });
    }

    fn on_key(
        &mut self,
        _key: KeyEvent,
        _state: &mut Self::State,
        mut _interior: Children<'_, '_>,
        _context: Context<'_, '_, Self::State>,
    ) {
        // unused for now
    }
}

fn main() -> anyhow::Result<()> {
    smol::block_on(async {
        /*
        let pty_system = native_pty_system();

        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let subprocess_cmd: Vec<OsString> = vec![
            "ls".to_string().into(),
            "--color=auto".to_string().into(),
            "-lh".to_string().into(),
        ];
        let mut cmd = CommandBuilder::from_argv(subprocess_cmd);
        if let Ok(cwd) = std::env::current_dir() {
            cmd.cwd(cwd);
        }

        let slave = pair.slave;
        // NOTE: deadlock avoidance
        let mut subprocess = smol::unblock(move || slave.spawn_command(cmd)).await?;

        {
            // NOTE: deadlock avoidance
            let writer = pair.master.take_writer()?;

            // Explicitly generate EOF
            drop(writer);
        }

        println!(
            "subprocess status: {:?}",
            smol::unblock(move || subprocess.wait().map_err(|e| anyhow!(": {}", e))).await?
        );

        let reader = pair.master.try_clone_reader()?;

        // NOTE: take care. only after processes are done
        drop(pair.master);

        let mut output: String = Default::default();
        let mut lines = smol::io::BufReader::new(smol::Unblock::new(reader)).lines();
        while let Some(line) = lines.next().await {
            let line = line.map_err(|e| anyhow!("problem reading line: {}", e))?;
            for c in line.escape_debug() {
                output.push(c);
            }
            output.push('\n');
        }
        */

        let doc = Document::new("@main");

        let mut backend = {
            let mut inst = TuiBackend::builder()
                .enable_alt_screen()
                .enable_raw_mode()
                .hide_cursor()
                .finish()
                .unwrap();
            inst.finalize();
            inst
        };

        let mut builder = Runtime::builder(doc, &backend);
        builder
            .component("main", "src/ui.aml", UIMain::new(), UIMainState::new())
            .unwrap();

        // statusline prototype
        builder
            .prototype(
                "statusline",
                "src/statusline.aml",
                StatusLine::new,
                StatusLineState::new,
            )
            .unwrap();

        // statusfeed prototype
        builder
            .prototype(
                "statusfeed",
                "src/statusfeed.aml",
                StatusFeed::new,
                StatusFeedState::new,
            )
            .unwrap();

        // canvasfx prototype
        builder
            .prototype(
                "canvasfx",
                "src/canvasfx.aml",
                CanvasFX::new,
                CanvasFXState::new,
            )
            .unwrap();

        builder
            .finish(&mut backend, |runtime, backend| runtime.run(backend))
            .unwrap();

        Ok(())
    })
}
