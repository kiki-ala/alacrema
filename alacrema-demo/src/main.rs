use anathema::backend::tui::Style;
use anathema::component::*;
use anathema::default_widgets::Canvas;
use anathema::prelude::*;
use rand::prelude::*;
use std::cmp::max;
use std::time::Duration;
use std::time::Instant;

use anyhow::anyhow;
use futures::prelude::*;
use portable_pty::native_pty_system;
use portable_pty::{CommandBuilder, PtySize};
use std::ffi::OsString;

const BUBBLE: &str = "·⋅◌⊙⊚⦾⁜";

#[derive(State)]
struct UIMainState {
    fps: Value<i32>,
    stdout_test: Value<String>,
}

impl UIMainState {
    fn _new() -> Self {
        Self {
            fps: 24.into(),
            stdout_test: "".to_string().into(),
        }
    }

    fn with_test_output(test_output: String) -> Self {
        Self {
            fps: 24.into(),
            stdout_test: test_output.into(),
        }
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
        let mut elements = interior.elements();
        elements
            .by_attribute("id", "canvasfx")
            .each(|_e, attributes| {
                attributes.set("fps", state.fps.copy_value());
                attributes.set("output", state.stdout_test.to_ref().to_string());
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
            KeyCode::Char('q') => context.stop_runtime(),
            KeyCode::Char('k') => {
                let current = *state.fps.to_mut();
                if current < 240 {
                    *state.fps.to_mut() += 1;
                }
            }
            _ => {}
        }
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
            let mut rng = StdRng::from_rng(&mut rand::rng());

            let output = a
                .get("output")
                .unwrap()
                .clone()
                .as_str()
                .unwrap()
                .to_string();
            let chars = output.chars();

            let mut x_index = 0;
            let mut y_index = 0;
            // rectangles are weird huh
            for char in chars {
                if char == '\n' {
                    y_index += 1;
                    x_index = 0;
                    continue;
                }
                canvas.put(char, style, (x_index, y_index));
                x_index += 1;
            }

            for y in 0..h {
                for x in 0..w {
                    let mut coord = (x, y);
                    let mut at_coord = canvas.get(coord);

                    if (at_coord.is_none() || (at_coord.is_some() && at_coord.unwrap().0 == ' '))
                        && rng.next_u32() < (u32::MAX / 1024)
                    {
                        canvas.put(BUBBLE.chars().nth(0).unwrap(), style, coord);
                    } else if at_coord.is_some() {
                        let c = at_coord.unwrap();
                        for (idx, anim_frame) in BUBBLE.chars().enumerate() {
                            if c.0 == anim_frame {
                                let next = idx + 1;
                                let next_char = BUBBLE.chars().nth(next).unwrap_or(' ');
                                canvas.put(' ', style, coord);
                                if coord.1 > 0 {
                                    coord.1 -= 1;
                                }
                                at_coord = canvas.get(coord);
                                if at_coord.is_none() {
                                    canvas.put(next_char, style, coord);
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
            .component(
                "main",
                "src/ui.aml",
                UIMain::new(),
                UIMainState::with_test_output(output),
            )
            .unwrap();
        builder
            .prototype(
                "canvasfx",
                "src/canvasFX.aml",
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
