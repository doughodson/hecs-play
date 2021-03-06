use hecs::*;
use rand::{thread_rng, Rng};
use std::io;

//
// Simple simulation
// Spawn multiple entities which have health, damage, position and other components
//
// On every tick every entity/unit:
//     1. Moves in random direction
//     2. Finds closest entity to itself
//     3. Fires at it and applies damage
//     4. Gets damaged by other entities firing at them
//     5. If health <= 0, the unit dies
// State of the simulation is displayed in the console using println!
//

#[derive(Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Health(i32);

#[derive(Debug)]
struct Speed(i32);

#[derive(Debug)]
struct Damage(i32);

#[derive(Debug)]
struct KillCount(i32);

fn manhattan_dist(x0: i32, x1: i32, y0: i32, y1: i32) -> i32 {
    let dx = (x0 - x1).abs();
    let dy = (y0 - y1).abs();
    dx + dy
}

fn spawn_entities(world: &mut World, n: usize)
{
    let mut rng = thread_rng();
    for _ in 0..n {
        let pos = Position {
            x: rng.gen_range(-10..10),
            y: rng.gen_range(-10..10),
        };
        let s = Speed(rng.gen_range(1..5));
        let hp = Health(rng.gen_range(30..50));
        let dmg = Damage(rng.gen_range(1..10));
        let kc = KillCount(0);
        world.spawn((pos, s, hp, dmg, kc));
    }
}

fn system_integrate_motion(world: &mut World) {
    let mut rng = thread_rng();

    for (id, (pos, s)) in &mut world.query::<(&mut Position, &Speed)>() {
        let change = (rng.gen_range(-s.0..s.0), rng.gen_range(-s.0..s.0));
        pos.x += change.0;
        pos.y += change.1;
        println!("Unit {:?} moved to {:?}", id, pos);
    }
}

// find the closest entity and fire at them
fn system_fire_at_closest(world: &mut World) {
    for (id0, (pos0, dmg0, kc0)) in
        &mut world.query::<With<Health, (&Position, &Damage, &mut KillCount)>>()
    {
        // find closest
        let closest = world
            .query::<With<Health, &Position>>()
            .iter()
            .filter(|(id1, _)| *id1 != id0)
            .min_by_key(|(_, pos1)| manhattan_dist(pos0.x, pos1.x, pos0.y, pos1.y))
            .map(|(entity, _pos)| entity);

        let closest = match closest {
            Some(entity) => entity,
            None => {
                println!("{:?} is the last survivor!", id0);
                return;
            }
        };

        // deal damage:
        //
        // get target unit hp like this:
        //   let mut hp1 = world.query_one::<&mut Health>(closest_id.unwrap()).unwrap();
        //   let hp1 = hp1.get().unwrap();
        //
        // or like this:
        let mut hp1 = world.get_mut::<Health>(closest).unwrap();

        // is target unit still alive?
        if hp1.0 > 0 {
            // apply damage
            hp1.0 -= dmg0.0;
            println!(
                "Unit {:?} was damaged by {:?} for {:?} HP",
                closest, id0, dmg0.0
            );
            if hp1.0 <= 0 {
                // if this killed it, increase own killcount
                kc0.0 += 1;
                println!("Unit {:?} was killed by unit {:?}!", closest, id0);
            }
        }
    }
}

fn system_remove_dead(world: &mut World) {
    // query entities, if less then or equal to 0 hp, save list to despawn 
    let mut to_remove: Vec<Entity> = Vec::new();
    for (id, hp) in &mut world.query::<&Health>() {
        if hp.0 <= 0 {
            to_remove.push(id);
        }
    }

    // despawn entities on list
    for entity in to_remove {
        world.despawn(entity).unwrap();
    }
}

fn print_world_state(world: &mut World) {
    println!("\nEntity stats:");
    for (id, (hp, pos, dmg, kc)) in &mut world.query::<(&Health, &Position, &Damage, &KillCount)>() {
        println!("ID: {:?}, {:?}, {:?}, {:?}, {:?}", id, hp, dmg, pos, kc);
    }
}

fn main() {
    let mut world = World::new();

    spawn_entities(&mut world, 5);

    loop {
        println!("\n'Enter' to continue simulation, '?' for enity list, 'q' to quit");

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "" => {
                // execute all systems
                system_integrate_motion(&mut world);
                system_fire_at_closest(&mut world);
                system_remove_dead(&mut world);
            }
            "q" => break,
            "?" => {
                print_world_state(&mut world);
            }
            _ => {}
        }
    }
}
