use ggdt::entities::*;
use ggdt::events::*;
use ggdt::graphics::*;
use ggdt::graphics::indexed::*;
use ggdt::math::*;
use ggdt::states::*;
use ggdt::system::*;

use crate::*;

pub const BALL_SIZE: i32 = 8;
pub const NUM_BALLS: usize = 32;
pub const NUM_BALL_SPRITES: usize = 16;

pub struct Context {
	pub delta: f32,
	pub system: System<DosLike>,
	pub font: BitmaskFont,
	pub sprites: Vec<Bitmap>,
	pub entities: Entities,
	pub event_publisher: EventPublisher<Event>,
}

pub struct Game {
	pub context: Context,
	pub component_systems: ComponentSystems<Context, Context>,
	pub event_listeners: EventListeners<Event, Context>,
}

impl Game {
	pub fn new(mut system: System<DosLike>) -> Result<Self> {
		let font = BitmaskFont::new_vga_font()?;

		let (balls_bmp, balls_palette) = Bitmap::load_pcx_file(Path::new("./assets/balls.pcx"))?;
		system.res.palette = balls_palette.clone();

		let mut sprites = Vec::new();
		for i in 0..NUM_BALL_SPRITES {
			let mut sprite = Bitmap::new(BALL_SIZE as u32, BALL_SIZE as u32)?;
			sprite.blit_region(
				BlitMethod::Solid,
				&balls_bmp,
				&Rect::new(i as i32 * BALL_SIZE as i32, 0, BALL_SIZE as u32, BALL_SIZE as u32),
				0,
				0,
			);
			sprites.push(sprite);
		}

		let entities = Entities::new();
		let mut component_systems = ComponentSystems::new();
		let event_publisher = EventPublisher::new();
		let mut event_listeners = EventListeners::new();

		init_component_system(&mut component_systems);
		init_event_listeners(&mut event_listeners);

		Ok(Game {
			context: Context {
				delta: 0.0,
				system,
				font,
				sprites,
				entities,
				event_publisher,
			},
			component_systems,
			event_listeners,
		})
	}

	pub fn do_events(&mut self) {
		self.event_listeners.take_queue_from(&mut self.context.event_publisher);
		self.event_listeners.dispatch_queue(&mut self.context);
	}
}

pub struct SimulationState;

impl AppState<Game> for SimulationState {
	fn update(&mut self, _state: State, context: &mut Game) -> Option<StateChange<Game>> {
		if context.context.system.res.keyboard.is_key_up(Scancode::S) {
			context.do_events();
			context.component_systems.update(&mut context.context);
		}

		if context.context.system.res.keyboard.is_key_pressed(Scancode::Escape) {
			return Some(StateChange::Pop(1));
		}

		None
	}

	fn render(&mut self, _state: State, context: &mut Game) {
		context.context.system.res.video.clear(2);
		context.component_systems.render(&mut context.context);
		context.context.system.res.video.print_string("hello, world!", 10, 10, FontRenderOpts::Color(15), &context.context.font);
	}

	fn transition(&mut self, _state: State, _context: &mut Game) -> bool {
		true
	}

	fn state_change(&mut self, new_state: State, _old_state: State, context: &mut Game) {
		if new_state == State::Pending {
			init_entities(&mut context.context.entities);
		}
	}
}