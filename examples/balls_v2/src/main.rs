use std::path::Path;

use anyhow::Result;

use libretrogd::states::*;
use libretrogd::system::*;

use crate::entities::*;
use crate::states::*;

mod entities;
mod states;

fn main() -> Result<()> {
    let system = SystemBuilder::new().window_title("Flying Balls").vsync(true).build()?;
    let mut game = Game::new(system)?;
    let mut states = States::new();
    states.push(SimulationState)?;

    let mut is_running = true;

    let tick_frequency = game.context.system.tick_frequency();
    let mut last_ticks = game.context.system.ticks();

    while is_running && !states.is_empty() {
        game.context.system.do_events_with(|event| {
            if let sdl2::event::Event::Quit { .. } = event {
                is_running = false;
            }
        });

        let ticks = game.context.system.ticks();
        let elapsed = ticks - last_ticks;
        last_ticks = ticks;
        game.context.delta = (elapsed as f64 / tick_frequency as f64) as f32;

        states.update(&mut game)?;

        game.context.system.video.clear(0);
        states.render(&mut game);

        game.context.system.display()?;
    }

    Ok(())
}
