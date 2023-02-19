use anyhow::{Context, Result};

use libretrogd::{SCREEN_HEIGHT, SCREEN_WIDTH};
use libretrogd::base::*;
use libretrogd::entities::*;
use libretrogd::events::*;
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
    let mut positions = context.entities.components_mut::<Position>().unwrap();
    let velocities = context.entities.components::<Velocity>().unwrap();

    for (entity, position) in positions.iter_mut() {
        let velocity = velocities.get(entity).unwrap();
        position.0 += velocity.0 * context.delta;
    }
}

pub fn update_system_remove_offscreen(context: &mut Core) {
    let positions = context.entities.components::<Position>().unwrap();
    for (entity, position) in positions.iter() {
        if !context.system.video.is_xy_visible(position.0.x as i32, position.0.y as i32) {
            context.event_publisher.queue(Event::Remove(*entity));
        }
    }
}

pub fn render_system_pixels(context: &mut Core) {
    let positions = context.entities.components::<Position>().unwrap();
    let colors = context.entities.components::<Color>().unwrap();

    for (entity, position) in positions.iter() {
        let color = colors.get(entity).unwrap();
        context.system.video.set_pixel(position.0.x as i32, position.0.y as i32, color.0);
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DemoState;

impl DemoState {
    fn init(&mut self, context: &mut App) {
        context.core.entities.init_components::<Position>();
        context.core.entities.init_components::<Velocity>();
        context.core.entities.init_components::<Color>();

        context.support.component_systems.reset();
        context.support.component_systems.add_update_system(update_system_movement);
        context.support.component_systems.add_update_system(update_system_remove_offscreen);
        context.support.component_systems.add_render_system(render_system_pixels);

        context.support.event_listeners.clear();
        context.support.event_listeners.add(event_listener);
    }
}

impl AppState<App> for DemoState {
    fn update(&mut self, state: State, context: &mut App) -> Option<StateChange<App>> {
        if state == State::Active {
            if context.core.system.keyboard.is_key_pressed(Scancode::Escape) {
                return Some(StateChange::Pop(1))
            }
        }

        if rnd_value(0, 100) < 80 {
            context.core.event_publisher.queue(Event::SpawnPixel);
        }

        context.support.do_events(&mut context.core);
        context.support.component_systems.update(&mut context.core);

        None
    }

    fn render(&mut self, state: State, context: &mut App) {
        context.core.system.video.clear(0);
        context.support.component_systems.render(&mut context.core);
    }

    fn transition(&mut self, state: State, context: &mut App) -> bool {
        true
    }

    fn state_change(&mut self, new_state: State, old_state: State, context: &mut App) {
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

impl CoreState for Core {
    fn system(&self) -> &System {
        &self.system
    }

    fn system_mut(&mut self) -> &mut System {
        &mut self.system
    }

    fn delta(&self) -> f32 {
        self.delta
    }

    fn set_delta(&mut self, delta: f32) {
        self.delta = delta;
    }
}

impl CoreStateWithEvents<Event> for Core {
    fn event_publisher(&mut self) -> &mut EventPublisher<Event> {
        &mut self.event_publisher
    }
}

pub struct Support {
    pub component_systems: ComponentSystems<Core, Core>,
    pub event_listeners: EventListeners<Event, Core>
}

impl SupportSystems for Support {}

impl SupportSystemsWithEvents<Event> for Support {
    type ContextType = Core;

    fn event_listeners(&mut self) -> &mut EventListeners<Event, Self::ContextType> {
        &mut self.event_listeners
    }
}

pub struct App {
    pub core: Core,
    pub support: Support,
}

impl AppContext for App {
    type CoreType = Core;
    type SupportType = Support;

    fn core(&mut self) -> &mut Core {
        &mut self.core
    }

    fn support(&mut self) -> &mut Support {
        &mut self.support
    }
}

impl App {
    pub fn new(system: System) -> Result<Self> {
        let entities = Entities::new();
        let component_systems = ComponentSystems::new();
        let event_publisher = EventPublisher::new();
        let event_listeners = EventListeners::new();

        Ok(App {
            core: Core {
                delta: 0.0,
                system,
                entities,
                event_publisher,
            },
            support: Support {
                component_systems,
                event_listeners,
            }
        })
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

fn main() -> Result<()> {
    let system = SystemBuilder::new().window_title("Complicated Template").vsync(true).build()?;
    let app = App::new(system)?;
    main_loop(app, DemoState).context("Main loop error")
}
