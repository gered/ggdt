extern crate core;

use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use anyhow::Result;

use libretrogd::entities::*;
use libretrogd::events::*;
use libretrogd::graphics::*;
use libretrogd::math::*;
use libretrogd::states::*;
use libretrogd::system::*;

use crate::entities::*;
use crate::states::*;
use crate::support::*;
use crate::tilemap::*;

mod states;
mod entities;
mod support;
mod tilemap;

pub const TILE_WIDTH: u32 = 16;
pub const TILE_HEIGHT: u32 = 16;

pub struct Core {
    pub delta: f32,
    pub system: System,
    pub font: BitmaskFont,
    pub entities: Entities,
    pub event_publisher: EventPublisher<Event>,
    pub palette: Palette,
    pub fade_out_palette: Palette,
    pub tiles: Rc<BitmapAtlas>,
    pub hero_male: Rc<BitmapAtlas>,
    pub hero_female: Rc<BitmapAtlas>,
    pub green_slime: Rc<BitmapAtlas>,
    pub blue_slime: Rc<BitmapAtlas>,
    pub orange_slime: Rc<BitmapAtlas>,
    pub fist: Rc<BitmapAtlas>,
    pub sword: Rc<BitmapAtlas>,
    pub particles: Rc<BitmapAtlas>,
    pub items: Rc<BitmapAtlas>,
    pub ui: Rc<BitmapAtlas>,
    pub tilemap: TileMap,
    pub slime_activity_states: Rc<HashMap<EntityActivity, Rc<AnimationDef>>>,
    pub hero_activity_states: Rc<HashMap<EntityActivity, Rc<AnimationDef>>>,
    pub poof1_animation_def: Rc<AnimationDef>,
    pub poof2_animation_def: Rc<AnimationDef>,
    pub sparkles_animation_def: Rc<AnimationDef>,
    pub sprite_render_list: Vec<(EntityId, Vector2, BlitMethod)>,
}

pub struct Support {
    pub component_systems: ComponentSystems<Core, Core>,
    pub event_listeners: EventListeners<Event, Core>,
}

pub struct Game {
    pub core: Core,
    pub support: Support,
}

impl Game {
    pub fn new(mut system: System) -> Result<Self> {
        let palette = load_palette(Path::new("./assets/db16.pal"))?;
        system.palette = palette.clone();

        let font = load_font(Path::new("./assets/dp.fnt"))?;

        let tiles = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/tiles.pcx"))?);
        let hero_male = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/hero_male.pcx"))?);
        let hero_female = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/hero_female.pcx"))?);
        let green_slime = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/green_slime.pcx"))?);
        let blue_slime = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/blue_slime.pcx"))?);
        let orange_slime = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/orange_slime.pcx"))?);
        let fist = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/fist.pcx"))?);
        let sword = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/sword.pcx"))?);
        let particles = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/particles.pcx"))?);
        let items = Rc::new(load_bitmap_atlas_autogrid(Path::new("./assets/items.pcx"))?);

        let mut ui = load_bitmap_atlas(Path::new("./assets/ui.pcx"))?;
        ui.add(Rect::new(0, 0, 16, 16))?;
        ui.add(Rect::new(16, 0, 16, 16))?;
        for i in 0..8 {
            ui.add(Rect::new(i * 8, 16, 8, 8))?;
        }

        let tilemap = TileMap::load_from(Path::new("./assets/title_screen.map.json"))?;

        let entities = Entities::new();
        let component_systems = ComponentSystems::new();
        let event_publisher = EventPublisher::new();
        let event_listeners = EventListeners::new();

        let slime_activity_states = HashMap::from([
            (EntityActivity::Idle, Rc::new(AnimationDef::new(&[1, 2], true, 1.0, Some(3)))),
            (EntityActivity::Walking, Rc::new(AnimationDef::new(&[1, 0, 2, 0], true, 0.25, Some(3)))),
            (EntityActivity::Attacking, Rc::new(AnimationDef::new(&[0], false, 0.3, Some(3)))),
            (EntityActivity::Dead, Rc::new(AnimationDef::new(&[12], false, 1.0, None))),
        ]);
        let hero_activity_states = HashMap::from([
            (EntityActivity::Idle, Rc::new(AnimationDef::new(&[0], true, 0.5, Some(4)))),
            (EntityActivity::Walking, Rc::new(AnimationDef::new(&[0, 1, 0, 2], true, 0.15, Some(4)))),
            (EntityActivity::Attacking, Rc::new(AnimationDef::new(&[3], false, 0.3, Some(4)))),
            (EntityActivity::Dead, Rc::new(AnimationDef::new(&[16], false, 1.0, None))),
        ]);
        let poof1_animation_def = Rc::new(AnimationDef::new(&[0, 1, 2], false, 0.15, None));
        let poof2_animation_def = Rc::new(AnimationDef::new(&[3, 4, 5], false, 0.15, None));
        let sparkles_animation_def = Rc::new(AnimationDef::new(&[6, 7, 8, 9], false, 0.1, None));

        Ok(Game {
            core: Core {
                delta: 0.0,
                system,
                font,
                entities,
                event_publisher,
                palette,
                fade_out_palette: Palette::new_with_default(20, 12, 28),
                tiles,
                hero_male,
                hero_female,
                green_slime,
                blue_slime,
                orange_slime,
                fist,
                sword,
                particles,
                items,
                ui: Rc::new(ui),
                tilemap,
                slime_activity_states: Rc::new(slime_activity_states),
                hero_activity_states: Rc::new(hero_activity_states),
                poof1_animation_def,
                poof2_animation_def,
                sparkles_animation_def,
                sprite_render_list: Vec::with_capacity(1024),
            },
            support: Support {
                component_systems,
                event_listeners,
            },
        })
    }

    pub fn do_events(&mut self) {
        self.support.event_listeners.take_queue_from(&mut self.core.event_publisher);
        self.support.event_listeners.dispatch_queue(&mut self.core);
    }

    pub fn update_frame_delta(&mut self, last_ticks: u64) -> u64 {
        let ticks = self.core.system.ticks();
        let elapsed = ticks - last_ticks;
        self.core.delta = (elapsed as f64 / self.core.system.tick_frequency() as f64) as f32;
        ticks
    }
}

fn main() -> Result<()> {
    let system = SystemBuilder::new().window_title("Slime Stabbing Simulator").vsync(true).build()?;
    let mut game = Game::new(system)?;
    let mut states = States::new();
    states.push(MainMenuState::new())?;

    let mut is_running = true;
    let mut last_ticks = game.core.system.ticks();

    while is_running && !states.is_empty() {
        game.core.system.do_events_with(|event| {
            match event {
                SystemEvent::Quit => {
                    is_running = false;
                },
                _ => {}
            }
        });

        last_ticks = game.update_frame_delta(last_ticks);
        states.update(&mut game)?;
        states.render(&mut game);
        game.core.system.display()?;
    }

    Ok(())
}
