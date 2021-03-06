extern crate nalgebra as na;
extern crate ncollide;

use na::{Isometry3, Vector3};
use ncollide::shape::Ball;
use ncollide::bounding_volume;
use ncollide::broad_phase::{BroadPhase, DBVTBroadPhase};

fn main() {
    /*
     * Create the objects.
     */
    let poss = [
        Isometry3::new(Vector3::new(0.0, 0.0, 0.0), na::zero()),
        Isometry3::new(Vector3::new(0.0, 0.5, 0.0), na::zero()),
        Isometry3::new(Vector3::new(0.5, 0.0, 0.0), na::zero()),
        Isometry3::new(Vector3::new(0.5, 0.5, 0.0), na::zero()),
    ];

    // We will use the same shape for the four objects.
    let ball = Ball::new(0.5);

    /*
     * Create the broad phase.
     * We know we will use small uids (from 0 to 3)so we can pass `true`
     * as the second argument.
     */
    let mut bf = DBVTBroadPhase::new(0.2);

    // First parameter:  a unique id for each object.
    // Second parameter: the object bounding box.
    // Third parameter:  some data (here, the id that identify each object).
    let proxy1 = bf.create_proxy(bounding_volume::aabb(&ball, &poss[0]), 0);
    let proxy2 = bf.create_proxy(bounding_volume::aabb(&ball, &poss[1]), 1);
    let proxy3 = bf.create_proxy(bounding_volume::aabb(&ball, &poss[2]), 2);
    let proxy4 = bf.create_proxy(bounding_volume::aabb(&ball, &poss[3]), 3);

    // Update the broad phase.
    // The collision filter (first closure) prevents self-collision.
    bf.update(&mut |a, b| *a != *b, &mut |_, _, _| {});

    assert!(bf.num_interferences() == 6);

    // Remove two objects.
    bf.remove(&[proxy1], &mut |_, _| {});
    bf.remove(&[proxy2], &mut |_, _| {});

    // Update the broad phase.
    // The collision filter (first closure) prevents self-collision.
    bf.update(&mut |a, b| *a != *b, &mut |_, _, _| {});

    assert!(bf.num_interferences() == 1)
}
