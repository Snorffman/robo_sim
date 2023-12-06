
use crate::{Triangle, Vec3D , Mat4x4};

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