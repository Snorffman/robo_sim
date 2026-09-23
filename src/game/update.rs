use std::collections::HashMap;

use crate::{game::game_obj::GameObj, lib::{Mat4x4, Mesh}};

/// Performs the game update loop
pub fn update(go: &mut Vec<GameObj>, go_map: &HashMap<&str,usize>, dt: f64) {
    // Move the 2nd cube down
    let cube1 = &mut go[go_map["cube1"]];

    let speed = 0.01;//100_000.0;

    let t = Mat4x4::translation_matrix(0.0,-speed * dt, 0.0);
    // cube1.mesh.triangles. 
    cube1.transform(&t);

}