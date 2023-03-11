use ggdt::prelude::*;

use crate::{Core, TILE_HEIGHT, TILE_WIDTH};
use crate::entities::*;
use crate::tilemap::*;

pub fn remove_entity(entities: &mut Entities, entity: EntityId) {
	remove_entity_attachment(entities, entity);
	entities.remove_entity(entity);
}

pub fn remove_entity_attachment(entities: &mut Entities, entity: EntityId) {
	let attachments = entities.components::<Attachment>();
	if let Some(attachment) = attachments.get(&entity) {
		let attached_entity_id = attachment.0;
		drop(attachments);
		entities.remove_entity(attached_entity_id);
	}
}

pub fn move_entity_forward(context: &mut Core, entity: EntityId) {
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
		Direction::South => Vector2::DOWN * movement_speed.0
	};

	velocity.0 += movement;
}

pub fn turn_and_move_entity(context: &mut Core, entity: EntityId, direction: Direction) {
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

fn move_entity_with_collision(position: &mut Position, bounds: &Bounds, velocity: Option<&Velocity>, forces: Option<&Forces>, map: &TileMap, delta: f32) -> bool {
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

pub fn kill_entity(context: &mut Core, entity: EntityId) {
	context.entities.add_component(entity, LifeTime(5.0));
	context.entities.add_component(entity, TimedFlicker::new_with_pre_timer(5.0, 4.0, FlickerMethod::OnOff));
	context.entities.remove_component::<RandomlyWalksAround>(entity);
	context.entities.remove_component::<WalkingTime>(entity);
	context.entities.remove_component::<Attackable>(entity);
	context.entities.remove_component::<Pusher>(entity);
	context.entities.remove_component::<Pushable>(entity);
	set_entity_activity(&mut context.entities, entity, EntityActivity::Dead);
	spawn_pickups_from_entity(context, entity);
}

pub fn apply_damage_at(context: &mut Core, area: Circle, damage: i32, source: EntityId) {
	if let Some(attackables) = context.entities.components::<Attackable>() {
		let positions = context.entities.components::<Position>();
		let bounds = context.entities.components::<Bounds>();

		//let source_position = Vector2::new(area.x as f32, area.y as f32);
		let source_position = positions.get(&source).unwrap();

		for (entity, _) in attackables.iter() {
			// entity cannot (currently) attack itself ...
			if *entity == source {
				continue;
			}

			let position = positions.get(entity).unwrap();
			let bound = bounds.get(entity).unwrap();

			let circle = Circle::new(
				position.0.x as i32 + bound.width as i32 / 2,
				position.0.y as i32 + bound.height as i32 / 2,
				bound.radius,
			);
			if area.overlaps(&circle) {
				context.event_publisher.queue(Event::Hit(*entity, source, damage, source_position.0));
			}
		}
	}
}

pub fn get_attack_area_of_effect(context: &mut Core, attacker: EntityId) -> Option<(Circle, i32)> {
	let positions = context.entities.components::<Position>();
	let facing_directions = context.entities.components::<FacingDirection>();
	let bounds = context.entities.components::<Bounds>();
	let weapons = context.entities.components::<Weapon>();

	let position = positions.get(&attacker).unwrap();
	let bound = bounds.get(&attacker).unwrap();
	if let Some(weapon) = weapons.get(&attacker) {
		if let Some(facing_direction) = facing_directions.get(&attacker) {
			let center_point = position.0 + weapon.offsets[facing_direction.0 as usize];
			return Some((
				Circle::new(
					center_point.x as i32 + 8,
					center_point.y as i32 + 8,
					weapon.radius_of_effect,
				),
				weapon.damage
			));
		} else {
			return Some((
				Circle::new(
					position.0.x as i32 + bound.width as i32 / 2,
					position.0.y as i32 + bound.height as i32 / 2,
					weapon.radius_of_effect,
				),
				weapon.damage
			));
		}
	}

	None
}

pub fn attack(context: &mut Core, entity: EntityId) {
	let activities = context.entities.components::<Activity>();
	let activity = activities.get(&entity).unwrap();

	match activity.0 {
		EntityActivity::Idle | EntityActivity::Walking => {
			drop(activities);
			// set attacking animation and "extend" the entity's weapon
			set_entity_activity(&mut context.entities, entity, EntityActivity::Attacking);
			if new_weapon_attachment_entity(context, entity).is_some() {
				// if the entity's weapon was actually extended, figure out where it hits
				// and who is being hit by it
				if let Some((area_of_effect, damage)) = get_attack_area_of_effect(context, entity) {
					apply_damage_at(context, area_of_effect, damage, entity);
				}
			}
		}
		_ => {}
	}
}

pub fn hit_entity(context: &mut Core, target: EntityId, source: EntityId, damage: i32, damage_position: Vector2) {
	let position;
	{
		let positions = context.entities.components::<Position>();
		position = positions.get(&target).unwrap().0;

		// apply knockback force to target being hit
		let mut forces = context.entities.components_mut::<Forces>();
		if let Some(force) = forces.get_mut(&target) {
			let knockback_direction = (position - damage_position).normalize();
			force.add(knockback_direction * HIT_KNOCKBACK_STRENGTH, HIT_KNOCKBACK_DISSIPATION);
		}

		// subtract damage from entity life, and kill if necessary
		let mut lifes = context.entities.components_mut::<Life>();
		if let Some(life) = lifes.get_mut(&target) {
			life.0 -= damage;
			if life.0 <= 0 {
				context.event_publisher.queue(Event::Kill(target));
			}
		}
	}

	spawn_pixel_cloud(context, position.x as i32, position.y as i32, 8, 64.0, 0.15, 15);
	context.entities.add_component(target, TimedFlicker::new(0.5, FlickerMethod::Color(4)));
}

pub fn stop_attack(context: &mut Core, entity: EntityId) {
	// after an entity's attack has finished, they go back to idle and we "sheath" their weapon
	set_entity_activity(&mut context.entities, entity, EntityActivity::Idle);
	remove_entity_attachment(&mut context.entities, entity);
}

pub fn pickup(context: &mut Core, picked_up_by: EntityId, picked_up: EntityId) {
	let kind;
	let position;
	{
		let positions = context.entities.components::<Position>();
		position = positions.get(&picked_up).unwrap().0;

		let pickupables = context.entities.components::<Pickupable>();
		kind = pickupables.get(&picked_up).unwrap().kind;
	}

	// TODO: tally up the kinds

	new_sparkles_animation(context, position.x as i32, position.y as i32, None);
	remove_entity(&mut context.entities, picked_up);
}

///////////////////////////////////////////////////////////////////////////////////////////////////

fn update_system_movement(context: &mut Core) {
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

fn update_system_friction(context: &mut Core) {
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

fn update_system_force_decay(context: &mut Core) {
	let mut forces = context.entities.components_mut::<Forces>().unwrap();
	for (_, force) in forces.iter_mut() {
		force.decay();
	}
}

fn update_system_pushing(context: &mut Core) {
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

		for (pushable_entity, pushable) in pushable.iter() {
			// don't push ourself ...
			if *pushable_entity == *pusher_entity {
				continue;
			}

			let pushable_position = positions.get(pushable_entity).unwrap();
			let pushable_bounds = bounds.get(pushable_entity).unwrap();
			let pushable_circle = Circle::new(pushable_position.0.x as i32, pushable_position.0.y as i32, pushable_bounds.radius);

			if pusher_circle.overlaps(&pushable_circle) {
				let push_direction = (pushable_position.0 - pusher_position.0).normalize();

				let pushable_force = forces.get_mut(pushable_entity).unwrap();
				pushable_force.add(push_direction * pusher.strength, pusher.push_force_dissipation);
			}
		}
	}
}

fn update_system_lifetime(context: &mut Core) {
	let mut lifetimes = context.entities.components_mut::<LifeTime>().unwrap();
	for (entity, lifetime) in lifetimes.iter_mut() {
		lifetime.0 -= context.delta;
		if lifetime.0 < 0.0 {
			context.event_publisher.queue(Event::Remove(*entity));
		}
	}
}

fn update_system_animation(context: &mut Core) {
	let mut animations = context.entities.components_mut::<AnimationInstance>().unwrap();
	let kill_when_animation_finishes = context.entities.components::<KillWhenAnimationFinishes>();

	for (entity, animation) in animations.iter_mut() {
		if animation.complete {
			continue;
		}

		animation.frame_timer += context.delta;

		let delay = if let Some(delay_override) = animation.delay_override {
			delay_override
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
					if kill_when_animation_finishes.contains_key(entity) {
						context.event_publisher.queue(Event::Remove(*entity));
					}
				} else {
					animation.frame_index = 0;
				}
			} else {
				animation.frame_index += 1;
			}
		}
	}
}

fn update_system_set_sprite_index_from_animation(context: &mut Core) {
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

fn update_system_set_sprite_index_by_direction(context: &mut Core) {
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

fn update_system_walking_time(context: &mut Core) {
	let mut walking_times = context.entities.components_mut::<WalkingTime>().unwrap();
	let activities = context.entities.components::<Activity>();

	for (entity, walking_time) in walking_times.iter_mut() {
		if let Some(activity) = activities.get(entity) {
			// dead entities can't walk!
			if activity.0 == EntityActivity::Dead {
				continue;
			}
		}
		if walking_time.0 > 0.0 {
			walking_time.0 -= context.delta;
			context.event_publisher.queue(Event::MoveForward(*entity));
		}
	}

	// remove walking time components whose timers have elapsed
	walking_times.retain(|_, comp| comp.0 > 0.0);
}

fn update_system_randomly_walk_around(context: &mut Core) {
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
					randomly_walk_around.cooldown_timer = rnd_value(randomly_walk_around.min_cooldown, randomly_walk_around.max_cooldown);

					let direction = Direction::new_random();
					let walk_time = rnd_value(randomly_walk_around.min_walk_time, randomly_walk_around.max_walk_time);

					walking_times.insert(*entity, WalkingTime(walk_time));
					context.event_publisher.queue(Event::TurnAndMove(*entity, direction));
				}
			}
		}
	}
}

fn update_system_current_entity_activity(context: &mut Core) {
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
				_ => {}
			}
		}
	}
}

fn update_system_randomly_spawn_slimes(context: &mut Core) {
	if let Some((entity, _)) = context.entities.components::<World>().single() {
		let mut spawn_timers = context.entities.components_mut::<SpawnTimer>();
		if let Some(spawn_timer) = spawn_timers.get_mut(entity) {
			spawn_timer.timer -= context.delta;
			if spawn_timer.timer <= 0.0 {
				spawn_timer.reset_timer();
				let slime_count = context.entities.components::<Slime>().len();
				if slime_count < spawn_timer.max_allowed {
					context.event_publisher.queue(Event::SpawnSlimeRandomly);
				}
			}
		}
	}
}

fn update_system_camera_follows_player(context: &mut Core) {
	if let Some((player_entity, _)) = context.entities.components::<Player>().single() {
		if let Some((_, mut camera)) = context.entities.components_mut::<Camera>().single_mut() {
			let positions = context.entities.components::<Position>().unwrap();
			let position = positions.get(player_entity).unwrap();

			let camera_x = position.0.x as i32 - (SCREEN_WIDTH as i32 / 2) + 8;
			let camera_y = position.0.y as i32 - (SCREEN_HEIGHT as i32 / 2) + 8;

			// clamp camera position to the map boundaries
			let map_pixel_width = context.tilemap.width() * TILE_WIDTH;
			let map_pixel_height = context.tilemap.height() * TILE_HEIGHT;
			let max_camera_x = map_pixel_width - SCREEN_WIDTH;
			let max_camera_y = map_pixel_height - SCREEN_HEIGHT;

			camera.x = camera_x.clamp(0, max_camera_x as i32);
			camera.y = camera_y.clamp(0, max_camera_y as i32);
		}
	}
}

fn update_system_turn_attached_entities(context: &mut Core) {
	let attachments = context.entities.components::<Attachment>().unwrap();
	let mut facing_directions = context.entities.components_mut::<FacingDirection>();

	for (parent_entity, attachment) in attachments.iter() {
		// the parent may not have a facing direction. and if so, we don't need to change the
		// attachment (child)
		let parent_facing_direction = if let Some(facing_direction) = facing_directions.get(&parent_entity) {
			facing_direction.0
		} else {
			continue;
		};

		// change the direction of the attachment (child) to match the parent ... if the
		// attachment even has a direction itself ...
		if let Some(mut facing_direction) = facing_directions.get_mut(&attachment.0) {
			facing_direction.0 = parent_facing_direction;
		}
	}
}

fn update_system_position_attached_entities(context: &mut Core) {
	let attachments = context.entities.components::<Attachment>().unwrap();
	let mut positions = context.entities.components_mut::<Position>();
	let facing_directions = context.entities.components::<FacingDirection>();
	let offsets = context.entities.components::<AttachmentOffset>();
	let offset_by_directions = context.entities.components::<AttachmentOffsetByDirection>();

	for (parent_entity, attachment) in attachments.iter() {
		// get the parent position used as the base for the attached (child) entity. if the
		// parent doesn't have one (probably it is dead?), then skip this attachment
		let parent_position;
		if let Some(position) = positions.get(&parent_entity) {
			parent_position = position.0;
		} else {
			continue;
		}

		let attached_entity = attachment.0;
		if let Some(mut attachment_position) = positions.get_mut(&attached_entity) {
			// start off the attachment by placing it directly at the parent
			attachment_position.0 = parent_position;

			// then add whatever position offset it needs
			if let Some(offset) = offsets.get(&attached_entity) {
				attachment_position.0 += offset.0;
			} else if let Some(offset_by_direction) = offset_by_directions.get(&attached_entity) {
				if let Some(facing_direction) = facing_directions.get(&attached_entity) {
					attachment_position.0 += offset_by_direction.offsets[facing_direction.0 as usize];
				}
			}
		}
	}
}

fn update_system_timed_flicker(context: &mut Core) {
	let mut timed_flickers = context.entities.components_mut::<TimedFlicker>().unwrap();
	for (_, flicker) in timed_flickers.iter_mut() {
		flicker.update(context.delta);
	}
	timed_flickers.retain(|_, flicker| flicker.timer > 0.0);
}

fn update_system_pickups(context: &mut Core) {
	let mut pickupables = context.entities.components_mut::<Pickupable>().unwrap();
	let pickupers = context.entities.components::<Pickuper>().unwrap();
	let positions = context.entities.components::<Position>();
	let bounds = context.entities.components::<Bounds>();

	// don't really think this pre_timer thing is necessary anymore ... ?
	for (_, pickupable) in pickupables.iter_mut() {
		if pickupable.pre_timer > 0.0 {
			pickupable.pre_timer -= context.delta;
		}
	}

	// TODO: this is slow

	for (pickuper_entity, _) in pickupers.iter() {
		let pickuper_position = positions.get(pickuper_entity).unwrap();
		let pickuper_bounds = bounds.get(pickuper_entity).unwrap();
		let pickuper_circle = Circle::new(
			pickuper_position.0.x as i32 + pickuper_bounds.width as i32 / 2,
			pickuper_position.0.y as i32 + pickuper_bounds.height as i32 / 2,
			pickuper_bounds.radius,
		);

		for (pickupable_entity, pickupable) in pickupables.iter() {
			if pickupable.pre_timer <= 0.0 {
				let pickupable_position = positions.get(pickupable_entity).unwrap();
				let pickupable_bounds = bounds.get(pickupable_entity).unwrap();
				let pickupable_circle = Circle::new(
					pickupable_position.0.x as i32 + pickupable_bounds.width as i32 / 2,
					pickupable_position.0.y as i32 + pickupable_bounds.height as i32 / 2,
					pickupable_bounds.radius,
				);

				if pickupable_circle.overlaps(&pickuper_circle) {
					context.event_publisher.queue(Event::Pickup(*pickuper_entity, *pickupable_entity));
				}
			}
		}
	}
}

///////////////////////////////////////////////////////////////////////////////////////////////////

fn render_system_sprites(context: &mut Core) {
	context.sprite_render_list.clear();

	let sprites = context.entities.components::<Sprite>().unwrap();
	let positions = context.entities.components::<Position>().unwrap();
	let timed_flickers = context.entities.components::<TimedFlicker>().unwrap();

	if let Some((_, camera)) = context.entities.components::<Camera>().single() {
		// build up list of entities to be rendered with their positions so we can sort them
		// and render these entities with a proper y-based sort order
		for (entity, _) in sprites.iter() {
			let mut blit_method = IndexedBlitMethod::Transparent(0);

			// check for flicker effects
			if let Some(flicker) = timed_flickers.get(entity) {
				if !flicker.flick {
					match flicker.method {
						FlickerMethod::OnOff => {
							// skip to the next entity, this one isn't visible
							continue;
						}
						FlickerMethod::Color(draw_color) => {
							blit_method = IndexedBlitMethod::TransparentSingle {
								transparent_color: 0,
								draw_color,
							};
						}
					}
				}
			}

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
				position.x as i32 - camera.x,
				position.y as i32 - camera.y,
			);
		}
	}
}

fn render_system_pixels(context: &mut Core) {
	let pixels = context.entities.components::<crate::entities::Pixel>().unwrap();
	let positions = context.entities.components::<Position>();

	if let Some((_, camera)) = context.entities.components::<Camera>().single() {
		for (entity, pixel) in pixels.iter() {
			if let Some(position) = positions.get(entity) {
				context.system.res.video.set_pixel(
					position.0.x as i32 - camera.x,
					position.0.y as i32 - camera.y,
					pixel.0,
				);
			}
		}
	}
}

pub fn init_component_system(cs: &mut ComponentSystems<Core, Core>) {
	cs.reset();
	cs.add_update_system(update_system_lifetime);
	cs.add_update_system(update_system_current_entity_activity);
	cs.add_update_system(update_system_walking_time);
	cs.add_update_system(update_system_pushing);
	cs.add_update_system(update_system_movement);
	cs.add_update_system(update_system_turn_attached_entities);
	cs.add_update_system(update_system_position_attached_entities);
	cs.add_update_system(update_system_friction);
	cs.add_update_system(update_system_force_decay);
	cs.add_update_system(update_system_randomly_walk_around);
	cs.add_update_system(update_system_animation);
	cs.add_update_system(update_system_set_sprite_index_from_animation);
	cs.add_update_system(update_system_set_sprite_index_by_direction);
	cs.add_update_system(update_system_randomly_spawn_slimes);
	cs.add_update_system(update_system_camera_follows_player);
	cs.add_update_system(update_system_timed_flicker);
	cs.add_update_system(update_system_pickups);
	cs.add_render_system(render_system_sprites);
	cs.add_render_system(render_system_pixels);
}
