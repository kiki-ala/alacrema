use anathema::backend::tui::Style;
use anathema::component::*;
use anathema::default_widgets::Canvas;
use anathema::prelude::*;
use rand::prelude::*;
use std::cmp::max;
use std::time::Duration;
use std::time::Instant;

const BUBBLE: &str = "·⋅◌⊙⊚⦾⁜";

#[derive(State)]
struct UIMainState {
    fps: Value<usize>,
}

impl UIMainState {
    fn new() -> Self {
        Self { fps: 24.into() }
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
                let fps = state.fps.copy_value();
                attributes.set("fps", fps as i32);
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
            let canvas = e.to::<Canvas>();
            let style = Style::new();
            let mut rng = StdRng::from_rng(&mut rand::rng());

            // just some random bubbles/fizz animation
            for y in 0..sz.height {
                for x in 0..sz.width {
                    let mut coord = (x, y);
                    let at_coord = canvas.get(coord);
                    if (at_coord.is_none() || (at_coord.is_some() && at_coord.unwrap().0 == ' '))
                        && rng.next_u32() < (u32::MAX / 2048)
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
                                canvas.put(next_char, style, coord);
                            }
                        }
                    }
                }
            }
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

fn main() {
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
}
