
use std::vec;

use piston_window::color;
// use robo_sim4::Tex2D;

use crate::{Mat4x4, Triangle, Vec3D, lib::{Tex2D, lerp, vec_lerp}};

////! Includes normals, lights,  

pub fn normal(triangle : &Triangle) -> Vec3D{ //// Find the cross product & normalize it.
    // Doesn't matter which point we leave out, only need 2. Origin of trig = vec1.
    let line1 : Vec3D = Vec3D::new( // Line pointing from vec1 to vec2.
        triangle.p[1].x - triangle.p[0].x, 
        triangle.p[1].y - triangle.p[0].y,
        triangle.p[1].z - triangle.p[0].z,
    );
    let line2 : Vec3D = Vec3D::new(  // vec1 to vec 3
        triangle.p[2].x - triangle.p[0].x, 
        triangle.p[2].y - triangle.p[0].y,
        triangle.p[2].z - triangle.p[0].z,
    );

    // Cross product:
    let mut normal: Vec3D = Vec3D::zero();
    normal.x = (line1.y * line2.z) - (line1.z * line2.y);
    normal.y = (line1.z * line2.x) - (line1.x * line2.z);
    normal.z = (line1.x * line2.y) - (line1.y * line2.x);

    // Normalize the normal!    Each component divided by (length, via pythag). 
    let len: f64 = ( normal.x*normal.x + normal.y*normal.y + normal.z*normal.z).sqrt();
    normal.x /= len ; normal.y /= len ; normal.z /= len;

    normal
}


////! There's a lot of things that could've gone wrong here.
//-------- Cameras
pub fn matrix_point_at(pos: &Vec3D, target: &Vec3D, up: &Vec3D) -> Mat4x4 { // Assuming pos = from (obj pos), to = target (cam)
    ////! 1) Find our 3 perpendicular vectors.
    //? Calculate new forward direction.
    let new_forward : Vec3D = target.sub_vec(pos) // Vector pointing from 'from', to 'to'
        .normalized();

    // Even tho we've provided an 'up' vector param, the actual up vector might change , as the new forward vector may have a y component that gives it some elevation. (pitching; rotating along the x axis)
    //? Calculate new Up direction - How much our 'up' param projects onto the new forward vector, and adjust it accordingly (it should be perpendicular)
    let dp : f64 = up.dot(&new_forward);
    let a : Vec3D = new_forward.mult_scalar(dp);
    
    let new_up : Vec3D = up.sub_vec(&a)
        .normalized();

    //? Calculate new Right direction. (it's just the cross product)
    let new_right : Vec3D = new_up.cross(&new_forward);

    //----------------- Construct the point_at matrix. (Constructing the Dimensioning & Translation Matrix)
    let mut mat : Mat4x4 = Mat4x4::zero();
    mat.O[0][0] = new_right.x  ; mat.O[0][1] = new_right.y  ; mat.O[0][2] = new_right.z;
    mat.O[1][0] = new_up.x     ; mat.O[1][1] = new_up.y     ; mat.O[1][2] = new_up.z;
    mat.O[2][0] = new_forward.x; mat.O[2][1] = new_forward.y; mat.O[2][2] = new_forward.z;
    mat.O[3][0] = pos.x        ; mat.O[3][1] = pos.y        ; mat.O[3][2] = pos.z;

    mat
}

// THIS IS A CHEAT ONLY for Rotation/ Translation Matrices
pub fn mat_quick_inverse(m : &Mat4x4) -> Mat4x4 {
    let mut mat : Mat4x4 = Mat4x4::zero();

    mat.O[0][0] = m.O[0][0] ; mat.O[0][1] = m.O[1][0];  mat.O[0][2] = m.O[2][0];  mat.O[0][3] = 0.0;
    mat.O[1][0] = m.O[0][1] ; mat.O[1][1] = m.O[1][1];  mat.O[1][2] = m.O[2][1];  mat.O[1][3] = 0.0;
    mat.O[2][0] = m.O[0][2] ; mat.O[2][1] = m.O[1][2];  mat.O[2][2] = m.O[2][2];  mat.O[2][3] = 0.0;

    mat.O[3][0] = -(m.O[3][0] * mat.O[0][0] + m.O[3][1] * mat.O[1][0] + m.O[3][2] * mat.O[2][0]);
    mat.O[3][1] = -(m.O[3][0] * mat.O[0][1] + m.O[3][1] * mat.O[1][1] + m.O[3][2] * mat.O[2][1]);
    mat.O[3][2] = -(m.O[3][0] * mat.O[0][2] + m.O[3][1] * mat.O[1][2] + m.O[3][2] * mat.O[2][2]);
    mat.O[3][3] = 1.0f64;

    mat
}


// pub fn matrix_quick_inverse(&mut self)  {



//*------------------- Clipping -------------------- */
/// Finds the intersection between a vector and a plane
/// plane_p: a point that lies on the plane
/// 
/// plane_n: The normal vector of the plane
/// 
/// vec_a: the starting position of the vector
/// 
/// vec_b: the direction vector of the vector
///! Could be an error here, since I used my own method & idk if he treats p as a different thing.
pub fn vec_intersect_plane(plane_p: &Vec3D, plane_n: &Vec3D, vec_a: &Vec3D, vec_b: &Vec3D) -> Option<(Vec3D, f64)> {
    let b_dot_n = vec_b.dot(&plane_n);


    if b_dot_n == 0.0 {
        return None
    }else {
        let  t = plane_p.sub_vec(&vec_a).dot(&plane_n) / b_dot_n;
        return Some( (vec_a.add_vec( &vec_b.mult_scalar(t)  ), t) )
    }
}



// pub fn vec_intersect_plane_old(plane_p: &Vec3D, plane_n: &Vec3D, vec_a: &Vec3D, vec_b: &Vec3D) -> Option<Vec3D> {
//     println!("vec_b = ({:.3},{:.3},{:.3})", vec_b.x, vec_b.y, vec_b.z);
//     println!("vec_a = ({:.3},{:.3},{:.3})", vec_a.x, vec_a.y, vec_a.z);
//     println!("plane_p= ({:.3},{:.3},{:.3})", plane_p.x, plane_p.y, plane_p.z);
//     println!("plane_n= ({:.3},{:.3}, {:.3})\n", plane_n.x, plane_n.y, plane_n.z);
    
//     let b_dot_p = vec_b.dot(&plane_p);

//     if b_dot_p == 0.0 {
//         return None
//     }else {
//         let  t = plane_p.sub_vec(&vec_a).dot(&plane_n) / b_dot_p;
//         println!("t={}", t);
//         return Some(vec_a.add_vec( &vec_b.mult_scalar(t)  ) )
//     }
// }




// pub fn vec_intersect_plane(plane_p: &Vec3D, plane_n: &Vec3D, vec_a: &Vec3D, vec_b: &Vec3D) -> Option<Vec3D> {
//     let plane_n = plane_n.normalized();
//     let plane_d = - plane_n.dot(plane_p);
//     let ad = vec_a.dot(&plane_n);
//     let bd = 

// }





/// Clips the given input triangle against the plane, and changes the output triangles accordingly that should be drawn
/// Inputs:
/// plane_p: a point that lies on the plane
/// 
/// plane_n: The normal vector of the plane
/// 
/// in_tri: input triangle
/// 
/// Outputs:
/// out_tri1, out_tri2, out_tri3: output triangles (viea mutability)
/// num_out_tri : the number of output triangles

pub fn triangle_clip_plane(plane_p: &Vec3D, plane_n: &Vec3D, in_tri: &Triangle, color_it: bool)  -> Vec<Triangle> {

    // Normalize plane normal
    let plane_n = plane_n.normalized(); // improves efficiency since now we can assume magnitude is 1.

    // Distance of a given point to the plane
    let dist = |p: &Vec3D| -> f64 {
        ( p.sub_vec(plane_p) ).dot(&plane_n)
    };

    // If distance is positive, point lies on "inside" of plane
    let zero3_vec = Vec3D::zero(); let zero2_vec = Tex2D::zero();
    let mut inside_points:  [(&Vec3D, &Tex2D);3] = [(&zero3_vec,&zero2_vec); 3]; let mut inside_point_count: usize = 0;
    let mut outside_points:  [(&Vec3D, &Tex2D);3]  = [(&zero3_vec,&zero2_vec); 3];  let mut outside_point_count: usize = 0;

    // Find distances
    let d0: f64 = dist(&in_tri.p[0]);
    let d1: f64 = dist(&in_tri.p[1]);
    let d2: f64 = dist(&in_tri.p[2]);
    let distances = [d0, d1, d2];

    // Classify inside and outside points
    for i in 0..distances.len() {
        let distance = distances[i];
        if distance >= 0.0 {
            inside_points[inside_point_count] = (&in_tri.p[i], &in_tri.tex[i]);
            inside_point_count += 1;
        }else {
            outside_points[outside_point_count] = (&in_tri.p[i], &in_tri.tex[i]);
            outside_point_count += 1;
        }
    }
    // println!("inside={:?}\noutside={:?}", inside_points, outside_points);
    // println!("inside={}, out={}\n\n", inside_point_count, outside_point_count);

    // Construct triangles & go through the cases
    if inside_point_count == 0 {
        return vec![]; // no returned triangles are valid
    }
    else if inside_point_count == 3 {
        // All points lie on the inside of the plane, so let the triangle pass through
        let out_tri1 = in_tri.clone();

        return vec![out_tri1];
    }
    else if inside_point_count == 1 && outside_point_count == 2 {
        let mut out_tri1 = Triangle::zero();
        // Copy appearence info into new triangle
        out_tri1.col = if color_it {[0,0,255,255]} else {in_tri.col}; // in_tri.col; // 

        // The inside point is valid, so keep that
        out_tri1.p[0] = *inside_points[0].0;    out_tri1.tex[0] = *inside_points[0].1;

        // The two new points are at the locations where the original sides of the triangle (lines) intersect with the plane
        //println!("{:?} {:?}", plane_n, inside_points);
        let (i1, t1) = vec_intersect_plane(plane_p, &plane_n, inside_points[0].0, &outside_points[0].0.sub_vec(&inside_points[0].0)).unwrap(); 
        out_tri1.p[1] = i1;     out_tri1.tex[1] = Tex2D::vec_lerp(inside_points[0].1, outside_points[0].1, t1);

        let (i2, t2) = vec_intersect_plane(plane_p, &plane_n, inside_points[0].0, &outside_points[1].0.sub_vec(&inside_points[0].0)).unwrap();
        out_tri1.p[2]  = i2;    out_tri1.tex[2] = Tex2D::vec_lerp(inside_points[0].1, outside_points[1].1, t2);

        return vec![out_tri1]; // return the newly formed triangle
    } 
    else if inside_point_count == 2 && outside_point_count == 1 {
        let mut out_tri1:Triangle = Triangle::zero(); let mut out_tri2:Triangle = Triangle::zero();
        // Return 2 smaller triangles
        out_tri1.col = if color_it {[0, 255, 0, 255]} else {in_tri.col}; //in_tri.col; 
        out_tri2.col = if color_it {[255, 0, 0, 255]} else {in_tri.col}; //in_tri.col;

        // The first triangle consists of the 2 inside points and a point where one side of the triangle intersects w/ the plane
        out_tri1.p[0] = *inside_points[0].0;   out_tri1.tex[0] = *inside_points[0].1;
        out_tri1.p[1] = *inside_points[1].0;   out_tri1.tex[1] = *inside_points[1].1;
        let (i1, t1) = vec_intersect_plane(plane_p, &plane_n, inside_points[0].0, &outside_points[0].0.sub_vec(&inside_points[0].0)).unwrap();
        out_tri1.p[2] = i1;                   out_tri1.tex[2] = Tex2D::vec_lerp(inside_points[0].1, outside_points[0].1, t1);

        // The second triangle consists of 1 inside points, the newly created point, and the intersection of the OTHER side of the triangle and the plane
        out_tri2.p[0] = *inside_points[1].0;  out_tri2.tex[0] = *inside_points[1].1;
        out_tri2.p[1] = out_tri1.p[2];        out_tri2.tex[1] = out_tri1.tex[2];
        let (i2, t2) = vec_intersect_plane(plane_p, &plane_n, inside_points[1].0, &outside_points[0].0.sub_vec(&inside_points[1].0)).unwrap();
        out_tri2.p[2] = i2;                   out_tri2.tex[2] = Tex2D::vec_lerp(inside_points[1].1 , outside_points[0].1, t2);

        return vec![out_tri1, out_tri2]; // return 2 newly formed triangles which form a quad
    }





    vec![]

}