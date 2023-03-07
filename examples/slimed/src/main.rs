extern crate core;

use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use anyhow::{Context, Result};

use ggdt::base::*;
use ggdt::entities::*;
use ggdt::events::*;
use ggdt::graphics::*;
use ggdt::math::*;
use ggdt::system::*;

use crate::entities::*;
use crate::states::*;
use crate::support::*;
use crate::tilemap::*;

mod states;
mod entities;
mod support;
mod tilemap;

pub const TILE_WIDTH: u32 = 16;
pub const TILE_HEIGHT: u32 = 16;

pub struct Core {
	pub delta: f32,
	pub system: System<DosLike>,
	pub font: BitmaskFont,
	pub entities: Entities,
	pub event_publisher: EventPublisher<Event>,
	pub palette: Palette,
	pub fade_out_palette: Palette,
	pub tiles: Rc<BitmapAtlas>,
	pub hero_male: Rc<BitmapAtlas>,
	pub hero_female: Rc<BitmapAtlas>,
	pub green_slime: Rc<BitmapAtlas>,
	pub blue_slime: Rc<BitmapAtlas>,
	pub orange_slime: Rc<BitmapAtlas>,
	pub fist: Rc<BitmapAtlas>,
	pub sword: Rc<BitmapAtlas>,
	pub particles: Rc<BitmapAtlas>,
	pub items: Rc<BitmapAtlas>,
	pub ui: Rc<BitmapAtlas>,
	pub tilemap: TileMap,
	pub slime_activity_states: Rc<HashMap<EntityActivity, Rc<AnimationDef>>>,
	pub hero_activity_states: Rc<HashMap<EntityActivity, Rc<AnimationDef>>>,
	pub poof1_animation_def: Rc<AnimationDef>,
	pub poof2_animation_def: Rc<AnimationDef>,
	pub sparkles_animation_def: Rc<AnimationDef>,
	pub sprite_render_list: Vec<(EntityId, Vector2, BlitMethod)>,
}

impl CoreState<DosLike> for Core {
	fn system(&self) -> &System<DosLike> {
		&self.system
	}

	fn system_mut(&mut self) -> &mut System<DosLike> {
		&mut self.system
	}

	fn delta(&self) -> f32 {
		self.delta
	}

	fn set_delta(&mut self, delta: f32) {
		self.delta = delta;
	}
}

impl CoreStateWithEvents<DosLike, Event> for Core {
	fn event_publisher(&mut self) -> &mut EventPublisher<Event> {
		&mut self.event_publisher
	}
}

pub struct Support {
	pub component_systems: ComponentSystems<Core, Core>,
	pub event_listeners: EventListeners<Event, Core>,
}

impl SupportSystems for Support {}

impl SupportSystemsWithEvents<DosLike, Event> for Support {
	type ContextType = Core;

	fn event_listeners(&mut self) -> &mut EventListeners<Event, Self::ContextType> {
		&mut self.event_listeners
	}
}

pub struct Game {
	pub core: Core,
	pub support: Support,
}

impl AppContext<DosLike> for Game {
	type CoreType = Core;
	type SupportType = Support;

	fn core(&mut self) -> &mut Self::CoreType {
		&mut self.core
	}

	fn support(&mut self) -> &mut Self::SupportType {
		&mut self.support
	}
}

impl Game {
	pub fn new(mut system: System<DosLike>) -> Result<Self> {
		let palette = load_palette(Path::new("./assets/db16.pal"))?;
		system.res.palette = palette.clone();

		let font = load_font(Path::new("./assets/dp.fnt"))?;

		let tiles = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/tiles.pcx"))?);
		let hero_male = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/hero_male.pcx"))?);
		let hero_female = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/hero_female.pcx"))?);
		let green_slime = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/green_slime.pcx"))?);
		let blue_slime = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/blue_slime.pcx"))?);
		let orange_slime = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/orange_slime.pcx"))?);
		let fist = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/fist.pcx"))?);
		let sword = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/sword.pcx"))?);
		let particles = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/particles.pcx"))?);
		let items = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/items.pcx"))?);

		let mut ui = load_bitmap_atlas(Path::new("./assets/ui.pcx"))?;
		ui.add(Rect::new(0, 0, 16, 16))?;
		ui.add(Rect::new(16, 0, 16, 16))?;
		for i in 0..8 {
			ui.add(Rect::new(i * 8, 16, 8, 8))?;
		}

		let tilemap = TileMap::load_from(Path::new("./assets/title_screen.map.json"))?;

		let entities = Entities::new();
		let component_systems = ComponentSystems::new();
		let event_publisher = EventPublisher::new();
		let event_listeners = EventListeners::new();

		let slime_activity_states = HashMap::from([
			(EntityActivity::Idle, Rc::new(AnimationDef::new(&[1, 2], true, 1.0, Some(3)))),
			(EntityActivity::Walking, Rc::new(AnimationDef::new(&[1, 0, 2, 0], true, 0.25, Some(3)))),
			(EntityActivity::Attacking, Rc::new(AnimationDef::new(&[0], false, 0.3, Some(3)))),
			(EntityActivity::Dead, Rc::new(AnimationDef::new(&[12], false, 1.0, None))),
		]);
		let hero_activity_states = HashMap::from([
			(EntityActivity::Idle, Rc::new(AnimationDef::new(&[0], true, 0.5, Some(4)))),
			(EntityActivity::Walking, Rc::new(AnimationDef::new(&[0, 1, 0, 2], true, 0.15, Some(4)))),
			(EntityActivity::Attacking, Rc::new(AnimationDef::new(&[3], false, 0.3, Some(4)))),
			(EntityActivity::Dead, Rc::new(AnimationDef::new(&[16], false, 1.0, None))),
		]);
		let poof1_animation_def = Rc::new(AnimationDef::new(&[0, 1, 2], false, 0.15, None));
		let poof2_animation_def = Rc::new(AnimationDef::new(&[3, 4, 5], false, 0.15, None));
		let sparkles_animation_def = Rc::new(AnimationDef::new(&[6, 7, 8, 9], false, 0.1, None));

		Ok(Game {
			core: Core {
				delta: 0.0,
				system,
				font,
				entities,
				event_publisher,
				palette,
				fade_out_palette: Palette::new_with_default(20, 12, 28),
				tiles,
				hero_male,
				hero_female,
				green_slime,
				blue_slime,
				orange_slime,
				fist,
				sword,
				particles,
				items,
				ui: Rc::new(ui),
				tilemap,
				slime_activity_states: Rc::new(slime_activity_states),
				hero_activity_states: Rc::new(hero_activity_states),
				poof1_animation_def,
				poof2_animation_def,
				sparkles_animation_def,
				sprite_render_list: Vec::with_capacity(1024),
			},
			support: Support {
				component_systems,
				event_listeners,
			},
		})
	}
}

fn main() -> Result<()> {
	let config = DosLikeConfig::new().vsync(true);
	let system = SystemBuilder::new().window_title("Slime Stabbing Simulator").build(config)?;
	let game = Game::new(system)?;
	main_loop(game, MainMenuState::new()).context("Main loop error")
}
