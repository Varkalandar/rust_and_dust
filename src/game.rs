use vecmath::{Vector2, vec2_sub, vec2_add, vec2_scale, vec2_normalized};

use glium::Texture2d;
use glium::winit::keyboard::Key;
use glium::Frame;

use rand::Rng;

use crate::ui::{UI, UiController, MouseButton, Button, ButtonState, ButtonEvent, MouseMoveEvent, ScrollEvent};
use crate::GameWorld;
use crate::move_player;
use crate::screen_to_world_pos;
use crate::views::player_items_view::PlayerItemsView;
use crate::views::shop_view::ShopView;
use crate::TileSet;
use crate::Map;
use crate::map::MoveEndAction;
use crate::map::MapObject;
use crate::map::MapObjectFactory;
use crate::map::MobType;
use crate::map::TransitionDestination;
use crate::MAP_OBJECT_LAYER;
use crate::PROJECTILE_TILESET;
use crate::SoundPlayer;
use crate::sound::Sound;
use crate::generate_dungeon;
use crate::map_pos;


const MAGIC_ITEM_CHANCE: f32 = 0.4;
const MAGIC_FIND_FACTOR: f32 = 0.8;


pub struct Game 
{
    piv: PlayerItemsView,
    shop_view: ShopView,
    item_tiles: TileSet,

    show_player_inventory: bool,
    show_shop_inventory: bool,
}


impl UiController for Game 
{
    type Appdata = GameWorld;

    /**
     * @return true if this controller could handle the event, false to pass the event to other controllers
     */
     fn handle_button_event(&mut self, ui: &mut UI, event: &ButtonEvent, world: &mut Self::Appdata) -> bool {
        // first pass the even to the UI. if there is a component
        // trigered it will consume the event. Events which are not
        // consumed by the UI will be handed to the game core

        let comp = ui.handle_button_event(&event);

        if event.args.state == ButtonState::Release {

            if event.args.button == Button::Keyboard(Key::Character("i".into())) {
                self.show_player_inventory = !self.show_player_inventory;
            }        


            match comp {
                None => {
                    // the click hit no UI element, so we look into handling it.

                    let mut consumed = false;

                    // first check our self-controlled user interfaces
                    if self.show_shop_inventory {
                        consumed = self.shop_view.handle_button_event(event, &ui.context.mouse_state, world);
                    }
        
                    if !consumed && self.show_player_inventory {
                        consumed = self.piv.handle_button_event(event, &ui.context.mouse_state, world);
                    }

                    if consumed {return true;}

                    // then check the game itself

                    let pos = screen_to_world_pos(&ui, &world.map.get_player_position(), &ui.context.mouse_state.position);
                    
                    if event.args.button == Button::Mouse(MouseButton::Left) {
                        ui.root.head.clear();

                        let target_opt = Map::find_nearest_object(&world.map.layers[MAP_OBJECT_LAYER], &pos, 100.0, world.map.player_id);
                        match target_opt {
                            None => {},
                            Some(target_uid) => {
                                let target = world.map.layers[MAP_OBJECT_LAYER].get(&target_uid).unwrap();

                                if target.creature.is_some() {
                                    let target_pos = target.position;
                                    fire_player_projectile(&mut world.map, target_pos, &mut world.speaker);
                                    return true;
                                }
                            }
                        }

                        {
                            // close shop and inventory views if the player moves from shop
                            if self.show_shop_inventory {
                                self.show_shop_inventory = false;
                                self.show_player_inventory = false;
                            }

                            // now move
                            let screen_direction = vec2_sub(ui.context.mouse_state.position, ui.window_center());
                            move_player(&mut world.map, screen_direction);                
                        }
                    }

                    if event.args.button == Button::Mouse(MouseButton::Right) {
                        fire_player_projectile(&mut world.map, pos, &mut world.speaker);
                    }
                },
                Some(_comp) => {
                    // the click hit some UI element, so we ignore it.
                }
            }
        }

        false
    }


    fn handle_mouse_move_event(&mut self, ui: &mut UI, event: &MouseMoveEvent, world: &mut Self::Appdata) -> bool 
    {
        let _comp = ui.handle_mouse_move_event(event);

        if self.show_player_inventory {
            self.piv.handle_mouse_move_event(event, &ui.context.mouse_state, &mut world.player_inventory);
        }

        if self.show_shop_inventory {
            self.shop_view.handle_mouse_move_event(event, &ui.context.mouse_state, &mut world.player_inventory);
        }

        false
    }


    /**
     * @return true if this controller could handle the event, false to pass the event to other controllers
     */
    fn handle_scroll_event(&mut self, ui: &mut UI, event: &ScrollEvent, _world: &mut Self::Appdata) -> bool 
    {
        let _comp = ui.handle_scroll_event(event);

        false
    }


    fn draw(&mut self, target: &mut Frame, ui: &mut UI, world: &mut Self::Appdata) 
    {
        ui.draw(target);
 
        if self.show_player_inventory {
            self.piv.draw(ui, target, 0, 10, &world.player_inventory, &self.item_tiles);
        }

        if self.show_shop_inventory {
            let shop = &world.map.shops[self.shop_view.get_shop_index()];
            self.shop_view.draw(ui, target, shop, &world.player_inventory, &self.item_tiles);
        }
    }


    fn draw_overlay(&mut self, target: &mut Frame, ui: &mut UI, _world: &mut Self::Appdata) 
    {
        ui.context.font_14.draw(&ui.display, target, &ui.program, 10, 20, "Game testing mode", &[1.0, 1.0, 1.0, 1.0]);
    }


    fn update(&mut self, world: &mut Self::Appdata, dt: f32) -> bool 
    {
        let map = &mut world.map;
        let inv = &mut world.player_inventory;
        let rng = &mut world.rng;
        let speaker = &mut world.speaker;
        
        let (killed_mob_list, transition_opt) = map.update(dt, inv, rng, speaker);

        drop_loot(map, killed_mob_list, rng, speaker);

        if transition_opt.is_some() {
            let index = transition_opt.unwrap();
            let transition = &world.map.transitions[index];
            
            match transition.destination {
                TransitionDestination::Map { to_map, to_location } => {

                    if to_map == 501 {
                        // preserve player
                        let mut player = world.map.layers[MAP_OBJECT_LAYER].remove(&world.map.player_id).unwrap();
        
                        world.map.clear();
                        let dungeon = generate_dungeon(&mut world.map);
                
                        // stop player movement
                        player.move_time_left = 0.0;
                        world.map.layers[MAP_OBJECT_LAYER].insert(world.map.player_id, player);
                
                        world.map.set_player_position(dungeon.start_position);
        
                        let x = (dungeon.rooms[5].x1 + dungeon.rooms[5].x2) / 2;
                        let y = (dungeon.rooms[5].y1 + dungeon.rooms[5].y2) / 2;
        
                        world.map.populate("dungeon.csv", rng, map_pos(x, y, 0));
                    }
                    else {
                        world.map.load("town.map");
                        // self.populate("town.csv", rng);
            
                        world.map.set_player_position(to_location);
                    }

                    return true;        
                },
                TransitionDestination::Shop { index } => {
                    if !self.show_shop_inventory {
                        speaker.play(Sound::Click, 0.5);
                        self.show_shop_inventory = true;
                        self.show_player_inventory = false;
                        self.shop_view.set_shop_index(index);
                        return true;
                    }
                }
            }
        }

        return false;
    }
}


impl Game {

    pub fn new(inventory_bg: Texture2d, shop_bg: Texture2d, ui: &UI, item_tiles: &TileSet) -> Game 
    {
        let piv = PlayerItemsView::new((ui.context.window_size[0] as i32) / 2, 0, inventory_bg,);
    
        let shop_view = ShopView::new(shop_bg);
        
        Game 
        {
            piv,
            shop_view,
            show_player_inventory: false,
            show_shop_inventory: false,
            item_tiles: item_tiles.shallow_copy(),
        }
    }
}


fn fire_player_projectile(map: &mut Map, fire_at: Vector2<f32>, speaker: &mut SoundPlayer) -> u64
{
    let player = map.layers[MAP_OBJECT_LAYER].get(&map.player_id).unwrap();
    let pc = player.creature.as_ref().unwrap();

    fire_projectile(map, "Fireball", fire_at, pc.projectile_spawn_distance, speaker)
}


pub fn fire_projectile(map: &mut Map, kind: &str, fire_at: Vector2<f32>, 
                       start_distance: f32,
                       speaker: &mut SoundPlayer) -> u64 
{
    let id = map.player_id;
    let player = map.layers[MAP_OBJECT_LAYER].get_mut(&id).unwrap();
    let factory = &mut map.factory;

    let mut projectile = launch_projectile(player.position, fire_at, start_distance, 
                                           MobType::PlayerProjectile, factory);
    map.projectile_builder.configure_projectile(kind, &mut projectile.visual, &mut projectile.velocity, speaker);

    let uid = projectile.uid;
    map.layers[MAP_OBJECT_LAYER].insert(uid, projectile);

    uid
}


pub fn launch_projectile(shooter_position: Vector2<f32>, fire_at: Vector2<f32>,
                         start_distance: f32, 
                         projectile_type: MobType, factory: &mut MapObjectFactory) -> MapObject 
{
    println!("New projectile fired at {:?}", fire_at);

    let np = vec2_sub(fire_at, shooter_position);
    let dir = vec2_normalized(np);
    let start_pos = vec2_add(shooter_position, vec2_scale(dir, start_distance));

    let mut projectile = factory.create_mob(1, PROJECTILE_TILESET, start_pos, 12.0, 0.5);
    projectile.velocity = dir;
    projectile.move_time_left = 2.0;
    projectile.move_end_action = MoveEndAction::RemoveFromMap;
    projectile.mob_type = projectile_type;

    projectile
} 


fn drop_loot<R: Rng + ?Sized>(map: &mut Map, killed_mob_list: Vec<MapObject>, 
                              rng: &mut R, speaker: &mut SoundPlayer) 
{
    // todo: monster or area levels
    for mob in killed_mob_list {
        let item = map.item_factory.create_random_item(rng, 1, 6, MAGIC_ITEM_CHANCE, MAGIC_FIND_FACTOR);

        map.place_item(item, mob.position);
        speaker.play(Sound::Click, 0.2);
    }
}