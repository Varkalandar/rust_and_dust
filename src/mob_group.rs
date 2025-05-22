use std::collections::HashMap;

use rand::Rng;
use rand::rngs::StdRng;
use vecmath::{Vector2, vec2_sub, vec2_square_len};

use crate::map::MapObject;
use crate::map::MapObjectFactory;
use crate::map::MobType;
use crate::map::move_mob;
use crate::game::launch_projectile;
use crate::projectile::ProjectileBuilder;
use crate::SoundPlayer;


pub struct MobGroup {

    // Group center x and y - the group should move as a whole
    center: Vector2<f32>,

    members: Vec<MobGroupMember>,
}

pub struct MobGroupMember {
    id: u64,

    // seconds till next action
    action_countdown: f32,
    mobile: bool,
}

enum MemberAction {
    SHOOT (Vector2<f32>),
    MOVE (Vector2<f32>),
}

impl MobGroup {

    pub fn new(mobs: Vec<u64>, center: Vector2<f32>, mobile: bool, rng: &mut StdRng) -> MobGroup {

        let mut members = Vec::with_capacity(mobs.len());

        for id in mobs {
            members.push(MobGroupMember {
                id,
                action_countdown: 0.1 + rng.random::<f32>(),
                mobile,
            });
        }

        MobGroup {
            center,
            members,
        }
    }


    pub fn update(&mut self, player_id: u64, dt: f32, mobs: &mut HashMap<u64, MapObject>, rng: &mut StdRng, 
                  factory: &mut MapObjectFactory, projectile_builder: &mut ProjectileBuilder,
                  speaker: &mut SoundPlayer) {
            
        let player_position = mobs.get(&player_id).unwrap().position;

        let mut kill_list = Vec::new();
        let mut action_map: HashMap<u64, MemberAction> = HashMap::new();
        let mut index = 0;

        // count down the action timers
        for member in &mut self.members {
            member.action_countdown -= dt;
        }

        // now see who is ready to do something
        for member in &self.members {

            let mob_opt = mobs.get(&member.id);

            match mob_opt {
                None => {
                    // no longer on the map, remove from group
                    kill_list.insert(0, index);
                }
                Some(mob) => {

                    if member.action_countdown < 0.0 {

                        // fire at a player?
                        if rng.random::<f32>() < 0.25 {

                            // player in range?
                            let len = vec2_square_len(vec2_sub(mob.position, player_position));
                            let reach = 500.0 * 500.0;
                            if len < reach {
                                let action = MemberAction::SHOOT(player_position);
                                action_map.insert(member.id, action);
                            }
                        }
                        else if member.mobile {
                            
                            // move
                            let mut count = 0;
                            let mut x;
                            let mut y;

                            loop {
                                x = mob.position[0] + 100.0 - rng.random::<f32>() * 200.0;
                                y = mob.position[1] + 100.0 - rng.random::<f32>() * 200.0;

                                let dx = x - self.center[0];
                                let dy = y - self.center[1];

                                let len = dx * dx + dy * dy;
                                count += 1;

                                // println!("len={}", len);

                                if len < 100.0 * 100.0 
                                   && self.is_destination_clear(mobs, x, y)
                                   || count >= 5 { break; }
                            } 

                            if count >= 5 {
                                println!("make {} return from {:?} to group center at {:?}", mob.uid, mob.position, self.center);
                                x = self.center[0] + 50.0 - rng.random::<f32>() * 100.0;
                                y = self.center[1] + 50.0 - rng.random::<f32>() * 100.0;
                            }

                            // println!("id=" + creature.id + "moves to " + x + ", " + y);
                            let action = MemberAction::MOVE([x, y]);
                            action_map.insert(member.id, action);
                        }
                    }
                }
            }

            index += 1;
        }

        // now perform the chosen actions
        for member in &mut self.members {
            let action = action_map.get(&member.id);
            if action.is_some() {
                match action.unwrap() {
                    MemberAction::SHOOT(target_position) => {
                        fire_at(mobs, member.id, *target_position, 
                                factory, projectile_builder, speaker);
                        member.action_countdown = 1.0 + rng.random::<f32>() * 1.0;
                    },
                    MemberAction::MOVE(target_position) => {
                        move_to(mobs, member.id, *target_position);
                        member.action_countdown = 3.0 + rng.random::<f32>() * 2.0;
                    },
                }
            }
        }

        for index in kill_list {
            self.members.remove(index);
        }

        // todo: cleaup of groups with no members left?
    }

    fn is_destination_clear(&self, mobs: &HashMap<u64, MapObject>, x: f32, y: f32) -> bool
    {
         for member in &self.members {
            let mob_opt = mobs.get(&member.id);
            match mob_opt {
                None => { /* doesn't exist anymore. Will be cleaned up after moving.*/ },
                Some(mob) => {

                    // find out where this mob is moving to. That is the spot
                    // we want to avoid moving to as well to avoid collisions
                    let time = if mob.move_time_left > 0.0 {mob.move_time_left} else {0.0}; 
                    let mob_x = mob.position[0] + mob.velocity[0] * time;
                    let mob_y = mob.position[1] + mob.velocity[1] * time;

                    // check distance
                    let dx = mob_x - x;
                    let dy = mob_y - y;
                    let len2 = dx * dx + dy * dy;
                    
                    let size = mob.creature.as_ref().unwrap().projectile_spawn_distance;

                    if len2 < size * size {
                        // loction is too close to another mob of the group
                        return false;
                    }
                }
            }
        }

        return true;
    }
}


fn move_to(mobs: &mut HashMap<u64, MapObject>, member_id: u64, target_position: Vector2::<f32>) 
{
    let mob = mobs.get_mut(&member_id).unwrap();
    let creature = mob.creature.as_ref().unwrap();
    move_mob(mob, target_position, creature.base_speed);
}


fn fire_at(mobs: &mut HashMap<u64, MapObject>, member_id: u64, target_position: Vector2::<f32>, 
           factory: &mut MapObjectFactory, projectile_builder: &mut ProjectileBuilder,
           speaker: &mut SoundPlayer) 
{
    let mob = mobs.get(&member_id).unwrap();
    let projectile_spawn_distance = mob.creature.as_ref().unwrap().projectile_spawn_distance;
    let mut projectile = launch_projectile(mob.position, target_position, projectile_spawn_distance, MobType::CreatureProjectile, factory);
    projectile_builder.configure_projectile("Iron shot", &mut projectile.visual, &mut projectile.velocity, speaker);
    mobs.insert(projectile.uid, projectile);
}
