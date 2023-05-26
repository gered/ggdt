use std::collections::HashMap;
use std::rc::Rc;

use ggdt::prelude::*;

use crate::context::{CoreContext, GameContext};
use crate::tilemap::TileMap;

pub const FRICTION: f32 = 0.5;
pub const DEFAULT_PUSH_STRENGTH: f32 = 0.5;
pub const DEFAULT_PUSH_DISSIPATION: f32 = 0.5;

///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum EntityActivity {
	Idle,
	Walking,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
	South = 0,
	West = 1,
	East = 2,
	North = 3,
}

impl Direction {
	pub fn new_random() -> Self {
		use Direction::*;
		match rnd_value(0, 3) {
			0 => South,
			1 => West,
			2 => East,
			3 => North,
			_ => panic!("unknown random direction!"),
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SlimeColor {
	Green = 0,
	Blue = 1,
	Orange = 2,
}

impl SlimeColor {
	pub fn new_random() -> Self {
		use SlimeColor::*;
		match rnd_value(0, 2) {
			0 => Green,
			1 => Blue,
			2 => Orange,
			_ => panic!("unknown random slime color!"),
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Force {
	pub force: Vector2,
	pub dissipation_factor: f32,
}

///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Activity(pub EntityActivity);

pub struct AnimateByActivity(pub Rc<HashMap<EntityActivity, Rc<AnimationDef>>>);

#[derive(Debug)]
pub struct AnimationDef {
	pub frames: &'static [usize],
	pub loops: bool,
	pub delay: f32,
	pub multi_direction_offset: Option<usize>,
}

impl AnimationDef {
	#[inline]
	pub fn new(frames: &'static [usize], loops: bool, delay: f32, multi_direction_offset: Option<usize>) -> Self {
		AnimationDef { frames, loops, delay, multi_direction_offset }
	}
}

#[derive(Debug)]
pub struct AnimationInstance {
	pub def: Rc<AnimationDef>,
	pub frame_index: usize,
	pub frame_timer: f32,
	pub complete: bool,
	pub delay_override: Option<f32>,
}

impl AnimationInstance {
	#[inline]
	pub fn from(def: Rc<AnimationDef>) -> Self {
		AnimationInstance {
			def, //
			frame_index: 0,
			frame_timer: 0.0,
			complete: false,
			delay_override: None,
		}
	}

	#[inline]
	pub fn change_to(&mut self, def: Rc<AnimationDef>) {
		self.def = def;
		self.reset();
	}

	#[inline]
	pub fn reset(&mut self) {
		self.frame_index = 0;
		self.frame_timer = 0.0;
		self.complete = false;
	}
}

pub struct Bounds {
	pub width: u32,
	pub height: u32,
	pub radius: u32,
}

pub struct FacingDirection(pub Direction);

pub struct Forces {
	pub forces: Vec<Force>,
}

impl Forces {
	pub fn new() -> Self {
		Forces { forces: Vec::with_capacity(5) }
	}

	pub fn current_force(&self) -> Vector2 {
		let mut total_force = Vector2::ZERO;
		for force in self.forces.iter() {
			total_force += force.force;
		}
		total_force
	}

	pub fn add(&mut self, force: Vector2, dissipation_factor: f32) {
		self.forces.push(Force { force, dissipation_factor });
	}

	pub fn decay(&mut self) {
		for force in self.forces.iter_mut() {
			force.force *= force.dissipation_factor;
		}
		self.forces.retain(|f| !f.force.almost_zero(0.001));
	}
}

pub struct IgnoresCollision;

pub struct IgnoresFriction;

pub struct MovementSpeed(pub f32);

pub struct Position(pub Vector2);

pub struct Pushable;

pub struct Pusher {
	pub strength: f32,
	pub push_force_dissipation: f32,
}

impl Pusher {
	pub fn new() -> Self {
		Pusher {
			strength: DEFAULT_PUSH_STRENGTH, //
			push_force_dissipation: DEFAULT_PUSH_DISSIPATION,
		}
	}
}

pub struct RandomlyWalksAround {
	pub min_walk_time: f32,
	pub max_walk_time: f32,
	pub chance_to_move: u32,
	pub min_cooldown: f32,
	pub max_cooldown: f32,
	pub cooldown_timer: f32,
}

impl RandomlyWalksAround {
	pub fn new(
		min_walk_time: f32,
		max_walk_time: f32,
		chance_to_move: u32,
		min_cooldown: f32,
		max_cooldown: f32,
	) -> Self {
		RandomlyWalksAround {
			min_walk_time,
			max_walk_time,
			chance_to_move,
			min_cooldown,
			max_cooldown,
			cooldown_timer: 0.0,
		}
	}

	pub fn should_start_walking(&self) -> bool {
		rnd_value(0, 100) < self.chance_to_move
	}
}

pub struct Slime(pub SlimeColor);

pub struct Sprite {
	pub atlas: Rc<BitmapAtlas<RgbaBitmap>>,
	pub index: usize,
}

pub struct SpriteIndexByDirection {
	pub base_index: usize,
}

pub struct Velocity(pub Vector2);

pub struct WalkingTime(pub f32);

pub fn init_entities(entities: &mut Entities) {
	entities.init_components::<Activity>();
	entities.init_components::<AnimateByActivity>();
	entities.init_components::<AnimationDef>();
	entities.init_components::<AnimationInstance>();
	entities.init_components::<Bounds>();
	entities.init_components::<FacingDirection>();
	entities.init_components::<Forces>();
	entities.init_components::<IgnoresCollision>();
	entities.init_components::<IgnoresFriction>();
	entities.init_components::<MovementSpeed>();
	entities.init_components::<Position>();
	entities.init_components::<Pushable>();
	entities.init_components::<Pusher>();
	entities.init_components::<RandomlyWalksAround>();
	entities.init_components::<Slime>();
	entities.init_components::<Sprite>();
	entities.init_components::<SpriteIndexByDirection>();
	entities.init_components::<Velocity>();
	entities.init_components::<WalkingTime>();
	entities.remove_all_entities();
}

///////////////////////////////////////////////////////////////////////////////////////////////////

pub enum Event {
	AnimationFinished(EntityId),
	MoveForward(EntityId),
	Remove(EntityId),
	SetActivity(EntityId, EntityActivity),
	TurnAndMove(EntityId, Direction),
}

fn event_handler(event: &Event, context: &mut CoreContext) -> bool {
	match event {
		Event::AnimationFinished(_entity) => {
			// no-op
		}
		Event::MoveForward(entity) => {
			if context.entities.has_entity(*entity) {
				move_entity_forward(context, *entity);
			}
		}
		Event::Remove(entity) => {
			if context.entities.has_entity(*entity) {
				remove_entity(&mut context.entities, *entity);
			}
		}
		Event::SetActivity(entity, activity) => {
			if context.entities.has_entity(*entity) {
				set_entity_activity(&mut context.entities, *entity, *activity);
			}
		}
		Event::TurnAndMove(entity, direction) => {
			if context.entities.has_entity(*entity) {
				turn_and_move_entity(context, *entity, *direction);
			}
		}
	};
	false
}

pub fn init_events(event_listener: &mut EventListeners<Event, CoreContext>) {
	event_listener.clear();
	event_listener.add(event_handler);
}

///////////////////////////////////////////////////////////////////////////////////////////////////

pub fn move_entity_forward(context: &mut CoreContext, entity: EntityId) {
	let mut velocities = context.entities.components_mut::<Velocity>();
	let facing_directions = context.entities.components::<FacingDirection>();
	let movement_speeds = context.entities.components::<MovementSpeed>();

	let velocity = velocities.get_mut(&entity).unwrap();
	let facing_direction = facing_directions.get(&entity).unwrap();
	let movement_speed = movement_speeds.get(&entity).unwrap();

	let movement = match facing_direction.0 {
		Direction::North => Vector2::UP * movement_speed.0,
		Direction::East => Vector2::RIGHT * movement_speed.0,
		Direction::West => Vector2::LEFT * movement_speed.0,
		Direction::South => Vector2::DOWN * movement_speed.0,
	};

	velocity.0 += movement;
}

fn move_entity_with_collision(
	position: &mut Position,
	bounds: &Bounds,
	velocity: Option<&Velocity>,
	forces: Option<&Forces>,
	map: &TileMap,
	delta: f32,
) -> bool {
	const NUM_STEPS: usize = 2;
	const STEP_SCALE: f32 = 1.0 / NUM_STEPS as f32;

	let mut collided = false;

	// apply entity velocity + force (if any/either) and exit early with no collision if this entity
	// has no movement ... no need to check collisions in such a case
	let mut step_velocity = Vector2::ZERO;
	if let Some(velocity) = velocity {
		step_velocity += velocity.0 * delta;
	}
	if let Some(forces) = forces {
		step_velocity += forces.current_force();
	}
	if step_velocity.nearly_equal(Vector2::ZERO, 0.00001) {
		return collided;
	}

	// entity is actually moving, so check collisions and move accordingly
	step_velocity *= STEP_SCALE;
	for _ in 0..NUM_STEPS {
		let old_position = position.0;

		position.0.x += step_velocity.x;
		if map.is_colliding(&Rect::new(position.0.x as i32, position.0.y as i32, bounds.width, bounds.height)) {
			collided = true;
			position.0.x = old_position.x;
		}

		position.0.y += step_velocity.y;
		if map.is_colliding(&Rect::new(position.0.x as i32, position.0.y as i32, bounds.width, bounds.height)) {
			collided = true;
			position.0.y = old_position.y;
		}
	}

	collided
}

pub fn remove_entity(entities: &mut Entities, entity: EntityId) {
	entities.remove_entity(entity);
}

pub fn set_entity_activity(entities: &mut Entities, entity: EntityId, new_activity: EntityActivity) {
	let mut activities = entities.components_mut::<Activity>();
	let mut activity = activities.get_mut(&entity).unwrap();

	// only change the activity, and more importantly, the animation if we are actually applying
	// an actual activity change from what it was before
	if activity.0 != new_activity {
		activity.0 = new_activity;

		let animate_by_activitys = entities.components::<AnimateByActivity>();
		if let Some(animate_by_activity) = animate_by_activitys.get(&entity) {
			if let Some(new_animation_def) = animate_by_activity.0.get(&new_activity) {
				let mut animations = entities.components_mut::<AnimationInstance>();
				let animation = animations.get_mut(&entity).unwrap();
				animation.change_to(new_animation_def.clone());
			}
		}
	}
}

pub fn turn_and_move_entity(context: &mut CoreContext, entity: EntityId, direction: Direction) {
	// can this entity currently move at all?
	let activities = context.entities.components::<Activity>();
	if let Some(activity) = activities.get(&entity) {
		if activity.0 != EntityActivity::Idle && activity.0 != EntityActivity::Walking {
			return;
		}
	}
	drop(activities);

	// make the entity face in the direction specified
	let mut facing_directions = context.entities.components_mut::<FacingDirection>();
	let facing_direction = facing_directions.get_mut(&entity).unwrap();
	facing_direction.0 = direction;
	drop(facing_directions);

	move_entity_forward(context, entity);
}

///////////////////////////////////////////////////////////////////////////////////////////////////

fn update_system_movement(context: &mut CoreContext) {
	let mut positions = context.entities.components_mut::<Position>().unwrap();
	let velocities = context.entities.components::<Velocity>();
	let forces = context.entities.components::<Forces>();
	let bounds = context.entities.components::<Bounds>();
	let ignores_collision = context.entities.components::<IgnoresCollision>();

	for (entity, position) in positions.iter_mut() {
		if ignores_collision.contains_key(entity) {
			if let Some(velocity) = velocities.get(entity) {
				position.0 += velocity.0 * context.delta;
			}
		} else {
			let velocity = velocities.get(entity);
			let force = forces.get(entity);

			if velocity.is_some() || force.is_some() {
				move_entity_with_collision(
					position,
					bounds.get(entity).unwrap(),
					velocity,
					force,
					&context.tilemap,
					context.delta,
				);
			}
		}
	}
}

fn update_system_friction(context: &mut CoreContext) {
	let mut velocities = context.entities.components_mut::<Velocity>().unwrap();
	let ignores_friction = context.entities.components::<IgnoresFriction>();

	for (entity, velocity) in velocities.iter_mut() {
		if !ignores_friction.contains_key(entity) {
			velocity.0 *= FRICTION;
			if velocity.0.almost_zero(0.001) {
				velocity.0 = Vector2::ZERO;
			}
		}
	}
}

fn update_system_force_decay(context: &mut CoreContext) {
	let mut forces = context.entities.components_mut::<Forces>().unwrap();
	for (_, force) in forces.iter_mut() {
		force.decay();
	}
}

fn update_system_pushing(context: &mut CoreContext) {
	let positions = context.entities.components::<Position>();
	let bounds = context.entities.components::<Bounds>();
	let mut forces = context.entities.components_mut::<Forces>();
	let pushers = context.entities.components::<Pusher>().unwrap();
	let pushable = context.entities.components::<Pushable>().unwrap();

	// TODO: this is slow

	for (pusher_entity, pusher) in pushers.iter() {
		let pusher_position = positions.get(pusher_entity).unwrap();
		let pusher_bounds = bounds.get(pusher_entity).unwrap();
		let pusher_circle = Circle::new(pusher_position.0.x as i32, pusher_position.0.y as i32, pusher_bounds.radius);

		for (pushable_entity, _pushable) in pushable.iter() {
			// don't push ourself ...
			if *pushable_entity == *pusher_entity {
				continue;
			}

			let pushable_position = positions.get(pushable_entity).unwrap();
			let pushable_bounds = bounds.get(pushable_entity).unwrap();
			let pushable_circle = Circle::new(
				pushable_position.0.x as i32, //
				pushable_position.0.y as i32,
				pushable_bounds.radius,
			);

			if pusher_circle.overlaps(&pushable_circle) {
				let mut push_direction = (pushable_position.0 - pusher_position.0).normalize();
				// this can happen if the pusher's and pushable's positions are exactly the same. we just need to
				// "break the tie" so to speak ...
				if !push_direction.x.is_normal() || !push_direction.y.is_normal() {
					push_direction = Vector2::UP; // TODO: use one of their facing directions? which one?
				}

				let pushable_force = forces.get_mut(pushable_entity).unwrap();
				pushable_force.add(push_direction * pusher.strength, pusher.push_force_dissipation);
			}
		}
	}
}

fn update_system_animation(context: &mut CoreContext) {
	let mut animations = context.entities.components_mut::<AnimationInstance>().unwrap();

	for (entity, animation) in animations.iter_mut() {
		if animation.complete {
			continue;
		}

		animation.frame_timer += context.delta;

		let delay = if let Some(delay_override) = animation.delay_override {
			delay_override //
		} else {
			animation.def.delay
		};

		if animation.frame_timer >= delay {
			// move to the next frame in the current sequence
			animation.frame_timer = 0.0;
			if animation.frame_index == (animation.def.frames.len() - 1) {
				// we're at the last frame in the current sequence
				if !animation.def.loops {
					animation.complete = true;
					context.event_publisher.queue(Event::AnimationFinished(*entity));
				} else {
					animation.frame_index = 0;
				}
			} else {
				animation.frame_index += 1;
			}
		}
	}
}

fn update_system_set_sprite_index_from_animation(context: &mut CoreContext) {
	let animations = context.entities.components::<AnimationInstance>().unwrap();
	let mut sprites = context.entities.components_mut::<Sprite>();
	let facing_directions = context.entities.components::<FacingDirection>();

	for (entity, animation) in animations.iter() {
		if let Some(sprite) = sprites.get_mut(entity) {
			// base animation sprite-sheet index for the current animation state
			let mut index = animation.def.frames[animation.frame_index];

			// add multi-direction offset if applicable
			let multi_direction_offset = animation.def.multi_direction_offset;
			let facing_direction = facing_directions.get(entity);
			if multi_direction_offset.is_some() && facing_direction.is_some() {
				index += multi_direction_offset.unwrap() * facing_direction.unwrap().0 as usize;
			}

			sprite.index = index;
		}
	}
}

fn update_system_set_sprite_index_by_direction(context: &mut CoreContext) {
	let sprite_index_by_directions = context.entities.components::<SpriteIndexByDirection>().unwrap();
	let mut sprites = context.entities.components_mut::<Sprite>();
	let facing_directions = context.entities.components::<FacingDirection>();

	for (entity, sprite_index_by_direction) in sprite_index_by_directions.iter() {
		if let Some(sprite) = sprites.get_mut(entity) {
			if let Some(facing_direction) = facing_directions.get(entity) {
				sprite.index = sprite_index_by_direction.base_index + facing_direction.0 as usize;
			}
		}
	}
}

fn update_system_walking_time(context: &mut CoreContext) {
	let mut walking_times = context.entities.components_mut::<WalkingTime>().unwrap();

	for (entity, walking_time) in walking_times.iter_mut() {
		if walking_time.0 > 0.0 {
			walking_time.0 -= context.delta;
			context.event_publisher.queue(Event::MoveForward(*entity));
		}
	}

	// remove walking time components whose timers have elapsed
	walking_times.retain(|_, comp| comp.0 > 0.0);
}

fn update_system_randomly_walk_around(context: &mut CoreContext) {
	let mut randomly_walk_arounds = context.entities.components_mut::<RandomlyWalksAround>().unwrap();
	let activities = context.entities.components::<Activity>();
	let mut walking_times = context.entities.components_mut::<WalkingTime>().unwrap();

	for (entity, randomly_walk_around) in randomly_walk_arounds.iter_mut() {
		if let Some(activity) = activities.get(entity) {
			if activity.0 == EntityActivity::Idle {
				if randomly_walk_around.cooldown_timer > 0.0 {
					randomly_walk_around.cooldown_timer -= context.delta;
					if randomly_walk_around.cooldown_timer < 0.0 {
						randomly_walk_around.cooldown_timer = 0.0;
					}
				} else if randomly_walk_around.should_start_walking() {
					randomly_walk_around.cooldown_timer = rnd_value(
						randomly_walk_around.min_cooldown, //
						randomly_walk_around.max_cooldown,
					);

					let direction = Direction::new_random();
					let walk_time = rnd_value(randomly_walk_around.min_walk_time, randomly_walk_around.max_walk_time);

					walking_times.insert(*entity, WalkingTime(walk_time));
					context.event_publisher.queue(Event::TurnAndMove(*entity, direction));
				}
			}
		}
	}
}

fn update_system_current_entity_activity(context: &mut CoreContext) {
	let activities = context.entities.components::<Activity>().unwrap();
	let velocities = context.entities.components::<Velocity>();

	for (entity, activity) in activities.iter() {
		// try to detect current entity activity based on it's own movement speed
		// (intentionally NOT checking force velocity!)
		if let Some(velocity) = velocities.get(entity) {
			match activity.0 {
				EntityActivity::Idle => {
					if velocity.0.length_squared() > 0.0 {
						context.event_publisher.queue(Event::SetActivity(*entity, EntityActivity::Walking));
					}
				}
				EntityActivity::Walking => {
					if velocity.0.almost_zero(0.001) {
						context.event_publisher.queue(Event::SetActivity(*entity, EntityActivity::Idle));
					}
				}
			}
		}
	}
}

fn render_system_sprites(context: &mut CoreContext) {
	context.sprite_render_list.clear();

	let sprites = context.entities.components::<Sprite>().unwrap();
	let positions = context.entities.components::<Position>().unwrap();

	// build up list of entities to be rendered with their positions so we can sort them
	// and render these entities with a proper y-based sort order
	for (entity, _) in sprites.iter() {
		let blit_method = RgbaBlitMethod::Transparent(context.transparent_color);

		let position = positions.get(entity).unwrap();
		context.sprite_render_list.push((*entity, position.0, blit_method));
	}
	context.sprite_render_list.sort_unstable_by(|a, b| (a.1.y as i32).cmp(&(b.1.y as i32)));

	// now render them in the correct order ...
	for (entity, position, blit_method) in context.sprite_render_list.iter() {
		let sprite = sprites.get(entity).unwrap();
		context.system.res.video.blit_atlas(
			blit_method.clone(),
			&sprite.atlas,
			sprite.index,
			position.x as i32 - context.camera_x,
			position.y as i32 - context.camera_y,
		);
	}
}

fn render_system_entity_ids(context: &mut CoreContext) {
	let sprites = context.entities.components::<Sprite>().unwrap();
	let positions = context.entities.components::<Position>().unwrap();

	for (entity, _) in sprites.iter() {
		let position = positions.get(entity).unwrap();

		let x = position.0.x as i32 - context.camera_x;
		let y = position.0.y as i32 - context.small_font.line_height() as i32 - context.camera_y;
		context.system.res.video.print_string(
			&entity.to_string(),
			x,
			y,
			FontRenderOpts::Color(context.palette[15]),
			&context.small_font,
		);
	}
}

pub fn init_component_system(cs: &mut ComponentSystems<CoreContext, CoreContext>) {
	cs.reset();
	cs.add_update_system(update_system_current_entity_activity);
	cs.add_update_system(update_system_walking_time);
	cs.add_update_system(update_system_pushing);
	cs.add_update_system(update_system_movement);
	cs.add_update_system(update_system_friction);
	cs.add_update_system(update_system_force_decay);
	cs.add_update_system(update_system_randomly_walk_around);
	cs.add_update_system(update_system_animation);
	cs.add_update_system(update_system_set_sprite_index_from_animation);
	cs.add_update_system(update_system_set_sprite_index_by_direction);
	cs.add_render_system(render_system_sprites);
	cs.add_render_system(render_system_entity_ids);
}

///////////////////////////////////////////////////////////////////////////////////////////////////

pub fn init(context: &mut GameContext) {
	init_entities(&mut context.core.entities);
	init_component_system(&mut context.support.component_systems);
	init_events(&mut context.support.event_listeners);
	context.core.event_publisher.clear();
}

pub fn new_slime_entity(
	context: &mut CoreContext,
	x: i32,
	y: i32,
	direction: Direction,
	color: SlimeColor,
) -> EntityId {
	let id = context.entities.new_entity();

	let (
		atlas, //
		chance_to_move,
		movement_speed,
		min_walk_time,
		max_walk_time,
		min_walk_cooldown,
		max_walk_cooldown,
	) = match color {
		SlimeColor::Green => (context.green_slime.clone(), 10, 8.0, 0.5, 2.0, 0.5, 5.0),
		SlimeColor::Blue => (context.blue_slime.clone(), 40, 12.0, 0.5, 2.0, 0.5, 3.0),
		SlimeColor::Orange => (context.orange_slime.clone(), 90, 24.0, 0.5, 1.0, 0.5, 2.0),
	};

	let activity = EntityActivity::Idle;
	let animate_by_activity = AnimateByActivity(context.slime_activity_states.clone());
	let animation = AnimationInstance::from(animate_by_activity.0.get(&activity).unwrap().clone());

	context.entities.add_component(id, Slime(color));
	context.entities.add_component(id, Position(Vector2::new(x as f32, y as f32)));
	context.entities.add_component(id, Velocity(Vector2::ZERO));
	context.entities.add_component(id, Forces::new());
	context.entities.add_component(id, Bounds { width: 16, height: 16, radius: 8 });
	context.entities.add_component(id, FacingDirection(direction));
	context.entities.add_component(id, Sprite { atlas, index: 0 });
	context.entities.add_component(id, Activity(activity));
	context.entities.add_component(id, animate_by_activity);
	context.entities.add_component(id, animation);
	context.entities.add_component(
		id,
		RandomlyWalksAround::new(
			min_walk_time, //
			max_walk_time,
			chance_to_move,
			min_walk_cooldown,
			max_walk_cooldown,
		),
	);
	context.entities.add_component(id, MovementSpeed(movement_speed));
	context.entities.add_component(id, Pusher::new());
	context.entities.add_component(id, Pushable);

	id
}
