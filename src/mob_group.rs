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
use crate::rng_source::RngReceiver;


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


impl MobGroup {

    pub fn new(mobs: Vec<u64>, center: Vector2<f32>, mobile: bool, rng: &RngReceiver) -> MobGroup {

        let mut members = Vec::with_capacity(mobs.len());

        for id in mobs {
            members.push(MobGroupMember {
                id,
                action_countdown: 0.1 + rng.random(),
                mobile,
            });
        }

        MobGroup {
            center,
            members,
        }
    }


    pub fn update(&mut self, player_id: u64, dt: f32, mobs: &mut HashMap<u64, MapObject>, 
                  rng: &RngReceiver, 
                  factory: &mut MapObjectFactory, projectile_builder: &mut ProjectileBuilder,
                  speaker: &mut SoundPlayer) {
            
        let player_position = mobs.get(&player_id).unwrap().position;

        let mut kill_list = Vec::new();
        let mut index = 0;

        for member in &mut self.members {

            let mob_opt = mobs.get_mut(&member.id);

            match mob_opt {
                None => {
                    // no longer on the map, remove from group
                    kill_list.insert(0, index);
                }
                Some(mob) => {
                    member.action_countdown -= dt;

                    if member.action_countdown < 0.0 {

                        // fire at a player?
                        if rng.random() < 0.25 {

                            // player in range?
                            let len = vec2_square_len(vec2_sub(mob.position, player_position));
                            let reach = 500.0 * 500.0;
                            if len < reach {
                                
                                let projectile_spawn_distance = mob.creature.as_ref().unwrap().projectile_spawn_distance;
                                let mut projectile = launch_projectile(mob.position, player_position, projectile_spawn_distance, MobType::CreatureProjectile, factory);
                                projectile_builder.configure_projectile("Iron shot", &mut projectile.visual, &mut projectile.velocity, speaker);
                                mobs.insert(projectile.uid, projectile);

                                member.action_countdown = 1.0 + rng.random();
                            }
                        }
                        else if member.mobile {
                            
                            // move
                            let mut count = 0;
                            let mut x;
                            let mut y;

                            loop {
                                x = mob.position[0] + 100.0 - rng.random() * 200.0;
                                y = mob.position[1] + 100.0 - rng.random() * 200.0;

                                let dx = x - self.center[0];
                                let dy = y - self.center[1];

                                let len = dx * dx + dy * dy;
                                count += 1;

                                // println!("len={}", len);

                                if len < 100.0 * 100.0 || count >= 5 { break; }
                            } 

                            if count >= 5 {
                                println!("make {} return from {:?} to group center at {:?}", mob.uid, mob.position, self.center);
                                x = self.center[0] + 50.0 - rng.random() * 100.0;
                                y = self.center[1] + 50.0 - rng.random() * 100.0;
                            }

                            // println!("id=" + creature.id + "moves to " + x + ", " + y);

                            let creature = mob.creature.as_ref().unwrap();
                            move_mob(mob, [x, y], creature.base_speed);
                            
                            member.action_countdown = 3.0 + rng.random() * 2.0;
                        }
                    }
                }
            }

            index += 1;
        }

        for index in kill_list {
            self.members.remove(index);
        }

        // todo: cleaup of groups with no members left?
    }
}