use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use libretrogd::entities::*;
use libretrogd::graphics::*;
use libretrogd::math::*;
use libretrogd::utils::rnd_value;

use crate::{Core, Game, TILE_HEIGHT, TILE_WIDTH, TileMap};

pub use self::events::*;
pub use self::systems::*;

pub mod events;
pub mod systems;

pub const FRICTION: f32 = 0.5;

pub const DEFAULT_PUSH_STRENGTH: f32 = 0.5;
pub const DEFAULT_PUSH_DISSIPATION: f32 = 0.5;

pub const HIT_KNOCKBACK_STRENGTH: f32 = 8.0;
pub const HIT_KNOCKBACK_DISSIPATION: f32 = 0.5;

pub const PICKUP_PRE_TIMER: f32 = 0.5;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum EntityActivity {
    Idle,
    Walking,
    Attacking,
    Dead,
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
            _ => panic!("unknown random direction!")
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
            _ => panic!("unknown random slime color!")
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum PickupType {
    GreenGem,
    BlueGem,
    OrangeGem,
    Coin,
}

impl PickupType {
    pub fn new_random() -> Self {
        use PickupType::*;
        match rnd_value(0, 3) {
            0 => GreenGem,
            1 => BlueGem,
            2 => OrangeGem,
            3 => Coin,
            _ => panic!("unknown random pickup type!")
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Force {
    pub force: Vector2,
    pub dissipation_factor: f32,
}

///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Player;

pub struct Slime;

pub struct Activity(pub EntityActivity);

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
        AnimationDef {
            frames,
            loops,
            delay,
            multi_direction_offset,
        }
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
            def,
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

pub struct AnimateByActivity(pub Rc<HashMap<EntityActivity, Rc<AnimationDef>>>);

pub struct KillWhenAnimationFinishes;

pub struct Position(pub Vector2);

pub struct Velocity(pub Vector2);

pub struct Forces {
    pub forces: Vec<Force>,
}

impl Forces {
    pub fn new() -> Self {
        Forces {
            forces: Vec::with_capacity(5),
        }
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

pub struct Bounds {
    pub width: u32,
    pub height: u32,
    pub radius: u32,
}

pub struct FacingDirection(pub Direction);

pub struct IgnoresCollision;

pub struct IgnoresFriction;

pub struct Particle;

pub struct LifeTime(pub f32);

pub struct Pixel(pub u8);

pub struct Sprite {
    pub atlas: Rc<BitmapAtlas>,
    pub index: usize,
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
    pub fn new(min_walk_time: f32, max_walk_time: f32, chance_to_move: u32, min_cooldown: f32, max_cooldown: f32) -> Self {
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

pub struct WalkingTime(pub f32);

pub struct MovementSpeed(pub f32);

pub struct World;

pub struct SpawnTimer {
    pub timer: f32,
    pub min_time: f32,
    pub max_time: f32,
    pub max_allowed: usize,
}

impl SpawnTimer {
    pub fn new(min_time: f32, max_time: f32, max_allowed: usize) -> Self {
        SpawnTimer {
            timer: 0.0,
            min_time,
            max_time,
            max_allowed,
        }
    }

    pub fn reset_timer(&mut self) {
        self.timer = rnd_value(self.min_time, self.max_time);
    }
}

pub struct Camera {
    pub x: i32,
    pub y: i32,
}

pub struct Pushable;

pub struct Pusher {
    pub strength: f32,
    pub push_force_dissipation: f32,
}

impl Pusher {
    pub fn new() -> Self {
        Pusher {
            strength: DEFAULT_PUSH_STRENGTH,
            push_force_dissipation: DEFAULT_PUSH_DISSIPATION,
        }
    }
}

pub struct AttachedTo(pub EntityId);

pub struct Attachment(pub EntityId);

pub struct AttachmentOffset(pub Vector2);

pub struct AttachmentOffsetByDirection {
    pub offsets: [Vector2; 4],
}

pub struct Attackable;

pub struct SpriteIndexByDirection {
    pub base_index: usize,
}

pub struct Weapon {
    pub atlas: Rc<BitmapAtlas>,
    pub base_index: usize,
    pub offsets: [Vector2; 4],
    pub damage: i32,
    pub radius_of_effect: u32,
}

pub struct Life(pub i32);

pub enum FlickerMethod {
    Color(u8),
    OnOff,
}

pub struct TimedFlicker {
    pub method: FlickerMethod,
    pub flick: bool,
    pub pre_timer: Option<f32>,
    pub timer: f32,
}

impl TimedFlicker {
    pub fn new(timer: f32, method: FlickerMethod) -> Self {
        TimedFlicker {
            timer,
            method,
            pre_timer: None,
            flick: true,
        }
    }

    pub fn new_with_pre_timer(timer: f32, pre_timer: f32, method: FlickerMethod) -> Self {
        TimedFlicker {
            timer,
            method,
            pre_timer: Some(pre_timer),
            flick: true,
        }
    }

    pub fn update(&mut self, delta: f32) {
        if let Some(mut pre_timer) = self.pre_timer {
            pre_timer -= delta;
            if pre_timer <= 0.0 {
                self.pre_timer = None;
            } else {
                self.pre_timer = Some(pre_timer);
            }
        } else {
            self.timer -= delta;
            self.flick = !self.flick;
        }
    }
}

pub struct HitParticleColor(pub u8);

pub struct Pickupable {
    pub kind: PickupType,
    pub pre_timer: f32,
}

pub struct Pickuper;

///////////////////////////////////////////////////////////////////////////////////////////////////

pub fn init_everything(context: &mut Game, map_file: &Path, min_spawn_time: f32, max_spawn_time: f32, max_slimes: usize) {
    init_entities(&mut context.core.entities);
    init_component_system(&mut context.component_systems);
    init_events(&mut context.event_listeners);
    context.core.event_publisher.clear();

    context.core.tilemap = TileMap::load_from(map_file).unwrap();
    new_camera_entity(&mut context.core, 0, 0);
    new_world_entity(&mut context.core, min_spawn_time, max_spawn_time, max_slimes);
}

pub fn init_entities(entities: &mut Entities) {
    entities.init_components::<Player>();
    entities.init_components::<Slime>();
    entities.init_components::<Activity>();
    entities.init_components::<AnimationDef>();
    entities.init_components::<AnimationInstance>();
    entities.init_components::<AnimateByActivity>();
    entities.init_components::<KillWhenAnimationFinishes>();
    entities.init_components::<Position>();
    entities.init_components::<Velocity>();
    entities.init_components::<Forces>();
    entities.init_components::<Bounds>();
    entities.init_components::<FacingDirection>();
    entities.init_components::<IgnoresCollision>();
    entities.init_components::<IgnoresFriction>();
    entities.init_components::<Particle>();
    entities.init_components::<LifeTime>();
    entities.init_components::<Pixel>();
    entities.init_components::<Sprite>();
    entities.init_components::<RandomlyWalksAround>();
    entities.init_components::<WalkingTime>();
    entities.init_components::<MovementSpeed>();
    entities.init_components::<World>();
    entities.init_components::<SpawnTimer>();
    entities.init_components::<Camera>();
    entities.init_components::<Pushable>();
    entities.init_components::<Pusher>();
    entities.init_components::<AttachedTo>();
    entities.init_components::<Attachment>();
    entities.init_components::<AttachmentOffset>();
    entities.init_components::<AttachmentOffsetByDirection>();
    entities.init_components::<Attackable>();
    entities.init_components::<SpriteIndexByDirection>();
    entities.init_components::<Weapon>();
    entities.init_components::<Life>();
    entities.init_components::<TimedFlicker>();
    entities.init_components::<HitParticleColor>();
    entities.init_components::<Pickupable>();
    entities.init_components::<Pickuper>();
    entities.remove_all_entities();
}

///////////////////////////////////////////////////////////////////////////////////////////////////

pub fn new_world_entity(context: &mut Core, min_spawn_time: f32, max_spawn_time: f32, max_slimes: usize) -> EntityId {
    let id = context.entities.new_entity();

    context.entities.add_component(id, World);

    let mut spawn_timer = SpawnTimer::new(min_spawn_time, max_spawn_time, max_slimes);
    spawn_timer.reset_timer();
    context.entities.add_component(id, spawn_timer);

    id
}

pub fn new_camera_entity(context: &mut Core, x: i32, y: i32) -> EntityId {
    let id = context.entities.new_entity();
    context.entities.add_component(id, Camera { x, y });
    id
}

pub fn new_slime_entity(context: &mut Core, x: i32, y: i32, direction: Direction, color: SlimeColor) -> EntityId {
    let id = context.entities.new_entity();

    let (atlas, chance_to_move, movement_speed, min_walk_time, max_walk_time, min_walk_cooldown, max_walk_cooldown, life, hit_color) = match color {
        SlimeColor::Green => (context.green_slime.clone(), 10, 8.0, 0.5, 2.0, 0.5, 5.0, 1, 11),
        SlimeColor::Blue => (context.blue_slime.clone(), 40, 12.0, 0.5, 2.0, 0.5, 3.0, 2, 13),
        SlimeColor::Orange => (context.orange_slime.clone(), 90, 24.0, 0.5, 1.0, 0.5, 2.0, 3, 9),
    };

    let activity = EntityActivity::Idle;
    let animate_by_activity = AnimateByActivity(context.slime_activity_states.clone());
    let animation = AnimationInstance::from(animate_by_activity.0.get(&activity).unwrap().clone());

    context.entities.add_component(id, Slime);
    context.entities.add_component(id, Position(Vector2::new(x as f32, y as f32)));
    context.entities.add_component(id, Velocity(Vector2::ZERO));
    context.entities.add_component(id, Forces::new());
    context.entities.add_component(id, Bounds { width: 16, height: 16, radius: 8 });
    context.entities.add_component(id, FacingDirection(direction));
    context.entities.add_component(id, Sprite { atlas, index: 0 });
    context.entities.add_component(id, Activity(activity));
    context.entities.add_component(id, animate_by_activity);
    context.entities.add_component(id, animation);
    context.entities.add_component(id, RandomlyWalksAround::new(min_walk_time, max_walk_time, chance_to_move, min_walk_cooldown, max_walk_cooldown));
    context.entities.add_component(id, MovementSpeed(movement_speed));
    context.entities.add_component(id, Pusher::new());
    context.entities.add_component(id, Pushable);
    context.entities.add_component(id, Attackable);
    context.entities.add_component(id, Life(life));
    context.entities.add_component(id, HitParticleColor(hit_color));

    id
}

pub fn spawn_slime_randomly(context: &mut Core) -> EntityId {
    let (x, y) = context.tilemap.get_random_spawnable_coordinates();
    let id = new_slime_entity(context, x * TILE_WIDTH as i32, y * TILE_HEIGHT as i32, Direction::new_random(), SlimeColor::new_random());
    spawn_poof_cloud(context, x * TILE_WIDTH as i32, y * TILE_HEIGHT as i32, 4, 8);
    id
}

pub fn new_player_entity(context: &mut Core, x: i32, y: i32, direction: Direction) -> EntityId {
    let id = context.entities.new_entity();

    let (atlas, weapon_offsets) = if rnd_value(0, 1) == 0 {
        (
            context.hero_female.clone(),
            [
                Vector2::new(-3.0, 13.0),
                Vector2::new(-14.0, 2.0),
                Vector2::new(14.0, 2.0),
                Vector2::new(3.0, -11.0)
            ]
        )
    } else {
        (
            context.hero_male.clone(),
            [
                Vector2::new(-3.0, 13.0),
                Vector2::new(-13.0, 2.0),
                Vector2::new(13.0, 2.0),
                Vector2::new(3.0, -11.0)
            ]
        )
    };

    let activity = EntityActivity::Idle;
    let animate_by_activity = AnimateByActivity(context.hero_activity_states.clone());
    let animation = AnimationInstance::from(animate_by_activity.0.get(&activity).unwrap().clone());

    let weapon = Weapon {
        atlas: context.sword.clone(),
        base_index: 0,
        offsets: weapon_offsets,
        damage: 1,
        radius_of_effect: 8,
    };

    context.entities.add_component(id, Player);
    context.entities.add_component(id, Position(Vector2::new(x as f32, y as f32)));
    context.entities.add_component(id, Velocity(Vector2::ZERO));
    context.entities.add_component(id, Forces::new());
    context.entities.add_component(id, Bounds { width: 14, height: 14, radius: 8 });
    context.entities.add_component(id, FacingDirection(direction));
    context.entities.add_component(id, Sprite { atlas, index: 0 });
    context.entities.add_component(id, Activity(activity));
    context.entities.add_component(id, animate_by_activity);
    context.entities.add_component(id, animation);
    context.entities.add_component(id, MovementSpeed(32.0));
    context.entities.add_component(id, Pusher::new());
    context.entities.add_component(id, Pushable);
    context.entities.add_component(id, weapon);
    context.entities.add_component(id, Pickuper);

    id
}

pub fn spawn_player_randomly(context: &mut Core) -> EntityId {
    let (x, y) = context.tilemap.get_random_spawnable_coordinates();
    new_player_entity(context, x * TILE_WIDTH as i32, y * TILE_HEIGHT as i32, Direction::South)
}

fn new_animation_effect(context: &mut Core, x: i32, y: i32, animation_def: Rc<AnimationDef>, delay_scaling_factor: Option<f32>) -> EntityId {
    let id = context.entities.new_entity();
    context.entities.add_component(id, Particle);
    context.entities.add_component(id, Position(Vector2::new(x as f32, y as f32)));
    context.entities.add_component(id, Sprite { atlas: context.particles.clone(), index: 0 });
    context.entities.add_component(id, KillWhenAnimationFinishes);
    context.entities.add_component(id, IgnoresCollision);
    context.entities.add_component(id, IgnoresFriction);

    let mut animation = AnimationInstance::from(animation_def);
    if let Some(delay_scaling_factor) = delay_scaling_factor {
        animation.delay_override = Some(animation.def.delay * delay_scaling_factor);
    }
    context.entities.add_component(id, animation);

    id
}

pub fn new_poof_animation(context: &mut Core, x: i32, y: i32, variant: usize, delay_scaling_factor: Option<f32>) -> EntityId {
    let def = match variant {
        0 => context.poof1_animation_def.clone(),
        1 => context.poof2_animation_def.clone(),
        _ => panic!("unknown poof animation variant")
    };
    new_animation_effect(context, x, y, def, delay_scaling_factor)
}

pub fn new_sparkles_animation(context: &mut Core, x: i32, y: i32, delay_scaling_factor: Option<f32>) -> EntityId {
    new_animation_effect(context, x, y, context.sparkles_animation_def.clone(), delay_scaling_factor)
}

pub fn new_pixel_particle(context: &mut Core, x: i32, y: i32, velocity: Vector2, lifetime: f32, color: u8) -> EntityId {
    let id = context.entities.new_entity();
    context.entities.add_component(id, Particle);
    context.entities.add_component(id, Position(Vector2::new(x as f32, y as f32)));
    context.entities.add_component(id, Velocity(velocity));
    context.entities.add_component(id, Pixel(color));
    context.entities.add_component(id, LifeTime(lifetime));
    context.entities.add_component(id, IgnoresCollision);
    context.entities.add_component(id, IgnoresFriction);
    id
}

pub fn spawn_pixel_cloud(context: &mut Core, x: i32, y: i32, count: usize, speed: f32, lifetime: f32, color: u8) {
    let mut angle = 0.0;
    for i in 0..count {
        angle += RADIANS_360 / count as f32;
        let velocity = Vector2::from_angle(angle) * speed;
        new_pixel_particle(context, x, y, velocity, lifetime, color);
    }
}

pub fn spawn_poof_cloud(context: &mut Core, x: i32, y: i32, count: usize, radius: i32) {
    for _ in 0..count {
        let x = x + rnd_value(-radius, radius);
        let y = y + rnd_value(-radius, radius);
        new_poof_animation(context, x, y, 0, match rnd_value(0, 5) {
            0 => Some(0.25),
            1 => Some(0.5),
            2 => Some(0.75),
            3 => Some(1.0),
            4 => Some(1.25),
            5 => Some(1.5),
            _ => None,
        });
    }
}

pub fn new_weapon_attachment_entity(context: &mut Core, attached_to: EntityId) -> Option<EntityId> {
    let sprite;
    let sprite_by_direction;
    let offset_by_direction;

    let weapons = context.entities.components::<Weapon>();
    if let Some(weapon) = weapons.get(&attached_to) {
        sprite = Sprite { atlas: weapon.atlas.clone(), index: 0 };
        sprite_by_direction = SpriteIndexByDirection { base_index: weapon.base_index };
        offset_by_direction = AttachmentOffsetByDirection { offsets: weapon.offsets };
    } else {
        // if the entity has no weapon "equipped" then they cannot attack!
        return None;
    }
    drop(weapons);

    let id = context.entities.new_entity();

    // note: no point in setting up initial position/direction, as attachment entities get these
    // properties automatically applied from their parent each frame

    context.entities.add_component(id, AttachedTo(attached_to));
    context.entities.add_component(id, Position(Vector2::ZERO));
    context.entities.add_component(id, FacingDirection(Direction::South));
    context.entities.add_component(id, sprite_by_direction);
    context.entities.add_component(id, sprite);
    context.entities.add_component(id, offset_by_direction);

    context.entities.add_component(attached_to, Attachment(id));

    Some(id)
}

pub fn new_pickupable_entity(context: &mut Core, x: i32, y: i32, force: Force, kind: PickupType) -> EntityId {
    let id = context.entities.new_entity();

    let mut forces = Forces::new();
    forces.forces.push(force);

    let sprite_index = match kind {
        PickupType::BlueGem => 0,
        PickupType::GreenGem => 1,
        PickupType::OrangeGem => 2,
        PickupType::Coin => 3,
    };

    context.entities.add_component(id, Pickupable { kind, pre_timer: PICKUP_PRE_TIMER });
    context.entities.add_component(id, Position(Vector2::new(x as f32, y as f32)));
    context.entities.add_component(id, Velocity(Vector2::ZERO));
    context.entities.add_component(id, forces);
    context.entities.add_component(id, Sprite { atlas: context.items.clone(), index: sprite_index });
    context.entities.add_component(id, Bounds { width: 16, height: 16, radius: 8 });
    context.entities.add_component(id, LifeTime(10.0));
    context.entities.add_component(id, TimedFlicker::new_with_pre_timer(10.0, 7.0, FlickerMethod::OnOff));

    id
}

pub fn spawn_pickups_from_entity(context: &mut Core, entity: EntityId) {
    let positions = context.entities.components::<Position>();
    let position = positions.get(&entity).unwrap().0;
    drop(positions);

    let num_pickups = rnd_value(0, 5);
    for _ in 0..num_pickups {
        let angle = (rnd_value(0, 359) as f32).to_radians();
        let force_strength = rnd_value(0.5, 5.0);
        let force = Force {
            force: Vector2::from_angle(angle) * force_strength,
            dissipation_factor: 0.5,
        };
        let kind = PickupType::new_random();
        new_pickupable_entity(context, position.x as i32, position.y as i32, force, kind);
    }
}
