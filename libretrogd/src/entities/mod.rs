use std::any::{TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::{HashMap, HashSet};

use crate::utils::AsAny;

pub type EntityId = usize;

///////////////////////////////////////////////////////////////////////////////////////////////////

// alias `Component` to always be `'static` ...
pub trait Component: 'static {}
impl<T: 'static> Component for T {}

pub type ComponentStore<T> = RefCell<HashMap<EntityId, T>>;
pub type RefComponents<'a, T> = Ref<'a, HashMap<EntityId, T>>;
pub type RefMutComponents<'a, T> = RefMut<'a, HashMap<EntityId, T>>;

pub trait GenericComponentStore: AsAny {
    fn has(&self, entity: EntityId) -> bool;
    fn remove(&mut self, entity: EntityId) -> bool;
    fn clear(&mut self);
}

impl<T: Component> GenericComponentStore for ComponentStore<T> {
    #[inline]
    fn has(&self, entity: EntityId) -> bool {
        self.borrow().contains_key(&entity)
    }

    #[inline]
    fn remove(&mut self, entity: EntityId) -> bool {
        self.get_mut().remove(&entity).is_some()
    }

    #[inline]
    fn clear(&mut self) {
        self.get_mut().clear();
    }
}

#[inline]
pub fn as_component_store<T: Component>(
    collection: &dyn GenericComponentStore,
) -> &ComponentStore<T> {
    collection.as_any().downcast_ref().unwrap()
}

#[inline]
pub fn as_component_store_mut<T: Component>(
    collection: &mut dyn GenericComponentStore,
) -> &mut ComponentStore<T> {
    collection.as_any_mut().downcast_mut().unwrap()
}

///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Entities {
    entities: HashSet<EntityId>,
    component_stores: HashMap<TypeId, Box<dyn GenericComponentStore>>,
    next_id: EntityId,
}

impl Entities {
    pub fn new() -> Self {
        Entities {
            entities: HashSet::new(),
            component_stores: HashMap::new(),
            next_id: 0,
        }
    }

    #[inline]
    fn has_component_store<T: Component>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.component_stores.contains_key(&type_id)
    }

    fn get_component_store<T: Component>(&self) -> Option<&ComponentStore<T>> {
        if !self.has_component_store::<T>() {
            None
        } else {
            let type_id = TypeId::of::<T>();
            Some(as_component_store(
                self.component_stores.get(&type_id).unwrap().as_ref(),
            ))
        }
    }

    fn add_component_store<T: Component>(&mut self) -> &ComponentStore<T> {
        if self.has_component_store::<T>() {
            self.get_component_store().unwrap()
        } else {
            let component_store = ComponentStore::<T>::new(HashMap::new());
            let type_id = TypeId::of::<T>();
            self.component_stores
                .insert(type_id, Box::new(component_store));
            as_component_store(self.component_stores.get(&type_id).unwrap().as_ref())
        }
    }

    #[inline]
    pub fn has_entity(&self, entity: EntityId) -> bool {
        self.entities.contains(&entity)
    }

    pub fn new_entity(&mut self) -> EntityId {
        let new_entity_id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.entities.insert(new_entity_id);
        new_entity_id
    }

    pub fn remove_entity(&mut self, entity: EntityId) -> bool {
        if !self.has_entity(entity) {
            return false;
        }

        self.entities.remove(&entity);
        for (_, component_store) in self.component_stores.iter_mut() {
            component_store.remove(entity);
        }
        true
    }

    pub fn remove_all_entities(&mut self) {
        self.entities.clear();
        for (_, component_store) in self.component_stores.iter_mut() {
            component_store.clear();
        }
    }

    pub fn has_component<T: Component>(&self, entity: EntityId) -> bool {
        if !self.has_entity(entity) {
            false
        } else {
            let type_id = TypeId::of::<T>();
            if let Some(component_store) = self.component_stores.get(&type_id) {
                component_store.has(entity)
            } else {
                false
            }
        }
    }

    pub fn add_component<T: Component>(&mut self, entity: EntityId, component: T) -> bool {
        if !self.has_entity(entity) {
            false
        } else {
            if let Some(component_store) = self.get_component_store::<T>() {
                component_store.borrow_mut().insert(entity, component);
            } else {
                self.add_component_store::<T>()
                    .borrow_mut()
                    .insert(entity, component);
            }
            true
        }
    }

    pub fn remove_component<T: Component>(&mut self, entity: EntityId) -> bool {
        if !self.has_entity(entity) {
            false
        } else {
            let type_id = TypeId::of::<T>();
            if let Some(component_store) = self.component_stores.get_mut(&type_id) {
                component_store.remove(entity)
            } else {
                false
            }
        }
    }

    #[inline]
    pub fn components<T: Component>(&self) -> Option<RefComponents<T>> {
        if let Some(component_store) = self.get_component_store() {
            Some(component_store.borrow())
        } else {
            None
        }
    }

    #[inline]
    pub fn components_mut<T: Component>(&self) -> Option<RefMutComponents<T>> {
        if let Some(component_store) = self.get_component_store() {
            Some(component_store.borrow_mut())
        } else {
            None
        }
    }

    pub fn init_components<T: Component>(&mut self) {
        if self.get_component_store::<T>().is_none() {
            self.add_component_store::<T>();
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

// TODO: is there some fancy way to get rid of the impl duplication here ... ?

pub trait ComponentStoreConvenience<T: Component> {
    fn single(&self) -> Option<(&EntityId, &T)>;
    fn get(&self, k: &EntityId) -> Option<&T>;
}

pub trait ComponentStoreConvenienceMut<T: Component> {
    fn single_mut(&mut self) -> Option<(&EntityId, &T)>;
    fn get_mut(&mut self, k: &EntityId) -> Option<&mut T>;
}

impl<'a, T: Component> ComponentStoreConvenience<T> for Option<RefComponents<'a, T>> {
    fn single(&self) -> Option<(&EntityId, &T)> {
        if let Some(components) = self {
            if let Some((entity_id, component)) = components.iter().next() {
                return Some((entity_id, component));
            }
        }
        None
    }

    fn get(&self, k: &EntityId) -> Option<&T> {
        if let Some(components) = self {
            components.get(k)
        } else {
            None
        }
    }
}

impl<'a, T: Component> ComponentStoreConvenience<T> for Option<RefMutComponents<'a, T>> {
    fn single(&self) -> Option<(&EntityId, &T)> {
        if let Some(components) = self {
            if let Some((entity_id, component)) = components.iter().next() {
                return Some((entity_id, component));
            }
        }
        None
    }

    fn get(&self, k: &EntityId) -> Option<&T> {
        if let Some(components) = self {
            components.get(k)
        } else {
            None
        }
    }
}

impl<'a, T: Component> ComponentStoreConvenienceMut<T> for Option<RefMutComponents<'a, T>> {
    fn single_mut(&mut self) -> Option<(&EntityId, &T)> {
        if let Some(components) = self {
            if let Some((entity_id, component)) = components.iter_mut().next() {
                return Some((entity_id, component));
            }
        }
        None
    }

    fn get_mut(&mut self, k: &EntityId) -> Option<&mut T> {
        if let Some(components) = self {
            components.get_mut(k)
        } else {
            None
        }
    }
}

pub trait OptionComponentStore<T: Component> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

impl<'a, T: Component> OptionComponentStore<T> for Option<RefComponents<'a, T>> {
    fn len(&self) -> usize {
        if let Some(components) = self {
            components.len()
        } else {
            0
        }
    }

    fn is_empty(&self) -> bool {
        if let Some(components) = self {
            components.is_empty()
        } else {
            true
        }
    }
}

impl<'a, T: Component> OptionComponentStore<T> for Option<RefMutComponents<'a, T>> {
    fn len(&self) -> usize {
        if let Some(components) = self {
            components.len()
        } else {
            0
        }
    }

    fn is_empty(&self) -> bool {
        if let Some(components) = self {
            components.is_empty()
        } else {
            true
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

pub type UpdateFn<T> = fn(&mut Entities, &mut T);
pub type RenderFn<T> = fn(&mut Entities, &mut T);

pub struct ComponentSystems<U, R> {
    update_systems: Vec<UpdateFn<U>>,
    render_systems: Vec<RenderFn<R>>,
}

impl<U, R> ComponentSystems<U, R> {
    pub fn new() -> Self {
        ComponentSystems {
            update_systems: Vec::new(),
            render_systems: Vec::new(),
        }
    }

    pub fn add_update_system(&mut self, f: UpdateFn<U>) {
        self.update_systems.push(f);
    }

    pub fn add_render_system(&mut self, f: RenderFn<R>) {
        self.render_systems.push(f);
    }

    pub fn reset(&mut self) {
        self.update_systems.clear();
        self.render_systems.clear();
    }

    pub fn update(&mut self, entities: &mut Entities, context: &mut U) {
        for f in self.update_systems.iter_mut() {
            f(entities, context);
        }
    }

    pub fn render(&mut self, entities: &mut Entities, context: &mut R) {
        for f in self.render_systems.iter_mut() {
            f(entities, context);
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use claim::*;

    use super::*;

    #[derive(Debug, Eq, PartialEq, Hash, Clone)]
    struct Name(&'static str);
    #[derive(Debug, Eq, PartialEq, Hash, Clone)]
    struct Position(i32, i32);
    #[derive(Debug, Eq, PartialEq, Hash, Clone)]
    struct Velocity(i32, i32);
    #[derive(Debug, Eq, PartialEq, Hash, Clone)]
    struct Health(u32);
    #[derive(Debug, Eq, PartialEq, Hash, Clone)]
    struct Counter(u32);

    #[test]
    fn add_and_remove_entities() {
        let mut em = Entities::new();

        // add first entity
        assert!(!em.has_entity(1));
        let a = em.new_entity();
        assert!(em.has_entity(a));

        // add second entity with totally different id from first entity
        let b = em.new_entity();
        assert!(em.has_entity(a));
        assert!(em.has_entity(b));
        assert_ne!(a, b);

        // remove first entity
        assert!(em.remove_entity(a));
        assert!(!em.has_entity(a));
        assert!(em.has_entity(b));

        // remove second entity
        assert!(em.remove_entity(b));
        assert!(!em.has_entity(b));

        // removing entities which don't exist shouldn't blow up
        assert!(!em.remove_entity(a));
        assert!(!em.remove_entity(b));

        // add third entity, will not re-use previous entity ids
        let c = em.new_entity();
        assert!(em.has_entity(c));
        assert_ne!(a, c);
        assert_ne!(b, c);
    }

    #[test]
    fn add_and_remove_entity_components() {
        let mut em = Entities::new();

        // create first entity
        let a = em.new_entity();
        assert!(em.has_entity(a));

        // add component
        assert!(!em.has_component::<Name>(a));
        assert!(em.add_component(a, Name("Someone")));
        assert!(em.has_component::<Name>(a));

        // verify the added component
        {
            let names = em.components::<Name>().unwrap();
            let name_a = names.get(&a).unwrap();
            assert_eq!("Someone", name_a.0);
        }

        // create second entity
        let b = em.new_entity();
        assert!(em.has_entity(b));

        // add component to second entity
        assert!(!em.has_component::<Position>(b));
        assert_none!(em.components::<Position>());
        assert!(em.add_component(b, Position(1, 2)));
        assert!(em.has_component::<Position>(b));

        // verify the added component
        {
            let positions = em.components::<Position>().unwrap();
            let position_b = positions.get(&b).unwrap();
            assert!(1 == position_b.0 && 2 == position_b.1);
        }

        // verify current components for both entities are what we expect
        assert!(em.has_component::<Name>(a));
        assert!(!em.has_component::<Name>(b));
        assert!(!em.has_component::<Position>(a));
        assert!(em.has_component::<Position>(b));

        // add new component to first entity
        assert!(em.add_component(a, Position(5, 3)));
        assert!(em.has_component::<Position>(a));

        // verify both position components for both entities
        {
            let positions = em.components::<Position>().unwrap();
            let position_a = positions.get(&a).unwrap();
            assert!(5 == position_a.0 && 3 == position_a.1);
            let position_b = positions.get(&b).unwrap();
            assert!(1 == position_b.0 && 2 == position_b.1);
        }

        // verify current components for both entities are what we expect
        assert!(em.has_component::<Name>(a));
        assert!(!em.has_component::<Name>(b));
        assert!(em.has_component::<Position>(a));
        assert!(em.has_component::<Position>(b));

        // remove position component from first entity
        assert!(em.remove_component::<Position>(a));
        assert!(!em.has_component::<Position>(a));

        // verify current components for both entities are what we expect
        assert!(em.has_component::<Name>(a));
        assert!(!em.has_component::<Name>(b));
        assert!(!em.has_component::<Position>(a));
        assert!(em.has_component::<Position>(b));
        {
            let positions = em.components::<Position>().unwrap();
            let position_b = positions.get(&b).unwrap();
            assert!(1 == position_b.0 && 2 == position_b.1);
        }
    }

    #[test]
    fn modify_components() {
        let mut em = Entities::new();

        // create entities
        let a = em.new_entity();
        em.add_component(a, Position(10, 20));
        let b = em.new_entity();
        em.add_component(b, Position(17, 5));

        // change entity positions
        {
            let mut positions = em.components_mut::<Position>().unwrap();
            let position_a = positions.get_mut(&a).unwrap();
            assert_eq!(Position(10, 20), *position_a);
            position_a.0 = 13;
            position_a.1 = 23;
        }

        {
            let mut positions = em.components_mut::<Position>().unwrap();
            let position_b = positions.get_mut(&b).unwrap();
            assert_eq!(Position(17, 5), *position_b);
            position_b.0 = 15;
            position_b.1 = 8;
        }

        // verify both entity position components
        {
            let positions = em.components::<Position>().unwrap();
            let position_a = positions.get(&a).unwrap();
            assert_eq!(Position(13, 23), *position_a);
            let position_b = positions.get(&b).unwrap();
            assert_eq!(Position(15, 8), *position_b);
        }
    }

    #[test]
    fn get_all_components_of_type() {
        let mut em = Entities::new();

        // create entities
        let a = em.new_entity();
        em.add_component(a, Health(20));
        em.add_component(a, Position(10, 20));
        let b = em.new_entity();
        em.add_component(b, Health(30));
        em.add_component(b, Position(17, 5));

        // verify initial entity positions
        {
            let positions = em.components::<Position>().unwrap();
            assert_eq!(2, positions.len());
            let positions: HashSet<&Position> = positions.values().collect();
            assert!(positions.contains(&Position(10, 20)));
            assert!(positions.contains(&Position(17, 5)));
        }

        // modify position components
        {
            let mut positions = em.components_mut::<Position>().unwrap();
            for mut component in positions.values_mut() {
                component.0 += 5;
            }

            assert_eq!(Position(15, 20), *positions.get(&a).unwrap());
            assert_eq!(Position(22, 5), *positions.get(&b).unwrap());
        }

        // modify health components
        {
            let mut healths = em.components_mut::<Health>().unwrap();
            for mut component in healths.values_mut() {
                component.0 += 5;
            }
            assert_eq!(Health(25), *healths.get(&a).unwrap());
            assert_eq!(Health(35), *healths.get(&b).unwrap());
        }
    }

    #[test]
    fn get_all_entities_with_component() {
        let mut em = Entities::new();

        // create entities
        let a = em.new_entity();
        em.add_component(a, Name("A"));
        em.add_component(a, Health(20));
        em.add_component(a, Position(10, 20));
        let b = em.new_entity();
        em.add_component(b, Name("B"));
        em.add_component(b, Position(17, 5));

        // get entities with position components
        {
            let positions = em.components::<Position>().unwrap();
            let entities = positions.keys();
            assert_eq!(2, entities.len());
            let entities: HashSet<&EntityId> = entities.collect();
            assert!(entities.contains(&a));
            assert!(entities.contains(&b));
        }

        //
        let names = em.components::<Name>().unwrap();
        for (entity, name) in names.iter() {
            // just written this way to verify can grab two mutable components at once
            // (since this wouldn't be an uncommon way to want to work with an entity)
            let mut healths = em.components_mut::<Health>().unwrap();
            let mut positions = em.components_mut::<Position>().unwrap();

            let health = healths.get_mut(&entity);
            let position = positions.get_mut(&entity);

            println!(
                "entity {}, health: {:?}, position: {:?}",
                name.0, health, position
            );

            if let Some(mut health) = health {
                health.0 += 5;
            }
            if let Some(mut position) = position {
                position.0 += 5;
            }
        }

        let positions = em.components::<Position>().unwrap();
        assert_eq!(Position(15, 20), *positions.get(&a).unwrap());
        assert_eq!(Position(22, 5), *positions.get(&b).unwrap());
        let healths = em.components::<Health>().unwrap();
        assert_eq!(Health(25), *healths.get(&a).unwrap());
        assert!(healths.get(&b).is_none());
    }

    struct UpdateContext(f32);
    struct RenderContext(f32);

    fn system_print_entity_positions(entities: &mut Entities, _context: &mut RenderContext) {
        let positions = entities.components::<Position>().unwrap();
        for (entity, position) in positions.iter() {
            println!("entity {} at x:{}, y:{}", entity, position.0, position.1)
        }
    }

    fn system_move_entities_forward(entities: &mut Entities, _context: &mut UpdateContext) {
        let mut positions = entities.components_mut::<Position>().unwrap();
        let velocities = entities.components::<Velocity>().unwrap();
        for (entity, position) in positions.iter_mut() {
            if let Some(velocity) = velocities.get(&entity) {
                position.0 += velocity.0;
                position.1 += velocity.1;
            }
        }
    }

    fn system_increment_counter(entities: &mut Entities, _context: &mut UpdateContext) {
        let mut counters = entities.components_mut::<Counter>().unwrap();
        for (_entity, counter) in counters.iter_mut() {
            counter.0 += 1;
        }
    }

    fn system_print_counter(entities: &mut Entities, _context: &mut RenderContext) {
        let counters = entities.components::<Counter>().unwrap();
        for (entity, counter) in counters.iter() {
            println!("entity {} has counter {}", entity, counter.0);
        }
    }

    #[test]
    pub fn component_systems() {
        let mut em = Entities::new();

        // create entities
        let a = em.new_entity();
        em.add_component(a, Position(5, 6));
        em.add_component(a, Velocity(1, 1));
        em.add_component(a, Counter(0));
        let b = em.new_entity();
        em.add_component(b, Position(-3, 0));
        em.add_component(b, Velocity(1, 0));
        em.add_component(b, Counter(0));
        let c = em.new_entity();
        em.add_component(c, Position(2, 9));
        em.add_component(c, Counter(0));

        // setup component systems
        let mut cs = ComponentSystems::new();
        cs.add_update_system(system_move_entities_forward);
        cs.add_update_system(system_increment_counter);
        cs.add_render_system(system_print_entity_positions);
        cs.add_render_system(system_print_counter);

        // run some update+render iterations
        for _ in 0..5 {
            let mut update_context = UpdateContext(0.0);
            let mut render_context = RenderContext(0.0);
            cs.update(&mut em, &mut update_context);
            cs.render(&mut em, &mut render_context);
        }

        // verify expected entity positions
        let positions = em.components::<Position>().unwrap();
        let velocities = em.components::<Velocity>().unwrap();
        let counters = em.components::<Counter>().unwrap();
        assert_eq!(Position(10, 11), *positions.get(&a).unwrap());
        assert_eq!(Velocity(1, 1), *velocities.get(&a).unwrap());
        assert_eq!(Counter(5), *counters.get(&a).unwrap());
        assert_eq!(Position(2, 0), *positions.get(&b).unwrap());
        assert_eq!(Velocity(1, 0), *velocities.get(&b).unwrap());
        assert_eq!(Counter(5), *counters.get(&b).unwrap());
        assert_eq!(Position(2, 9), *positions.get(&c).unwrap());
        assert_eq!(None, velocities.get(&c));
        assert_eq!(Counter(5), *counters.get(&c).unwrap());
    }
}
