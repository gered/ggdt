use ggdt::prelude::dos_like::*;

use crate::states::*;

pub const BALL_BASE_SPEED: i32 = 32;
pub const BOUNCE_PARTICLE_COLOR: u8 = 32;
pub const BOUNCE_PARTICLE_LIFETIME: f32 = 0.2;
pub const BOUNCE_PARTICLE_SPEED: i32 = 32;
pub const BALL_TRAIL_PARTICLE_INTERVAL: f32 = 0.05;
pub const TRAIL_PARTICLE_LIFETIME: f32 = 0.5;

pub struct Position(Vector2);

pub struct Velocity(Vector2);

pub struct SpriteIndex(usize);

pub struct BouncesAgainstEdge;

pub struct Particle;

pub struct Color(u8);

pub struct LifeLeft {
	pub life: f32,
	pub initial: f32,
}

pub struct LeavesTrail {
	pub timer: f32,
}

pub struct ColorByLifeTime(u8, u8, u8, u8, u8);

pub enum Event {
	CollideAgainstEdge(EntityId),
	Kill(EntityId),
	LeaveTrail(Vector2),
}

fn new_basic_particle_entity(entities: &mut Entities, x: f32, y: f32, color: u8, lifetime: f32, angle: f32, speed: i32) {
	let id = entities.new_entity();
	entities.add_component(id, Particle);
	entities.add_component(id, Color(color));
	entities.add_component(id, LifeLeft { life: lifetime, initial: lifetime });
	entities.add_component(id, Position(Vector2::new(x, y)));
	entities.add_component(id, Velocity(Vector2::from_angle(angle) * speed as f32));
}

fn new_trail_particle_entity(entities: &mut Entities, x: f32, y: f32, lifetime: f32) {
	let id = entities.new_entity();
	entities.add_component(id, Particle);
	entities.add_component(id, ColorByLifeTime(33, 26, 21, 16, 10));
	entities.add_component(id, LifeLeft { life: lifetime, initial: lifetime });
	entities.add_component(id, Position(Vector2::new(x, y)));
}

fn new_bounce_particles(entities: &mut Entities, x: f32, y: f32) {
	for direction in 0..6 {
		let angle = direction as f32 * (RADIANS_360 / 6.0);
		new_basic_particle_entity(
			entities,
			x,
			y,
			BOUNCE_PARTICLE_COLOR,
			BOUNCE_PARTICLE_LIFETIME,
			angle,
			BOUNCE_PARTICLE_SPEED,
		);
	}
}

fn new_ball_entity(entities: &mut Entities) {
	let id = entities.new_entity();

	let x: i32 = rnd_value(SCREEN_LEFT as i32 + 1, SCREEN_RIGHT as i32 - BALL_SIZE - 1);
	let y: i32 = rnd_value(SCREEN_TOP as i32 + 1, SCREEN_BOTTOM as i32 - BALL_SIZE - 1);

	let speed = rnd_value(1, 3) * BALL_BASE_SPEED;
	let vx = if rnd_value(0, 1) == 0 { -speed } else { speed };
	let vy = if rnd_value(0, 1) == 0 { -speed } else { speed };

	let sprite_index = rnd_value(0, NUM_BALL_SPRITES - 1);

	entities.add_component(id, Position(Vector2::new(x as f32, y as f32)));
	entities.add_component(id, Velocity(Vector2::new(vx as f32, vy as f32)));
	entities.add_component(id, SpriteIndex(sprite_index));
	entities.add_component(id, BouncesAgainstEdge);
	entities.add_component(id, LeavesTrail { timer: BALL_TRAIL_PARTICLE_INTERVAL });
}

fn update_system_movement(context: &mut Context) {
	let mut positions = context.entities.components_mut::<Position>().unwrap();
	let velocities = context.entities.components::<Velocity>();

	for (entity, position) in positions.iter_mut() {
		if let Some(velocity) = velocities.get(&entity) {
			position.0 += velocity.0 * context.delta;
		}
	}
}

fn update_system_collision(context: &mut Context) {
	let bounceables = context.entities.components::<BouncesAgainstEdge>().unwrap();
	let mut positions = context.entities.components_mut::<Position>();
	let mut velocities = context.entities.components_mut::<Velocity>();

	for (entity, _) in bounceables.iter() {
		let mut position = positions.get_mut(&entity).unwrap();
		let mut velocity = velocities.get_mut(&entity).unwrap();

		let mut bounced = false;
		if position.0.x as i32 <= SCREEN_LEFT as i32 || position.0.x as i32 + BALL_SIZE >= SCREEN_RIGHT as i32 {
			position.0.x -= velocity.0.x * context.delta;
			velocity.0.x = -velocity.0.x;
			bounced = true;
		}
		if position.0.y as i32 <= SCREEN_TOP as i32 || position.0.y as i32 + BALL_SIZE >= SCREEN_BOTTOM as i32 {
			position.0.y -= velocity.0.y * context.delta;
			velocity.0.y = -velocity.0.y;
			bounced = true;
		}
		if bounced {
			context.event_publisher.queue(Event::CollideAgainstEdge(*entity));
		}
	}
}

fn update_system_lifetime(context: &mut Context) {
	let mut lifetimes = context.entities.components_mut::<LifeLeft>().unwrap();
	for (entity, lifetime) in lifetimes.iter_mut() {
		lifetime.life -= context.delta;
		if lifetime.life < 0.0 {
			context.event_publisher.queue(Event::Kill(*entity));
		}
	}
}

fn update_system_leave_particle_trail(context: &mut Context) {
	let mut leaves_trails = context.entities.components_mut::<LeavesTrail>().unwrap();
	let positions = context.entities.components::<Position>();

	for (entity, leaves_trail) in leaves_trails.iter_mut() {
		leaves_trail.timer -= context.delta;

		if leaves_trail.timer <= 0.0 {
			leaves_trail.timer = BALL_TRAIL_PARTICLE_INTERVAL;
			let position = positions.get(&entity).unwrap();
			let mut trail_position = position.0;
			trail_position.x += (BALL_SIZE / 2) as f32;
			trail_position.y += (BALL_SIZE / 2) as f32;
			context.event_publisher.queue(Event::LeaveTrail(trail_position));
		}
	}
}

fn render_system_sprites(context: &mut Context) {
	let sprite_indices = context.entities.components::<SpriteIndex>().unwrap();
	let positions = context.entities.components::<Position>();

	for (entity, sprite_index) in sprite_indices.iter() {
		let position = positions.get(&entity).unwrap();
		context.system.res.video.blit(
			BlitMethod::Transparent(0),
			&context.sprites[sprite_index.0],
			position.0.x as i32,
			position.0.y as i32,
		);
	}
}

fn render_system_particles(context: &mut Context) {
	let particles = context.entities.components::<Particle>().unwrap();
	let positions = context.entities.components::<Position>();
	let colors = context.entities.components::<Color>();
	let colors_by_lifetime = context.entities.components::<ColorByLifeTime>();
	let lifetimes = context.entities.components::<LifeLeft>();

	for (entity, _) in particles.iter() {
		let position = positions.get(&entity).unwrap();

		let pixel_color;
		if let Some(color) = colors.get(&entity) {
			pixel_color = Some(color.0);
		} else if let Some(color_by_lifetime) = colors_by_lifetime.get(&entity) {
			let lifetime = lifetimes.get(&entity).unwrap();
			let percent_life = lifetime.life / lifetime.initial;
			pixel_color = Some(if percent_life >= 0.8 {
				color_by_lifetime.0
			} else if percent_life >= 0.6 {
				color_by_lifetime.1
			} else if percent_life >= 0.4 {
				color_by_lifetime.2
			} else if percent_life >= 0.2 {
				color_by_lifetime.3
			} else {
				color_by_lifetime.4
			});
		} else {
			pixel_color = None;
		}

		if let Some(color) = pixel_color {
			context.system.res.video.set_pixel(position.0.x as i32, position.0.y as i32, color);
		}
	}
}

fn event_handler(event: &Event, context: &mut Context) -> bool {
	match event {
		Event::Kill(entity) => {
			context.entities.remove_entity(*entity);
		}
		Event::CollideAgainstEdge(entity) => {
			let positions = context.entities.components::<Position>();
			let position = positions.get(entity).unwrap();
			let x = position.0.x + (BALL_SIZE / 2) as f32;
			let y = position.0.y + (BALL_SIZE / 2) as f32;
			drop(positions);
			new_bounce_particles(&mut context.entities, x, y);
		}
		Event::LeaveTrail(position) => {
			new_trail_particle_entity(&mut context.entities, position.x, position.y, TRAIL_PARTICLE_LIFETIME);
		}
	}
	false
}

pub fn init_entities(entities: &mut Entities) {
	entities.init_components::<Position>();
	entities.init_components::<Velocity>();
	entities.init_components::<SpriteIndex>();
	entities.init_components::<BouncesAgainstEdge>();
	entities.init_components::<Particle>();
	entities.init_components::<Color>();
	entities.init_components::<LifeLeft>();
	entities.init_components::<LeavesTrail>();
	entities.init_components::<ColorByLifeTime>();

	entities.remove_all_entities();
	for _ in 0..NUM_BALLS {
		new_ball_entity(entities);
	}
}

pub fn init_component_system(cs: &mut ComponentSystems<Context, Context>) {
	cs.add_update_system(update_system_movement);
	cs.add_update_system(update_system_collision);
	cs.add_update_system(update_system_lifetime);
	cs.add_update_system(update_system_leave_particle_trail);
	cs.add_render_system(render_system_particles);
	cs.add_render_system(render_system_sprites);
}

pub fn init_event_listeners(event_listeners: &mut EventListeners<Event, Context>) {
	event_listeners.add(event_handler);
}
