use libretrogd::entities::*;
use libretrogd::events::*;

use crate::Core;
use crate::entities::*;

#[derive(Debug, Copy, Clone)]
pub enum Event {
    TurnAndMove(EntityId, Direction),
    MoveForward(EntityId),
    Remove(EntityId),
    RemoveAttachment(EntityId),
    AnimationFinished(EntityId),
    Spawn(EntityId),
    SpawnSlimeRandomly,
    SetActivity(EntityId, EntityActivity),
    Attack(EntityId),
    Hit(EntityId, EntityId, i32, Vector2),
    Kill(EntityId),
    Pickup(EntityId, EntityId),
}

fn event_handler(event: &Event, context: &mut Core) -> bool {
    match event {
        Event::Remove(entity) => {
            if context.entities.has_entity(*entity) {
                remove_entity(&mut context.entities, *entity);
            }
        },
        Event::RemoveAttachment(entity) => {
            if context.entities.has_entity(*entity) {
                remove_entity_attachment(&mut context.entities, *entity);
            }
        }
        Event::TurnAndMove(entity, direction) => {
            if context.entities.has_entity(*entity) {
                turn_and_move_entity(context, *entity, *direction);
            }
        },
        Event::MoveForward(entity) => {
            if context.entities.has_entity(*entity) {
                move_entity_forward(context, *entity);
            }
        },
        Event::Spawn(entity) => {
            // todo
        },
        Event::AnimationFinished(entity) => {
            if context.entities.has_entity(*entity) {
                // if the entity's 'attack' animation just finished, move them back to 'idle'
                let activities = context.entities.components::<Activity>();
                if let Some(activity) = activities.get(entity) {
                    if activity.0 == EntityActivity::Attacking {
                        drop(activities);
                        stop_attack(context, *entity);
                    }
                }
            }
        }
        Event::SpawnSlimeRandomly => {
            spawn_slime_randomly(context);
        },
        Event::SetActivity(entity, activity) => {
            if context.entities.has_entity(*entity) {
                set_entity_activity(&mut context.entities, *entity, *activity);
            }
        },
        Event::Attack(entity) => {
            if context.entities.has_entity(*entity) {
                attack(context, *entity);
            }
        },
        Event::Hit(target, source, damage, damage_position) => {
            if context.entities.has_entity(*target) {
                hit_entity(context, *target, *source, *damage, *damage_position);
            }
        },
        Event::Kill(entity) => {
            kill_entity(context, *entity);
        },
        Event::Pickup(picked_up_by, picked_up) => {
            if context.entities.has_entity(*picked_up_by) && context.entities.has_entity(*picked_up) {
                pickup(context, *picked_up_by, *picked_up);
            }
        }
    }
    false
}

pub fn init_events(event_listener: &mut EventListeners<Event, Core>) {
    event_listener.clear();
    event_listener.add(event_handler);
}