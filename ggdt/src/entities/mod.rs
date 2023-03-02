use std::any::TypeId;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::{HashMap, HashSet};
use std::fmt::Formatter;

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
	/// Returns true if this component store currently has a component for the specified entity.
	fn has(&self, entity: EntityId) -> bool;

	/// If this component store has a component for the specified entity, removes it and returns
	/// true. Otherwise, returns false.
	fn remove(&mut self, entity: EntityId) -> bool;

	/// Removes all components from this store.
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

/// Entity manager. Stores entity components and manages entity IDs.
pub struct Entities {
	entities: HashSet<EntityId>,
	component_stores: HashMap<TypeId, Box<dyn GenericComponentStore>>,
	next_id: EntityId,
}

impl std::fmt::Debug for Entities {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Entities")
			.field("entities.len()", &self.entities.len())
			.field("component_stores.keys()", &self.component_stores.keys())
			.field("next_id", &self.next_id)
			.finish_non_exhaustive()
	}
}

impl Entities {
	/// Creates and returns a new instance of an entity manager.
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

	/// Returns true if the entity manager currently is aware of an entity with the given ID.
	#[inline]
	pub fn has_entity(&self, entity: EntityId) -> bool {
		self.entities.contains(&entity)
	}

	/// Returns a previously unused entity ID. Use this to "create" a new entity.
	pub fn new_entity(&mut self) -> EntityId {
		let new_entity_id = self.next_id;
		self.next_id = self.next_id.wrapping_add(1);
		self.entities.insert(new_entity_id);
		new_entity_id
	}

	/// Removes an entity, making the entity ID unusable with this entity manager as well as
	/// removing all of the entity's components. Returns true if the entity was removed, false if
	/// the entity ID given did not exist.
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

	/// Removes all entities from the entity manager, as well as all of their components.
	pub fn remove_all_entities(&mut self) {
		self.entities.clear();
		for (_, component_store) in self.component_stores.iter_mut() {
			component_store.clear();
		}
	}

	/// Returns true if the given entity currently has a component of the specified type.
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

	/// Adds the given component to the entity manager, associating it with the given entity ID.
	/// If this entity already had a component of the same type, that existing component is replaced
	/// with this one. Returns true if the component was set to the entity.
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

	/// Removes any component of the given type from the specified entity. If the entity had a
	/// component of that type and it was removed, returns true.
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

	/// Returns a reference to the component store for the given component type. This allows you
	/// to get components of the specified type for any number of entities. If there is currently
	/// no component store for this type of component, `None` is returned.
	#[inline]
	pub fn components<T: Component>(&self) -> Option<RefComponents<T>> {
		if let Some(component_store) = self.get_component_store() {
			Some(component_store.borrow())
		} else {
			None
		}
	}

	/// Returns a reference to the mutable component store for the given component type. This allows
	/// you to get and modify components of the specified type for any number of entities. IF there
	/// is currently no component store for this type of component, `None` is returned.
	///
	/// Note that while technically you can add/remove components using the returned store, you
	/// should instead prefer to use [`Entities::add_component`] and [`Entities::remove_component`]
	/// instead.
	#[inline]
	pub fn components_mut<T: Component>(&self) -> Option<RefMutComponents<T>> {
		if let Some(component_store) = self.get_component_store() {
			Some(component_store.borrow_mut())
		} else {
			None
		}
	}

	/// Initializes a component store for the given component type if one does not exist already.
	///
	/// This is technically never needed to be called explicitly (because
	/// [`Entities::add_component`] will initialize a missing component store automatically if
	/// needed), but is provided as a convenience so that you could, for example, always
	/// pre-initialize all of your component stores so that subsequent calls to
	/// [`Entities::components`] and [`Entities::components_mut`] are guaranteed to never return
	/// `None`.
	pub fn init_components<T: Component>(&mut self) {
		if self.get_component_store::<T>().is_none() {
			self.add_component_store::<T>();
		}
	}
}

///////////////////////////////////////////////////////////////////////////////////////////////////

// TODO: is there some fancy way to get rid of the impl duplication here ... ?

/// Convenience methods that slightly help the ergonomics of using component stores returned from
/// [`Entities::components`].
pub trait ComponentStoreConvenience<T: Component> {
	/// Returns the "first" component from the component store along with the entity ID the
	/// component is for. This method should only ever be used with components that you know will
	/// only ever be attached to one entity (and therefore, the component store for this type of
	/// component only has a single entry in it). Otherwise, which component it returns is
	/// undefined.
	fn single(&self) -> Option<(&EntityId, &T)>;

	/// Returns the component for the given entity, if one exists, otherwise returns `None`.
	fn get(&self, k: &EntityId) -> Option<&T>;

	/// Returns true if there is a component for the given entity in this store.
	fn contains_key(&self, k: &EntityId) -> bool;
}

pub trait ComponentStoreConvenienceMut<T: Component> {
	/// Returns the "first" component from the component store along with the entity ID the
	/// component is for as a mutable reference. This method should only ever be used with
	/// components that you know will only ever be attached to one entity (and therefore, the
	/// component store for this type of component only has a single entry in it). Otherwise, which
	/// component it returns is undefined.
	fn single_mut(&mut self) -> Option<(&EntityId, &mut T)>;

	/// Returns the component for the given entity as a mutable reference if one exists, otherwise
	/// returns `None`.
	fn get_mut(&mut self, k: &EntityId) -> Option<&mut T>;

	/// Returns true if there is a component for the given entity in this store.
	fn contains_key(&mut self, k: &EntityId) -> bool;
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

	fn contains_key(&self, k: &EntityId) -> bool {
		if let Some(components) = self {
			components.contains_key(k)
		} else {
			false
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

	fn contains_key(&self, k: &EntityId) -> bool {
		if let Some(components) = self {
			components.contains_key(k)
		} else {
			false
		}
	}
}

impl<'a, T: Component> ComponentStoreConvenienceMut<T> for Option<RefMutComponents<'a, T>> {
	fn single_mut(&mut self) -> Option<(&EntityId, &mut T)> {
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

	fn contains_key(&mut self, k: &EntityId) -> bool {
		if let Some(components) = self {
			components.contains_key(k)
		} else {
			false
		}
	}
}

/// Convenience methods that slightly help the ergonomics of using component stores returned from
/// [`Entities::components`].
pub trait OptionComponentStore<T: Component> {
	/// Returns the total number of components in this component store. This is the same as the
	/// number of entities that have a component of this type.
	fn len(&self) -> usize;

	/// Returns true if this store is empty.
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

/// An "update component system" function that is used to update entity component state for a
/// certain type/set of components. The generic type used is some application-specific context
/// type that should be passed to all of your "update component system" functions.
pub type UpdateFn<T> = fn(&mut T);

/// A "render component system" function that is used to execute render logic for entities based
/// on a certain type/set of components. The generic type used is some application-specific context
/// type that should be passed to all of your "render component system" functions.
pub type RenderFn<T> = fn(&mut T);

/// This is a totally optional minor convenience to help you to manage your "component systems"
/// and ensure they are always called in the same order.
///
/// The generic types `U` and `R` refer to application-specific context types that are needed by
/// all of your "update" and "render" component system functions. Both of these types may be the
/// same type or different depending on your needs.
pub struct ComponentSystems<U, R> {
	update_systems: Vec<UpdateFn<U>>,
	render_systems: Vec<RenderFn<R>>,
}

impl<U, R> std::fmt::Debug for ComponentSystems<U, R> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ComponentSystems")
			.field("update_systems.len()", &self.update_systems.len())
			.field("render_systems.len()", &self.render_systems.len())
			.finish_non_exhaustive()
	}
}

impl<U, R> ComponentSystems<U, R> {
	pub fn new() -> Self {
		ComponentSystems {
			update_systems: Vec::new(),
			render_systems: Vec::new(),
		}
	}

	/// Adds an update component system function to the list of functions that will be called in
	/// order whenever [`ComponentSystems::update`] is called.
	pub fn add_update_system(&mut self, f: UpdateFn<U>) {
		self.update_systems.push(f);
	}

	/// Adds a render component system function to the list of functions that will be called in
	/// order whenever [`ComponentSystems::render`] is called.
	pub fn add_render_system(&mut self, f: RenderFn<R>) {
		self.render_systems.push(f);
	}

	/// Removes all existing update and render component system functions.
	pub fn reset(&mut self) {
		self.update_systems.clear();
		self.render_systems.clear();
	}

	/// Calls each of the update component system functions in the same order that they were added,
	/// passing each of them the context argument provided.
	pub fn update(&mut self, context: &mut U) {
		for f in self.update_systems.iter_mut() {
			f(context);
		}
	}

	/// Calls each of the render component system functions in the same order that they were added,
	/// passing each of them the context argument provided.
	pub fn render(&mut self, context: &mut R) {
		for f in self.render_systems.iter_mut() {
			f(context);
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

	struct ComponentSystemContext {
		pub delta: f32,
		pub entities: Entities,
	}

	impl ComponentSystemContext {
		pub fn new(entities: Entities) -> Self {
			ComponentSystemContext {
				delta: 0.0,
				entities,
			}
		}
	}

	fn system_print_entity_positions(context: &mut ComponentSystemContext) {
		let positions = context.entities.components::<Position>().unwrap();
		for (entity, position) in positions.iter() {
			println!("entity {} at x:{}, y:{}", entity, position.0, position.1)
		}
	}

	fn system_move_entities_forward(context: &mut ComponentSystemContext) {
		let mut positions = context.entities.components_mut::<Position>().unwrap();
		let velocities = context.entities.components::<Velocity>().unwrap();
		for (entity, position) in positions.iter_mut() {
			if let Some(velocity) = velocities.get(&entity) {
				position.0 += velocity.0;
				position.1 += velocity.1;
			}
		}
	}

	fn system_increment_counter(context: &mut ComponentSystemContext) {
		let mut counters = context.entities.components_mut::<Counter>().unwrap();
		for (_entity, counter) in counters.iter_mut() {
			counter.0 += 1;
		}
	}

	fn system_print_counter(context: &mut ComponentSystemContext) {
		let counters = context.entities.components::<Counter>().unwrap();
		for (entity, counter) in counters.iter() {
			println!("entity {} has counter {}", entity, counter.0);
		}
	}

	#[test]
	pub fn component_systems() {
		let mut context = ComponentSystemContext::new(Entities::new());

		// create entities
		let a = context.entities.new_entity();
		context.entities.add_component(a, Position(5, 6));
		context.entities.add_component(a, Velocity(1, 1));
		context.entities.add_component(a, Counter(0));
		let b = context.entities.new_entity();
		context.entities.add_component(b, Position(-3, 0));
		context.entities.add_component(b, Velocity(1, 0));
		context.entities.add_component(b, Counter(0));
		let c = context.entities.new_entity();
		context.entities.add_component(c, Position(2, 9));
		context.entities.add_component(c, Counter(0));

		// setup component systems
		let mut cs = ComponentSystems::new();
		cs.add_update_system(system_move_entities_forward);
		cs.add_update_system(system_increment_counter);
		cs.add_render_system(system_print_entity_positions);
		cs.add_render_system(system_print_counter);

		// run some update+render iterations
		for _ in 0..5 {
			context.delta = 0.0;
			cs.update(&mut context);
			cs.render(&mut context);
		}

		// verify expected entity positions
		let positions = context.entities.components::<Position>().unwrap();
		let velocities = context.entities.components::<Velocity>().unwrap();
		let counters = context.entities.components::<Counter>().unwrap();
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
