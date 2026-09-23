// use piston_window::types::[u8;4];
use std::{fs, error::Error};

use image::{ImageBuffer, Rgba};
use regex::Regex;

pub type FrameBuf = ImageBuffer<Rgba<u8>, Vec<u8>>;
pub type Color = [u8; 4];

#[allow(dead_code)]
pub const PIX_MULT :i32 = 4;

pub fn vec_lerp(a: &Vec<f64>, b: &Vec<f64>, t: f64) -> Vec<f64> {
    assert!(a.len() == b.len());
    let mut v: Vec<f64> = Vec::with_capacity(a.len());
    for i in 0..a.len() {v.push(  lerp(a[i], b[i], t)) }
    v
}
pub fn lerp(a:f64, b:f64, t: f64) -> f64 {
    a + (b-a)*t
}


#[allow(dead_code)]
#[derive(Debug)]
#[derive(Clone, Copy)] // Since vec3Ds are small enough I think it's fair; Keep checking by removing the Copy.
pub struct Vec3D{ pub x : f64,  pub y : f64, pub z : f64}
impl Vec3D{
    pub fn zero() -> Self{
        Vec3D { x: 0.0f64, y: 0.0f64, z: 0.0f64}
    }
    
    pub fn new(x: f64, y: f64, z: f64) -> Self{
        Vec3D { x: x, y: y, z: z}
    }
    //----------------------------------------------
    pub fn add_vec(&self, v2: &Vec3D) -> Vec3D { // Change position
        Vec3D::new(self.x + v2.x, self.y + v2.y,  self.z + v2.z )
    }
    pub fn sub_vec(&self, v2 : &Vec3D) -> Vec3D{ // Direction resultant vector
        Vec3D::new(self.x - v2.x, self.y - v2.y,  self.z - v2.z )
    }
    pub fn mult_scalar(&self, s : f64) -> Vec3D{ // Scaling vectors
        Vec3D::new(self.x * s, self.y * s, self.z * s )
    }
    pub fn div_scalar(&self, s : f64) -> Vec3D {// Scaling vectors
        Vec3D::new(self.x / s,self.y / s, self.z / s )
    }

    pub fn dot(&self, v2 : &Vec3D) -> f64{ // Dot product, similarity of 2 vectors.
        self.x * v2.x + self.y * v2.y + self.z * v2.z
    }
    pub fn cross(&self, v2 : &Vec3D) -> Vec3D{ // Cross product is an un-normalized normal between 3D vecs.
        let cross_prod : Vec3D = Vec3D::new(
            (self.y * v2.z) - (self.z * v2.y),
            (self.z * v2.x) - (self.x * v2.z),
            (self.x * v2.y) - (self.y * v2.x),
        );
        cross_prod
    }
    pub fn len(&self) -> f64{ // Magnitude of vector
        (self.x*self.x + self.y*self.y + self.z*self.z).sqrt()
    }
    pub fn normalized(&self) -> Vec3D { // Get magnitude = 1, so it's only direction.
        let len : f64 = self.len();
        Vec3D::new(
            self.x / len,
            self.y / len, 
            self.z / len,
        )
    }

    pub fn vec_multiply_mat( v_old : &Vec3D, pmat: &Mat4x4) -> Vec3D{
    
        //println!("{:?}", pmat.O);
        let mut v_new : Vec3D = Vec3D::zero();
        //Multiply   Same method as shown when creating the projction matrix.
        v_new.x = v_old.x * pmat.O[0][0] + v_old.y * pmat.O[1][0] + v_old.z * pmat.O[2][0]  + pmat.O[3][0]; // Simply sum the final one because 4 element of input vector it's 1.
        v_new.y = v_old.x * pmat.O[0][1] + v_old.y * pmat.O[1][1] + v_old.z * pmat.O[2][1]  + pmat.O[3][1];
        v_new.z = v_old.x * pmat.O[0][2] + v_old.y * pmat.O[1][2] + v_old.z * pmat.O[2][2]  + pmat.O[3][2];
    
        let w : f64 =  v_old.x * pmat.O[0][3] + v_old.y * pmat.O[1][3] + v_old.z * pmat.O[2][3] + pmat.O[3][3];
        
        if w != 0.0f64{
            v_new.x /= w ; v_new.y /= w ; v_new.z /= w;
        } 
    
        v_new
    }

    pub fn vec_lerp(a: &Self, b: &Self, t: f64) -> Self {
        Self::new(lerp(a.x,b.x,t), lerp(a.y, b.y, t), lerp(a.z,b.z,t))
    }

}

#[allow(dead_code)]
#[derive(Debug)]
#[derive(Clone)]
pub struct Triangle{
    pub p : [Vec3D ; 3],
    pub col : Color,
    pub tex: [Tex2D; 3],
}
impl Triangle{
    pub fn zero() -> Self{
        Triangle { p: [Vec3D::zero(); 3], col: [255, 255, 255, 1], tex: [Tex2D::zero();3] }
    }
    /// Creates a triangle while letting you fully specify things like its texture
    pub fn new_full(vec1: Vec3D, vec2: Vec3D, vec3: Vec3D, tex1: Tex2D,tex2: Tex2D, tex3: Tex2D) -> Self {
        Triangle { p: [Vec3D::from(vec1), Vec3D::from(vec2), Vec3D::from(vec3)] , col: [255, 255, 255, 1], tex: [tex1,tex2,tex3]}
    }


    pub fn new(vec1: Vec3D, vec2: Vec3D, vec3: Vec3D) -> Self{ // Col system is pretty inefficient
        Triangle { p: [Vec3D::from(vec1), Vec3D::from(vec2), Vec3D::from(vec3)] , col: [255, 255, 255, 1], tex: [Tex2D::zero();3]}
    }
    

    pub fn col(&mut self, col : [u8;4]) { // [u8;4]ed new
       self.col = col;
    }

    pub fn apply_light(&mut self, light : &[u8;4]){
        self.col[0] = light[0];
        self.col[1] = light[1];
        self.col[2] = light[2];
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Mesh{
    pub triangles: Vec< Triangle >,
}
impl Mesh{
    pub fn zero() -> Self{
        Mesh{  triangles : vec![Triangle { p: [Vec3D::zero() ;3 ], col: [255, 255, 255, 1], tex: [Tex2D::zero();3] } ] }
    }
    pub fn new(triangles: Vec< Triangle > ) -> Self{
        Mesh { triangles: triangles}
    }

    pub fn load_from_obj(path : &str) -> Result<Mesh, Box<dyn Error> >{
        let contents: String= fs::read_to_string(path)?;
        let textures_enabled = contents.contains("\nvt ");

        // Tempory pool of vertices (local cache of verts)
        let mut verts : Vec<Vec3D> = Vec::new(); ////! Keep in mind, the dude pushed_back tho potentially less effient, so we're also doing that.
        let mut tex_verts: Vec<Tex2D> = Vec::new();
        let mut trigs : Vec<Triangle> = Vec::new(); // All the triangles (faces)
        
        // Ignore everything prior to the first v
        let first_v = contents.find("v ").unwrap();
        let contents = &contents[first_v..];


        // Iterate through the file
        // let re_v = Regex::new(r"\s+([0-9\.-]*)\s+([0-9\.-]*)\s+([0-9\.-]*)\s?").unwrap();
        let re_v = Regex::new(r"\s+([0-9\.-]*)\s+([0-9\.-]*)\s*([0-9\.-]*)?\s?").unwrap();
        
        let re_split = Regex::new(r" +").unwrap();
        for line in contents.lines() {
            let v : Vec3D; // vertex
            

            // Shouldn't do if the line isn't v or f, but that only happens like 3 times so who cares.
            let line_trimmed = line.trim();
            let mut chr_itr   = line_trimmed.split(" "); // Iter<char>, split by spaces

            let indicator : &str = chr_itr.next().unwrap(); // Junk eg ('v')

            if indicator == "v"{ //? Vertex handle
                let groups = re_v.captures(line).unwrap();
                let x:f64 = groups.get(1).unwrap().as_str().parse().unwrap();
                let y:f64 = groups.get(2).unwrap().as_str().parse().unwrap();
                let z:f64 = groups.get(3).unwrap().as_str().parse().unwrap();

                
                v = Vec3D::new(x, y, z);
                verts.push(v);
            }
            else if indicator == "vt" {
                let groups = re_v.captures(line).unwrap();

                let x:f64 = groups.get(1).unwrap().as_str().parse().unwrap(); let y:f64 = groups.get(2).unwrap().as_str().parse().unwrap(); 
                // let z:f64 = groups.get(3).unwrap().as_str().parse().unwrap();
                // if let Some(_z) = groups.get(3) {
                //     if let Ok(z) = _z.as_str().parse::<f64>() {
                //         let tex =  Tex2D::init(x, y, z);
                //         tex_verts.push(tex);
                //     }
                // }else {
                    let tex = Tex2D::init(x, y, 1.0);
                    tex_verts.push(tex);
                // }


            }
            else if indicator == "f"{ //? Face handle (making a triangle)
                // First split with a regex on space split: If it has 4 splits, then we can work differently
                let split_raw: Vec<Vec<&str>> = re_split.split(line_trimmed)
                    .map(|point_info| point_info.split("/").collect())
                    .collect();


                let split: Vec<Vec<usize>> = (1..split_raw.len()).into_iter().map(|i| {
                    split_raw[i].iter().map(|n| n.parse::<usize>().unwrap()).collect::<Vec<usize>>()
                }).collect();


                if split.len()  == 3 {
                    let v1_i: usize = split[0][0];
                    let v2_i: usize = split[1][0];
                    let v3_i: usize = split[2][0];
                    let vec1 = verts[v1_i - 1]; let vec2 = verts[v2_i - 1]; let vec3 = verts[v3_i - 1];
                    let mut f1 = Triangle::new(vec1, vec2, vec3);

                    if textures_enabled {
                        let t1_i:usize = split[0][1]; let t2_i:usize = split[1][1]; let t3_i:usize = split[2][1];
                        f1.tex[0] = tex_verts[t1_i-1]; f1.tex[1] = tex_verts[t2_i-1];  f1.tex[2] = tex_verts[t3_i-1];
                    }
                    trigs.push(f1);
                }
                if split.len() == 4{
                    let v1_i: usize = split[0][0];
                    let v2_i: usize = split[1][0];
                    let v3_i: usize = split[2][0];
                    let v4_i: usize = split[3][0];
                    let vec1 = verts[v1_i - 1]; let vec2 = verts[v2_i - 1]; let vec3 = verts[v3_i - 1]; let vec4 = verts[v4_i - 1];
                    let mut f1 = Triangle::new(vec1, vec2, vec3);
                    let mut f2 = Triangle::new(vec1, vec3, vec4);

                    if textures_enabled {
                        let t1_i:usize = split[0][1]; let t2_i:usize = split[1][1]; let t3_i:usize = split[2][1]; let t4_i:usize = split[3][1];
                        f1.tex[0] = tex_verts[t1_i-1]; f1.tex[1] = tex_verts[t2_i-1];  f1.tex[2] = tex_verts[t3_i-1];
                        f2.tex[0] = tex_verts[t1_i-1]; f2.tex[1] = tex_verts[t3_i-1]; f2.tex[2] = tex_verts[t4_i-1];
                    }
                    trigs.push(f1);
                    trigs.push(f2);
                }
            }

        }
        let new_mesh :Mesh = Mesh::new(trigs);        
        
        Ok( new_mesh )

    }


}


#[derive(Debug)]
#[allow(non_snake_case)]
pub struct Mat4x4{ // O means nothing, like the Rgb.O[1,1,1,]
    pub O: Vec<Vec<f64>> ,
}
impl Mat4x4{
    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ Builds
    pub fn zero() -> Self{ // Fills with default 0 values so doesn't access empty arays which is unsafe.
        Mat4x4 { O: vec![vec![0.0f64;4] ; 4] }
    }
    pub fn build_identity_matrix() -> Mat4x4 { // A type of matrix with weird properties.
        let mut imat : Mat4x4 = Mat4x4::zero();
        imat.O[0][0] = 1.0f64;
        imat.O[1][1] = 1.0f64;
        imat.O[2][2] = 1.0f64;
        imat.O[3][3] = 1.0f64;
        imat
    }
    ////! impls with make_  in them assume they're already built from ::zero(), as they're likely to be built many times.
    //Todo: Could be more efficient by also building the specific 1.0. Have an init_thing_mat, then a make_thing_mat
    
    //? Theta is angle in rads.
    pub fn init_rotx_matrix() -> Mat4x4 {
        let mut rotx_mat : Mat4x4 = Mat4x4::zero();
        rotx_mat.O[0][0] =   1.0;
        rotx_mat.O[3][3] =   1.0;
        rotx_mat
    }
    pub fn make_rotx_matrix(&mut self, theta : f64) {// We could build from ::zero() each time, but that's inefficient.
        // rotx_mat.O[0][0] =   1.0;
        self.O[1][1] =   (theta * 0.5).cos();
        self.O[1][2] =   (theta * 0.5).sin();
        self.O[2][1] = - (theta * 0.5).sin();
        self.O[2][2] =   (theta * 0.5).cos();
        // rotx_mat.O[3][3] =   1.0;
    }
    
    pub fn init_roty_matrix() -> Mat4x4 {
        let mut roty_mat : Mat4x4 = Mat4x4::zero();
        roty_mat.O[1][1] =   1.0;
        roty_mat.O[3][3] =   1.0;
        roty_mat
    }
    pub  fn make_roty_matrix(&mut self, theta : f64) {
        self.O[0][0] =   theta.cos();
        self.O[0][2] =   theta.sin();
        self.O[2][0] = - theta.sin();
        // self.O[1][1] =   1.0;
        self.O[2][2] =   theta.cos();
        // self.O[3][3] =   1.0;
    }

    fn init_rotz_matrix() -> Mat4x4 {
        let mut rotz_mat : Mat4x4 = Mat4x4::zero();
        rotz_mat.O[2][2] = 1.0;
        rotz_mat.O[3][3] = 1.0;
        rotz_mat
    }
    fn make_rotz_matrix(&mut self, theta : f64) { 
        self.O[0][0] =   theta.cos();
        self.O[0][1] =   theta.sin();
        self.O[1][0] = - theta.sin();
        self.O[1][1] =   theta.cos();
        // self.O[2][2] =   1.0;
        // self.O[3][3] =   1.0;
    }

    /// Sometimes its faster to do this sort of process. But yeah its kind of micro-optimization, use translation_matrix for a nicer interface.
    /// ```
    /// let mut t = Mat4x4::init_translation_matrix();
    /// t.make_translation_matrix(x, y, z);
    /// ```
    /// 
    pub fn init_translation_matrix() -> Mat4x4 {
        let mut trans_mat : Mat4x4 = Mat4x4::zero();
        trans_mat.O[0][0] = 1.0;
        trans_mat.O[1][1] = 1.0;
        trans_mat.O[2][2] = 1.0;
        trans_mat.O[3][3] = 1.0;
        trans_mat
    }
    pub fn make_translation_matrix(&mut self, x : f64, y: f64, z : f64) {
        // self.O[0][0] =   1.0;
        // self.O[1][1] =   1.0;
        // self.O[2][2] =   1.0;
        // self.O[3][3] =   1.0;
        self.O[3][0] =   x;
        self.O[3][1] =   y;
        self.O[3][2] =   z;
    }
    
    
    pub fn translation_matrix(x : f64, y: f64, z : f64) -> Mat4x4 {
        let mut t = Self::init_translation_matrix();
        t.make_translation_matrix(x, y, z);
        t
    }
    pub fn translation_matrix_(v: &Vec3D) -> Mat4x4 {
        Self::translation_matrix(v.x, v.y, v.z)
    }
    pub fn rotx_matrix(theta: f64) -> Mat4x4 {
        let mut t = Self::init_rotx_matrix();
        t.make_rotx_matrix(theta);
        return t;
    }
    pub fn roty_matrix(theta: f64) -> Mat4x4 {
        let mut t = Self::init_roty_matrix();
        t.make_roty_matrix(theta);
        return t;
    }
    pub fn rotz_matrix(theta: f64) -> Mat4x4 {
        let mut t = Self::init_rotz_matrix();
        t.make_rotz_matrix(theta);
        return t;
    }
    //------------------------------------ Methods

    // performs:  M1 @ M2
    pub fn mat_mult_mat(&mut self, m2 : &Mat4x4) { // Multiply 2 matrices together
        for c in 0..4{ // collumn
            for r in 0..4{ // row
                self.O[r][c] = {self.O[r][0] * m2.O[0][c] +
                                self.O[r][1] * m2.O[1][c] +
                                self.O[r][2] * m2.O[2][c] +
                                self.O[r][3] * m2.O[3][c]  };
            }
        }
    }

    // performs M1 @ M2,  so m2 is applied first against the vector then m1
    pub fn mat_mult_mat_(mat1: &Mat4x4, mat2: &Mat4x4)-> Mat4x4 {
        let mut new_mat:Mat4x4 = Mat4x4{O: vec![vec![0. ; 4 ] ; 4]} ;

        for c in 0..4{ // collumn
            for r in 0..4{ // row
                new_mat.O[r][c] = {mat1.O[r][0] * mat2.O[0][c] +
                                mat1.O[r][1] * mat2.O[1][c] +
                                mat1.O[r][2] * mat2.O[2][c] +
                                mat1.O[r][3] * mat2.O[3][c]  };
            }
        }
        return new_mat
    }

}


pub fn vec_multiply_mat( v_old : &Vec3D, pmat: &Mat4x4) -> Vec3D{
    //println!("{:?}", pmat.O);
    let mut v_new : Vec3D = Vec3D::zero();
    //Multiply   Same method as shown when creating the projction matrix.
    v_new.x = v_old.x * pmat.O[0][0] + v_old.y * pmat.O[1][0] + v_old.z * pmat.O[2][0]  + pmat.O[3][0]; // Simply sum the final one because 4 element of input vector it's 1.
    v_new.y = v_old.x * pmat.O[0][1] + v_old.y * pmat.O[1][1] + v_old.z * pmat.O[2][1]  + pmat.O[3][1];
    v_new.z = v_old.x * pmat.O[0][2] + v_old.y * pmat.O[1][2] + v_old.z * pmat.O[2][2]  + pmat.O[3][2];

    let w : f64 =  v_old.x * pmat.O[0][3] + v_old.y * pmat.O[1][3] + v_old.z * pmat.O[2][3] + pmat.O[3][3];
    
    if w != 0.0f64{
        v_new.x /= w ; v_new.y /= w ; v_new.z /= w;
    } 


    v_new
}


//*~~~~~~~~~~~~~~~~~~~~~~ Lights ~~~~~~~~~~~~~~~~~~~~~~~*// Could later be done with traits/ enums for generalising
#[derive(Debug)]
pub struct DirLight{
    pub dir : Vec3D,
    pub col : [u8;4],
}
impl DirLight{
    pub fn new(dir: Vec3D, col: [u8;4]) -> Self{
        DirLight { dir: dir, col: col}
    }

    //? The dot product (angle) between the direction of the light and the normal of the object's face being shined on
    pub fn intensity(&mut self, normal : &Vec3D) -> f32{

        let light_len : f64 = (self.dir.x * self.dir.x + self.dir.y * self.dir.y + self.dir.z * self.dir.z).sqrt();
        self.dir.x /= light_len; self.dir.y /= light_len; self.dir.z /= light_len; // Normalize so we can have wacky angles w/out consequence
    
        // Dot product between the normal  of the trig surface and the  light
        let intensity : f32 = (normal.x * self.dir.x + normal.y * self.dir.y + normal.z * self.dir.z ) as f32;
        
        intensity
    }
}

//*---------------------- Textures --------------------------- *//
#[derive(Debug, Clone, Copy)]
pub struct Tex2D {
    pub u: f64,
    pub v: f64,
    pub w: f64,
}
impl Tex2D {
    pub fn zero() -> Self {
        Self { u: 0., v: 0., w: 1.0}
    }
    pub fn new(u: f64, v: f64) -> Self {
        Self { u, v , w: 1.0}
    }
    /// Same as new but also lets you define w for it
    pub fn init(u: f64, v: f64, w: f64) -> Self {
        Self { u, v , w}
    }

    /// Applies lerp on the w component too
    pub fn vec_lerp(a: &Self, b: &Self, t: f64) -> Self {
        Tex2D::init(lerp(a.u,b.u,t), lerp(a.v, b.v, t), lerp(a.w, b.w, t) )
    }
}