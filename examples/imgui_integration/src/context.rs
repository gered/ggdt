use crate::entities::{AnimationDef, EntityActivity, Event};
use crate::support::{load_bitmap_atlas_autogrid, load_font, load_palette};
use crate::tilemap::TileMap;
use anyhow::Result;
use ggdt::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

pub struct CoreContext {
	pub delta: f32,
	pub camera_x: i32,
	pub camera_y: i32,
	pub transparent_color: ARGB,
	pub system: System<Standard>,
	pub palette: Palette,
	pub font: BitmaskFont,
	pub small_font: BitmaskFont,
	pub entities: Entities,
	pub event_publisher: EventPublisher<Event>,
	pub tiles: Rc<BitmapAtlas<RgbaBitmap>>,
	pub green_slime: Rc<BitmapAtlas<RgbaBitmap>>,
	pub blue_slime: Rc<BitmapAtlas<RgbaBitmap>>,
	pub orange_slime: Rc<BitmapAtlas<RgbaBitmap>>,
	pub tilemap: TileMap,
	pub slime_activity_states: Rc<HashMap<EntityActivity, Rc<AnimationDef>>>,
	pub sprite_render_list: Vec<(EntityId, Vector2, RgbaBlitMethod)>,
}

impl CoreState<Standard> for CoreContext {
	fn system(&self) -> &System<Standard> {
		&self.system
	}

	fn system_mut(&mut self) -> &mut System<Standard> {
		&mut self.system
	}

	fn delta(&self) -> f32 {
		self.delta
	}

	fn set_delta(&mut self, delta: f32) {
		self.delta = delta;
	}
}

impl CoreStateWithEvents<Standard, Event> for CoreContext {
	fn event_publisher(&mut self) -> &mut EventPublisher<Event> {
		&mut self.event_publisher
	}
}

pub struct SupportContext {
	pub component_systems: ComponentSystems<CoreContext, CoreContext>,
	pub event_listeners: EventListeners<Event, CoreContext>,
	pub imgui: ggdt_imgui::ImGui,
}

impl SupportSystems for SupportContext {}

impl SupportSystemsWithEvents<Standard, Event> for SupportContext {
	type ContextType = CoreContext;

	fn event_listeners(&mut self) -> &mut EventListeners<Event, Self::ContextType> {
		&mut self.event_listeners
	}
}

pub struct GameContext {
	pub core: CoreContext,
	pub support: SupportContext,
}

impl AppContext<Standard> for GameContext {
	type CoreType = CoreContext;
	type SupportType = SupportContext;

	fn core(&mut self) -> &mut Self::CoreType {
		&mut self.core
	}

	fn support(&mut self) -> &mut Self::SupportType {
		&mut self.support
	}
}

impl GameContext {
	pub fn new(system: System<Standard>) -> Result<Self> {
		let palette = load_palette(Path::new("./assets/db16.pal"))?;

		let font = load_font(Path::new("./assets/dp.fnt"))?;
		let small_font = load_font(Path::new("./assets/small.fnt"))?;

		let tiles = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/tiles.pcx"))?);
		let green_slime = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/green_slime.pcx"))?);
		let blue_slime = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/blue_slime.pcx"))?);
		let orange_slime = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/orange_slime.pcx"))?);

		let tilemap = TileMap::load_from(Path::new("./assets/arena.map.json"))?;

		let entities = Entities::new();
		let component_systems = ComponentSystems::new();
		let event_publisher = EventPublisher::new();
		let event_listeners = EventListeners::new();
		let imgui = ggdt_imgui::ImGui::new();

		let slime_activity_states = HashMap::from([
			(EntityActivity::Idle, Rc::new(AnimationDef::new(&[1, 2], true, 1.0, Some(3)))),
			(EntityActivity::Walking, Rc::new(AnimationDef::new(&[1, 0, 2, 0], true, 0.25, Some(3)))),
		]);

		Ok(GameContext {
			core: CoreContext {
				delta: 0.0,
				camera_x: 0,
				camera_y: 0,
				transparent_color: palette[0],
				system,
				palette,
				font,
				small_font,
				entities,
				event_publisher,
				tiles,
				green_slime,
				blue_slime,
				orange_slime,
				tilemap,
				slime_activity_states: Rc::new(slime_activity_states),
				sprite_render_list: Vec::new(),
			},
			support: SupportContext { component_systems, event_listeners, imgui },
		})
	}
}
