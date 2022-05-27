use anyhow::Result;
use sdl2::keyboard::Scancode;

use libretrogd::{SCREEN_HEIGHT, SCREEN_WIDTH};
use libretrogd::entities::*;
use libretrogd::events::*;
use libretrogd::graphics::*;
use libretrogd::math::*;
use libretrogd::states::*;
use libretrogd::system::*;
use libretrogd::utils::rnd_value;

//////////////////////////////////////////////////////////////////////////////////////////////////

pub enum Event {
    Remove(EntityId),
    SpawnPixel,
}

pub fn event_listener(event: &Event, context: &mut Core) -> bool {
    match event {
        Event::Remove(entity) => {
            context.entities.remove_entity(*entity);
            true
        },
        Event::SpawnPixel => {
            let speed = rnd_value(1, 10) as f32 * 10.0;
            let angle = (rnd_value(0, 359) as f32).to_radians();
            let x = (SCREEN_WIDTH / 2) as f32;
            let y = (SCREEN_HEIGHT / 2) as f32;
            let color = rnd_value(0, 255);
            let id = context.entities.new_entity();
            context.entities.add_component(id, Position(Vector2::new(x, y)));
            context.entities.add_component(id, Velocity(Vector2::from_angle(angle) * speed));
            context.entities.add_component(id, Color(color));
            true
        },
        _ => false
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Position(Vector2);

pub struct Velocity(Vector2);

pub struct Color(u8);

//////////////////////////////////////////////////////////////////////////////////////////////////

pub fn update_system_movement(context: &mut Core) {
    if let Some(mut positions) = context.entities.components_mut::<Position>() {
        let velocities = context.entities.components::<Velocity>().unwrap();

        for (entity, position) in positions.iter_mut() {
            let velocity = velocities.get(entity).unwrap();
            position.0 += velocity.0 * context.delta;
        }
    }
}

pub fn update_system_remove_offscreen(context: &mut Core) {
    if let Some(positions) = context.entities.components::<Position>() {
        for (entity, position) in positions.iter() {
            if !context.system.video.is_xy_visible(position.0.x as i32, position.0.y as i32) {
                context.event_publisher.queue(Event::Remove(*entity));
            }
        }
    }
}

pub fn render_system_pixels(context: &mut Core) {
    if let Some(positions) = context.entities.components::<Position>() {
        let colors = context.entities.components::<Color>().unwrap();

        for (entity, position) in positions.iter() {
            let color = colors.get(entity).unwrap();
            context.system.video.set_pixel(position.0.x as i32, position.0.y as i32, color.0);
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DemoState;

impl DemoState {
    fn init(&mut self, context: &mut Game) {
        context.component_systems.reset();
        context.component_systems.add_update_system(update_system_movement);
        context.component_systems.add_update_system(update_system_remove_offscreen);
        context.component_systems.add_render_system(render_system_pixels);

        context.event_listeners.clear();
        context.event_listeners.add(event_listener);
    }
}

impl GameState<Game> for DemoState {
    fn update(&mut self, state: State, context: &mut Game) -> Option<StateChange<Game>> {
        if state == State::Active {
            if context.core.system.keyboard.is_key_pressed(Scancode::Escape) {
                return Some(StateChange::Pop(1))
            }
        }

        if rnd_value(0, 100) < 80 {
            context.core.event_publisher.queue(Event::SpawnPixel);
        }

        context.do_events();
        context.component_systems.update(&mut context.core);

        None
    }

    fn render(&mut self, state: State, context: &mut Game) {
        context.core.system.video.clear(0);
        context.component_systems.render(&mut context.core);
    }

    fn transition(&mut self, state: State, context: &mut Game) -> bool {
        true
    }

    fn state_change(&mut self, new_state: State, old_state: State, context: &mut Game) {
        match new_state {
            State::Pending => {
                self.init(context);
            },
            _ => {}
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Core {
    pub delta: f32,
    pub system: System,
    pub entities: Entities,
    pub event_publisher: EventPublisher<Event>,
}

pub struct Game {
    pub core: Core,
    pub component_systems: ComponentSystems<Core, Core>,
    pub event_listeners: EventListeners<Event, Core>
}

impl Game {
    pub fn new(system: System) -> Result<Self> {
        let entities = Entities::new();
        let component_systems = ComponentSystems::new();
        let event_publisher = EventPublisher::new();
        let event_listeners = EventListeners::new();

        Ok(Game {
            core: Core {
                delta: 0.0,
                system,
                entities,
                event_publisher
            },
            component_systems,
            event_listeners
        })
    }

    pub fn do_events(&mut self) {
        self.event_listeners.take_queue_from(&mut self.core.event_publisher);
        self.event_listeners.dispatch_queue(&mut self.core);
    }

    pub fn update_frame_delta(&mut self, last_ticks: u64) -> u64 {
        let ticks = self.core.system.ticks();
        let elapsed = ticks - last_ticks;
        self.core.delta = (elapsed as f64 / self.core.system.tick_frequency() as f64) as f32;
        ticks
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

fn main() -> Result<()> {
    let system = SystemBuilder::new().window_title("Complicated Template").vsync(true).build()?;
    let mut game = Game::new(system)?;
    let mut states = States::new();
    states.push(DemoState)?;

    let mut is_running = true;
    let mut last_ticks = game.core.system.ticks();

    while is_running && !states.is_empty() {
        game.core.system.do_events_with(|event| {
            if let sdl2::event::Event::Quit { .. } = event {
                is_running = false;
            }
        });

        last_ticks = game.update_frame_delta(last_ticks);
        states.update(&mut game)?;
        states.render(&mut game);
        game.core.system.display()?;
    }

    Ok(())
}
