use std::path::Path;

use anyhow::Result;

use ggdt::prelude::dos_like::*;

use crate::entities::*;
use crate::states::*;

mod entities;
mod states;

fn main() -> Result<()> {
	let config = DosLikeConfig::new();
	let system = SystemBuilder::new()
		.window_title("Flying Balls")
		.vsync(true)
		.build(config)?;
	let mut game = Game::new(system)?;
	let mut states = States::new();
	states.push(SimulationState)?;

	let tick_frequency = game.context.system.tick_frequency();
	let mut last_ticks = game.context.system.ticks();

	while !game.context.system.do_events()? && !states.is_empty() {
		let ticks = game.context.system.ticks();
		let elapsed = ticks - last_ticks;
		last_ticks = ticks;
		game.context.delta = (elapsed as f64 / tick_frequency as f64) as f32;

		states.update(&mut game)?;
		game.context.system.update()?;

		game.context.system.res.video.clear(0);
		states.render(&mut game);

		game.context.system.display()?;
	}

	Ok(())
}
