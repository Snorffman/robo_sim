
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::vec::IntoIter;

use piston::Key::P;
use crate::normal_stuff::{self as funky, vec_intersect_plane};

// use robo_sim4::Mat4x4;
use crate::{F64_SCREEN_HEIGHT, F64_SCREEN_WIDTH, draw::{self, _blue, _green, _red}, lib::{FrameBuf, Mat4x4, Mesh, Vec3D}, normal_stuff, vec_multiply_mat};


/// # Axis
/// # Attributes
/// points = ((1,0,0),red), ((0,1,0),blue), ((0,0,1),green) for a standard euclidean axis.
/// 
/// screen pos = The position on screen that the root of the axis is drawn from.
#[derive(Clone)]
pub struct Axis {
    pub points: [(Vec3D,[u8;4] ); 3],

    pub screen_pos: Vec3D,
}
impl Axis {
    pub fn new(p: [Vec3D; 3], screen_pos: [usize; 2]) -> Self {
        let points = [(p[0],_red), (p[1],_blue), (p[2],_green)];
        Axis{points, screen_pos: Vec3D::new(screen_pos[0] as f64, screen_pos[1] as f64,0.0)}
    }

    /// Draws the axis
    pub fn draw(&self, mat_cam_rot: &Mat4x4, fb: &mut FrameBuf) {
        let mut axis_dirs: Vec<(Vec3D,[u8;4])> = Vec::with_capacity(3);
        for (axis_dir,axis_col) in &self.points { 
            let axis_dir = vec_multiply_mat(axis_dir, &mat_cam_rot); 
            axis_dirs.push( (axis_dir, *axis_col) );
        }
        axis_dirs.sort_by(|(a,_),(b,_)| a.z.total_cmp(&b.z));
        
        // Draw it (orthographic projection, sort by z)
        for (axis_dir,axis_col) in &axis_dirs {
            let axis_dir2d = self.screen_pos.add_vec(&Vec3D::new(axis_dir.x, axis_dir.y,0.0).mult_scalar(20.0));
            draw::draw_line_2d(&self.screen_pos,&axis_dir2d, 1.0, 1, *axis_col, fb);
        }
    }
}


pub struct GameObjCollection {
    map: HashMap<String, usize>,
    pub objects: Vec< Box<dyn GameObjType>  >,
}
impl GameObjCollection {
    pub fn from_iter( iter: IntoIter<(&str, Box<dyn GameObjType>  )> ) -> Self  {
        let mut map = HashMap::new();
        let mut objects = Vec::new();
        for (i, (name, obj) ) in iter.enumerate() {
            map.insert(name.to_string(), i);
            objects.push( obj  );
        }
        Self { map, objects }
    }

    pub fn get_object(&mut self, name: &str) ->  *const GameObj {
        return self.objects[ self.map[name] ].get_game_obj() as *const GameObj ;
    }
    pub fn get_mut_object(&mut self, name: &str) -> *mut GameObj {
        return self.objects[self.map[name]].get_mut_game_obj() as *mut GameObj;
    }
    /// gets the trait object from the given location
    pub fn get_unknown(&self, name: &str) -> &Box<dyn GameObjType> {
        &self.objects[self.map[name]]
    }
    pub fn get_mut_unknown(&mut self, name: &str) -> &mut Box<dyn GameObjType> {
        &mut self.objects[self.map[name]]
    }
}

use std::any::Any;

// ":Any" means Any type that inherits GameObjType must inherit Any.
pub trait GameObjType: Any + 'static {
    fn get_game_obj(&self) -> &GameObj;
    fn get_mut_game_obj(&mut self) -> &mut GameObj;
    fn get_collision_box(&self) -> &CollisionBox {
        &self.get_game_obj().collision_box
    }

    fn transform(&mut self, mat: &Mat4x4 ) {
        self.get_mut_game_obj().transform(mat);
    }
    fn get_view_mat(&self, look_dir: &Vec3D, up: Option<&Vec3D>) -> Mat4x4 {
        self.get_game_obj().get_view_mat(look_dir, up)
    }

    //-------------------------
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;

}


#[derive(Clone)]
pub struct GameObj {
    pub name: String,
    pub collision_box: CollisionBox,
    pub mesh: Mesh,
    pub drawable:bool,
}
impl GameObj {
    /// is_static = Means the triangles can never change positions.
    pub fn new(name: &str, collision_box: CollisionBox,  mesh: Mesh, drawable:bool) -> Self {
        Self { name: name.to_string(), collision_box, mesh, drawable}
    }



    /// Applies a matrix transform on the triangles of the game object
    pub fn transform(&mut self, mat: &Mat4x4) {
        for t in &mut self.mesh.triangles {
            for v in &mut t.p {
                *v = vec_multiply_mat(&v, &mat);
            }
        }
        self.collision_box.transform(mat);
    }


    //-------------- Get the camera to centre on this object ------------//
    /// Gets a camera view matrix that points at this objects collision box centre
    pub fn get_view_mat(&self, look_dir: &Vec3D, up: Option<&Vec3D>) -> Mat4x4 {
        let default_up = Vec3D::new(0.,1.,0.);
        let up = up.unwrap_or(&default_up);

        let camera = self.collision_box.centre.add_vec(&Vec3D::new(0., -2.0, 0.0));
        let target_pos : Vec3D = camera.add_vec(&look_dir);
        // if a_is_pressed {cube1.transform(mat);}
        let mat_cam = funky::matrix_point_at(&camera, &target_pos, &up);
        let mat_view = funky::mat_quick_inverse(&mat_cam);

        mat_view
    }

    pub fn static_sat(&mut self, other: &GameObj) -> Option<Vec3D> {
        self.collision_box.static_sat(&other.collision_box)
    }

    pub fn draw_collision_box(&self, mat_proj: &Mat4x4,draw_norms:bool, res:f64, scale:u32,  fb: &mut FrameBuf) {
        self.collision_box.draw(mat_proj, draw_norms, res, scale,  fb);
    }




}
impl GameObjType for GameObj {
    fn get_game_obj(&self) -> &GameObj {self}
    fn get_mut_game_obj(&mut self) -> &mut GameObj {self}
    
    /// as_any converts it into an 'Any' trait, so you can then convert it into its original type
    fn as_any(&self) -> &dyn Any {self}
    fn as_mut_any(&mut self) -> &mut dyn Any {self}
}

// I just realised it would be far easier to just use normals to decide ordering at the start really.
enum Either<L,R> {
    Left(L),
    Right(R),
}
/// Two different ways to create a collision box, either you rely on winding to define the normals, or define the normals yourself.
pub enum FaceInput {
    NormalImplicit(Vec<Vec<usize>>), // second inpt
    NormalExplicit(Vec<(Vec<usize>, Vec3D)>)
}
impl FaceInput {
    fn len(&self) -> usize {
        match self {
            Self::NormalImplicit(v) => v.len(),
            Self::NormalExplicit(v) => v.len(),
        }
    }

    pub fn into_iter(&self) ->  IntoIter< Either<&Vec<usize>, &(Vec<usize>, Vec3D) > > {
        let mut buf = vec![];
        match self {
            Self::NormalImplicit(v) => {
                for e in v {
                    buf.push(Either::Left(e));
                }
            }
            Self::NormalExplicit(v) => {
                for e in v {
                    buf.push(Either::Right(e));
                }
            }
        }

        buf.into_iter()
    }

}


/// p just gives the index numbers to the points
/// normal is the face normal, calculated by using the cross product. Should work as long as >=3 points, and they are coplanar.
#[derive(Clone)]
pub struct Face {
    pub p: Vec<usize>,
    edges: Vec<usize>,
    pub normal: Vec3D,
}
#[derive(Clone)]
pub struct CollisionBox {
    pub points: Vec<Vec3D>,
    pub centre: Vec3D,
    pub edges: Vec<Vec3D>,
    pub faces: Vec<Face>,
    pub color:  [u8;4],
}
impl CollisionBox {
    pub fn new(points: Vec<Vec3D>, faces: FaceInput, color: Option<[u8;4]>) -> Self {
        let color = color.unwrap_or(_red);
        let faces = match faces {
            FaceInput::NormalImplicit(v) => v,
            FaceInput::NormalExplicit(v) => {
                assert!(v.iter().all(|x| x.0.len() >= 3 ));

                // 1) Use the normals to decide whether to reverse ordering
                let mut v_adjusted:Vec<Vec<usize>> = Vec::with_capacity(v.len());

                for (mut face, norm) in v {
                    // let new_face = face.clone();
                    let edge0 = points[face[1]].sub_vec(&points[face[0]]);
                    let edge1 = points[face[2]].sub_vec(&points[face[1]]);
                    let face_norm = edge1.cross(&edge0);
                    if face_norm.dot(&norm) >= 0.0 { // same direction  ; keep same ordering
                        v_adjusted.push(face)
                    }else {
                        face.reverse();
                        v_adjusted.push(face);
                    }
                }
                v_adjusted
            }
        };


        let (unique_edges, box_faces) = Self::recalculate_normals(&points, faces);
        println!("n={}, unique_edges = {:?}\n\n", unique_edges.len(), unique_edges);

        let centre = Self::get_centre(&points);
        CollisionBox { points, centre , edges: unique_edges, faces: box_faces , color}
    }
    

    pub fn get_centre(points: &Vec<Vec3D>) -> Vec3D {
        let mut centre = Vec3D::zero();
        let n = points.len();
        if n > 0 {
            for i in 0..n {
                centre = centre.add_vec( &points[i]);
            }
            centre = centre.div_scalar(n as f64);
        }
        centre
    }
    
    /// Projects and draws the collision box
    pub fn draw(&self, mat_proj: &Mat4x4, draw_norms: bool,  res:f64, scale:u32, fb: &mut FrameBuf) {
        let norm_size = 20.0;

        // Project the points & norms
        let project = |p: &Vec3D| -> Vec3D {
            let p = Vec3D::new(p.x, p.y, p.z+3.0);
            let mut p = vec_multiply_mat(&p, &mat_proj);

            // Shift into view
            p.x = (p.x + 1.0) * 0.5 * F64_SCREEN_WIDTH;
            p.y = (p.y + 1.0) * 0.5 * F64_SCREEN_HEIGHT;
            p
        };


        let points = &self.points;

        // let plane = (Vec3D::new(0.,0.0,-1.5), Vec3D::new(0.,0.,1.) ); // (point, normal)
        let view_planes = vec![
            (Vec3D::new(0.,0.0,-1.5), Vec3D::new(0.,0.,1.) ), // the plane in front of camera
        ];
        // Its just easier to handle these planes in projected space 
        let proj_planes = vec![
            (Vec3D::new(0.0, 1.0, 0.0), Vec3D::new(0.0, 1.0, 0.0)),
            (Vec3D::new(0.0, F64_SCREEN_HEIGHT, 0.0), Vec3D::new(0.0, -1.0, 0.0) ),
            (Vec3D::new(1.0, 0.0, 0.0), Vec3D::new(1.0, 0.0, 0.0)),
            (Vec3D::new(F64_SCREEN_WIDTH, 0.0,0.0), Vec3D::new(-1., 0.,0.) ), // right
        ];
        // println!("ar={:?}")

        let dist = |p: &Vec3D, plane_p: &Vec3D, plane_n: &Vec3D| -> f64 {
            ( p.sub_vec(plane_p) ).dot(&plane_n)
        };

        let draw_line = |v1: &Vec3D, v2: &Vec3D, fb: &mut FrameBuf, view_planes: &Vec<(Vec3D,Vec3D)>, proj_planes: &Vec<(Vec3D, Vec3D)>| {
            let mut draw_line = true; // We can draw it if it is on the correct half space of every plane
            let mut clipped_v1 = v1.clone(); let mut clipped_v2 = v2.clone();


            let clip_against_planes = |planes: &Vec<(Vec3D, Vec3D)>, clipped_v1: &mut Vec3D, clipped_v2: &mut Vec3D, draw_line: &mut bool| {
                'a: for plane in planes {
                    // Clip the line against each plane
                    let d1 =  dist(&clipped_v1, &plane.0, &plane.1);
                    let d2 =  dist(&clipped_v2, &plane.0, &plane.1);

                    let (furthest, other) = if d1 > d2 {(clipped_v1.clone(), clipped_v2.clone())} else {(clipped_v2.clone(), clipped_v1.clone())};
                    let vec_a = furthest;
                    let vec_b = other.sub_vec(&vec_a);

                    if d1 > 0.0 && d2 > 0.0 { // both points are in the half space - then we can just draw it
                        // draw::draw_line_2d(&project(&v1), &project(&v2), res, scale,color,  fb);
                        *draw_line = true;
                    }else if d1> 0.0 || d2 > 0.0 { // at least one point is in the half spaced
                        if let Some( (intersect,t)) = vec_intersect_plane(&plane.0, &plane.1, &vec_a, &vec_b) {
                            // Draw from intersect to furthest point in direction the plane normal is facing
                            if t >= 0.0 { // intersect is in front of the plane
                                if t <= 1.0  {
                                    // draw::draw_line_2d(&project(&intersect), &project(furthest), res, scale, color, fb);
                                    *clipped_v1 = intersect; *clipped_v2 = furthest;
                                }else {
                                    *draw_line = true;
                                }
                            }else {
                                *draw_line = false; break 'a;
                            }
                        }else{ *draw_line = false; break 'a;}
                    }else {*draw_line = false; break 'a;}
                }
            };
            clip_against_planes(view_planes, &mut clipped_v1, &mut clipped_v2, &mut draw_line);
            if !draw_line {return;}

            clipped_v1 = project(&clipped_v1); clipped_v2 = project(&clipped_v2); // convert to projected space
            clip_against_planes(proj_planes, &mut clipped_v1, &mut clipped_v2, &mut draw_line);
            if draw_line {draw::draw_line_2d(&clipped_v1, &clipped_v2, res, scale, self.color, fb);}
        };


        
        for f in &self.faces {
            let n:usize = f.p.len();
            
            for i in 0..n{
                //* Clip */
                let v1:Vec3D = points[ f.p[i] ] ; // project( &points[ f.p[i] ] ); 
                let v2:Vec3D = points[ f.p[(i+1)%n] ]; //  project( &points[ f.p[(i+1)%n] ] );

                draw_line(&v1, &v2, fb, &view_planes, &proj_planes);
            }

        }


        if draw_norms {
            for f in &self.faces   { 
                let avr:Vec3D = f.p.iter().fold(Vec3D::zero(), |acc,curr| {
                    acc.add_vec( &self.points[*curr]  )
                }).div_scalar( f.p.len() as f64);


                let n = &avr.add_vec(&f.normal.normalized().mult_scalar(0.01)); 
                let avr = avr;
                let diff = n.sub_vec(&avr);

                // Draw normal from the centre of the face cos why not, will also need to project the norms I believe
                let v2 = avr.add_vec(&diff.mult_scalar(norm_size) );
                draw_line(&avr, &v2, fb, &view_planes, &proj_planes);
                //-------------------------------------
                // let n = project(&avr.add_vec(&f.normal.normalized().mult_scalar(0.01))); 
                // let avr = project(&avr);
                // let diff = n.sub_vec(&avr);
                //-------------------------------------
                // Set-size normal approch
                // let n = project(&avr.add_vec(&f.normal)); 
                // let avr = project(&avr);
                // let diff = n.sub_vec(&avr).normalized();
                //-------------------------------------
            }

            // let centre_proj = project(&self.centre);
            //* Draw the centre point */
            let mut centre_proj = &self.centre;
            let mut centre_proj_valid = true;
            'a: for plane in &view_planes {
                let d = dist(&centre_proj, &plane.0, &plane.1);
                if d < 0.0 {centre_proj_valid = false; break 'a;}
            }
            if centre_proj_valid {
                let temp = project(&centre_proj); centre_proj = &temp;
                'b: for plane in  &proj_planes {
                    let d = dist(&centre_proj, &plane.0, &plane.1);
                    if d < 0.0 {centre_proj_valid = false; break 'b;}
                }
                if centre_proj_valid {draw::draw_square(centre_proj.x, centre_proj.y, 1, self.color, fb);}
            }
        }
    }



    /// Uses SAT algorithm to check if the 2 objects intersect with each
    pub fn intersect_sat(&self, other: &CollisionBox) -> bool {
        //? 1. For each face normal, project the points of the other box onto it.
        let test_face_norms = |faces: &Vec<Face>| -> bool {
            let mut overlaps = true;
            'a: for f1 in faces {
                let f1_n = &f1.normal;

                let mut min_shadow2:f64 = f64::MAX; let mut max_shadow2:f64 = f64::MIN;

                for v2 in &other.points {
                    let d = f1_n.dot(v2);
                    min_shadow2 = min_shadow2.min(d);
                    max_shadow2 = max_shadow2.max(d);
                }
                //---------------
                let mut min_shadow1:f64 = f64::MAX; let mut max_shadow1:f64 = f64::MIN;

                for v1 in &self.points {
                    let d = f1_n.dot(v1);
                    min_shadow1 = min_shadow1.min(d);    
                    max_shadow1 = max_shadow1.max(d);
                }
                let intersects:bool = !( max_shadow1 < min_shadow2 || max_shadow2 < min_shadow1 );
                if !intersects {overlaps = false; break 'a;} // found a plane where there is a gap.
            }
            overlaps
        };

        // Find min / max shadows using faces of object 1
        if !test_face_norms(&self.faces) {return false};
        // Repeat but for object2 faces. We can use a pointer to simplify it
        if !test_face_norms(&other.faces) {return false};

        //? 2. The friggin cross product case thing
        // First, we can precalculate all the cross products to test
        let cross_prods= self.edges.iter().zip(&other.edges)
            .map(|(edge_a, edge_b)|  edge_a.cross(edge_b)  );


        for cp in cross_prods {
            let mut min_shadow2 = f64::MAX; let mut max_shadow2 = f64::MIN;

            for v2 in &other.points {
                let d = cp.dot(v2);
                min_shadow2 = min_shadow2.min(d); max_shadow2 = max_shadow2.max(d);
            }
            
            let mut min_shadow1 = f64::MAX; let mut max_shadow1 = f64::MIN;
            
            for v1 in &self.points {
                let d = cp.dot(v1);
                min_shadow1 = min_shadow1.min(d); max_shadow1 = max_shadow1.max(d);
            }
            let intersects:bool = !(max_shadow1 < min_shadow2 || max_shadow2 < min_shadow1);
            if !intersects {return false;} // found a plane where there is a gap
        }

        // Intersects on all axis, so they must be intersecting
        return true;
        // for p in &self.faces.edges {
            // we enforced the winding in the faces so we use that I guess. - BAD IDEA, some faces reuse vectors, so it's inneficient.
            // This is actually kinda awkward in the current setup.

            // IDEA1 - try and deal with it anyways: Eg if we see 5,4 then ignore subsequent 4,5 accesses. Maybe not too too bad with a set.
            // IDEA2 - Fucking restructure the thing I guess.

            //? 2.1 For each dir vec in cube a, do the cross product against each dir vec in cube b & project points.
            // let edge_a: Vec<&Vec3D> = p.edges.iter().map(|e| &self.edges[*e]).collect();
        // }
    }

    /// Uses SAT algorithm to collide and resolve the collision statically.
    /// 
    /// The 'other' collisionBox acts as the static object and will not be moved
    pub fn static_sat(&mut self, other: &CollisionBox) -> Option<Vec3D> {
        //* Keep track of the projection with the MINIMUM overlap, as that is what we'll use */
        let mut min_overlap: f64 = f64::INFINITY;
        let mut min_overlap_dir = Vec3D::zero();

        //* Get the centre -> centre direction in order to allign the reaction correctly */
        let centre_centre_dir = self.centre.sub_vec(&other.centre);

        //? 1. For each face normal, project the points of the other box onto it.
        let test_face_norms = |faces: &Vec<Face>, min_overlap: &mut f64, min_overlap_dir: &mut Vec3D| -> bool {
            let mut overlaps = true;
            'a: for f1 in faces {
                let f1_n = &f1.normal;

                let mut min_shadow2:f64 = f64::MAX; let mut max_shadow2:f64 = f64::MIN;

                for v2 in &other.points {
                    let d = f1_n.dot(v2);
                    min_shadow2 = min_shadow2.min(d);
                    max_shadow2 = max_shadow2.max(d);
                }
                //---------------
                let mut min_shadow1:f64 = f64::MAX; let mut max_shadow1:f64 = f64::MIN;

                for v1 in &self.points {
                    let d = f1_n.dot(v1);
                    min_shadow1 = min_shadow1.min(d);    
                    max_shadow1 = max_shadow1.max(d);
                }

                //* Calculate minimum overlap */
                let current_overlap = max_shadow1.min(max_shadow2) - min_shadow1.max(min_shadow2);
                if current_overlap < *min_overlap {
                    *min_overlap = current_overlap;
                    *min_overlap_dir = f1_n.clone();
                }
                //* ------------------------ */


                let intersects:bool = !( max_shadow1 < min_shadow2 || max_shadow2 < min_shadow1 );
                if !intersects {overlaps = false; break 'a;} // found a plane where there is a gap.
            }
            overlaps
        };

        // Find min / max shadows using faces of object 1
        if !test_face_norms(&self.faces, &mut min_overlap, &mut min_overlap_dir) {return None};
        // Repeat but for object2 faces. We can use a pointer to simplify it
        if !test_face_norms(&other.faces, &mut min_overlap, &mut min_overlap_dir) {return None};
        // println!("min_overlap= {}", min_overlap);

        //? 2. The friggin cross product case thing
        // println!("CROSS PRODUCTS:");
        let mut cross_prods : Vec<Vec3D> = Vec::with_capacity( self.edges.len() * other.edges.len());
        for edge1 in &self.edges {
            for edge2 in &other.edges {
                let hamming_dist = (edge1.x-edge2.x) + (edge1.y-edge2.y) + (edge1.z-edge2.z);

                if hamming_dist > 0.000_1 {
                    // println!("({:.3},{:.3},{:.3}) x ({:.3},{:.3},{:.3}) =  ({:.3},{:.3},{:.3})", edge1.x,edge1.y,edge1.z, edge2.x,edge2.y,edge2.z, edge1.cross(edge2).x, edge1.cross(edge2).y,edge1.cross(edge2).z);
                    cross_prods.push( edge1.cross(edge2).normalized());
                }
            }
        }




        for cp in cross_prods {
            let mut min_shadow2 = f64::MAX; let mut max_shadow2 = f64::MIN;

            for v2 in &other.points {
                let d = cp.dot(v2);
                min_shadow2 = min_shadow2.min(d); max_shadow2 = max_shadow2.max(d);
            }
            
            let mut min_shadow1 = f64::MAX; let mut max_shadow1 = f64::MIN;
            
            for v1 in &self.points {
                let d = cp.dot(v1);
                min_shadow1 = min_shadow1.min(d); max_shadow1 = max_shadow1.max(d);
            }
            //* Caluclate minimum overlap */
            let current_overlap = max_shadow1.min(max_shadow2) - min_shadow1.max(min_shadow2);
            if current_overlap < min_overlap {
                min_overlap =  current_overlap;
                min_overlap_dir = cp;
                // println!("b, {}, cp =({},{},{})", min_overlap, cp.x,cp.y,cp.z );
                // println!("THIS POINTS= {:?}", self.points);
                // println!("##########################################################");
            }

            //* ------------------------- */
            let intersects:bool = !(max_shadow1 < min_shadow2 || max_shadow2 < min_shadow1);
            if !intersects {return None;} // found a plane where there is a gap
        }

        // It must face the same direction - If dot -ve then they face opposite directions.
        let similarity = min_overlap_dir.dot(&centre_centre_dir).signum();

        // Respond to collision
        let offset = min_overlap_dir.mult_scalar( similarity *  min_overlap);
        self.transform(&Mat4x4::translation_matrix(offset.x, offset.y, offset.z));

        // Intersects on all axis, so they must be intersecting
        return Some(offset)
    }






    pub fn transform(&mut self, mat: &Mat4x4) {
        // 1 a) Transform the points
        for p in &mut self.points {
            *p = vec_multiply_mat(p, mat);
        }
        // 1 b) Translate centre
        self.centre = vec_multiply_mat(&self.centre, mat);

        // 2) Recalculate normals & direction vectors
        let face_points:Vec<Vec<usize>> = self.faces.iter()
            .map(|f| f.p.clone() ).collect();

        let (edges,faces) = Self::recalculate_normals(&self.points, face_points);



        self.faces = faces;
        self.edges = edges;
    }


    /// Recalculates the normals and vectors, eg after a transform to the points.
    fn recalculate_normals(points: &Vec<Vec3D>, faces: Vec<Vec<usize>>) -> (Vec<Vec3D>, Vec<Face> ) {
        assert!( faces.iter().all(|x| x.len() >= 3));
        let mut box_faces:Vec<Face> = Vec::with_capacity(faces.len());
        //------------ Storing edges: It's more optimal to store identical dir vecs ------------------
        // Assumes the given vectors are normalized. Returns index if it is found, and if not it adds it and returns new index.
        let add_if_unique = |stored_v: &mut Vec<Vec3D>, v: &Vec3D| -> usize{
            let n = stored_v.len();
            for i in 0..n {   if (stored_v[i].dot(v).abs()-1.0).abs() < 1e-5 {return i;} } //  180 deg or 0 deg  (+-1 )
            stored_v.push(*v); // It isn't in stored_v, so we must add it ourselves
            return n;
        };
        let mut unique_edges: Vec<Vec3D> = vec![]; // Contains direction vectors
        //-------------------------------------------------------------------------------------------
        for f in faces {
            // let v1 = points[f[2]].sub_vec(&points[f[1]]).normalized(); // Since it is planar, it shouldn't matter right?  ;      // let v2 = points[f[1]].sub_vec(&points[f[0]]).normalized();             ;  // let norm = v1.cross(&v2).normalized(); // |axb| = |a||b|sin t = sin t,  ie not guaranteed to be 1 here so need to normalize again.
            // Add edges
            let mut edges:Vec<Vec3D> = vec![];
            let mut edges_idx: Vec<usize> = vec![];
            
            let nf = f.len();
            for i in 0..=nf {
                let edge = points[f[(i+1)%nf]].sub_vec(&points[f[i%nf]]).normalized();
                let i = add_if_unique(&mut unique_edges, &edge);
                edges.push(edge);    edges_idx.push(i);
            }
            let norm  = edges[1].cross(&edges[0]).normalized();
            let f = Face{p: f, edges: edges_idx, normal: norm };
            box_faces.push(f);
        }
        return (unique_edges, box_faces)
    }

}





/// A function only really used for debugging.
pub fn get_norms(col_box: &CollisionBox) -> Vec<[(f64,f64,f64);2]> {
    // Get averages 
    let mut averages = vec![];

    for f in &col_box.faces   { 
        let avr:Vec3D = f.p.iter().fold(Vec3D::zero(), |acc,curr| {
            acc.add_vec( &col_box.points[*curr]  )
        }).div_scalar( f.p.len() as f64);
        // let n = avr.add_vec(&f.normal);
        let n = f.normal;
        // let n = project(&avr.add_vec(&f.normal)); 
        // let avr = project(&avr);
        averages.push( [ (avr.x,avr.y,avr.z), (n.x,n.y,n.z)]);       
    }
    averages
} 


















        // let draw_line = |v1: &Vec3D, v2: &Vec3D, fb: &mut FrameBuf| {
        //     let mut draw_line = true; // We can draw it if it is on the correct half space of every plane
        //     let mut clipped_v1 = v1.clone(); let mut clipped_v2 = v2.clone();

        //     'a: for plane in &planes {
        //         // Clip the line against each plane
        //         let d1 =  dist(&clipped_v1, &plane.0, &plane.1);
        //         let d2 =  dist(&clipped_v2, &plane.0, &plane.1);

        //         let (furthest, other) = if d1 > d2 {(clipped_v1.clone(), clipped_v2.clone())} else {(clipped_v2.clone(), clipped_v1.clone())};
        //         let vec_a = furthest;
        //         let vec_b = other.sub_vec(&vec_a);

        //         if d1 > 0.0 && d2 > 0.0 { // both points are in the half space - then we can just draw it
        //             // draw::draw_line_2d(&project(&v1), &project(&v2), res, scale,color,  fb);
        //             draw_line = true;
                     
        //         }else if d1> 0.0 || d2 > 0.0 { // at least one point is in the half spaced
        //             if let Some( (intersect,t)) = vec_intersect_plane(&plane.0, &plane.1, &vec_a, &vec_b) {
        //                 // Draw from intersect to furthest point in direction the plane normal is facing
        //                 if t >= 0.0 { // intersect is in front of the plane
        //                     if t <= 1.0  {
        //                         // draw::draw_line_2d(&project(&intersect), &project(furthest), res, scale, color, fb);
        //                         clipped_v1 = intersect; clipped_v2 = furthest;
        //                     }else {
        //                         // draw::draw_line_2d(&project(&v1), &project(&v2), res, scale,color,  fb);
        //                         draw_line = true;
        //                     }
        //                 }else {
        //                     draw_line = false;
        //                 }
        //             }else{ draw_line = false}
        //         }else {draw_line = false; break 'a;}
        //     }

            
        //     if draw_line {
        //         for plane in 
        //         draw::draw_line_2d(&project(&clipped_v1), &project(&clipped_v2), res, scale, color, fb);
        //     }
                


        //         // if d1 > 0.0 && d2 > 0.0 {
        //         //     if d1> 0.0 || d2 > 0.0 { // at least one point is in the half spaced
        //         //         if let Some( (intersect,t)) = vec_intersect_plane(&plane.0, &plane.1, &vec_a, &vec_b) {
        //         //             // Draw from intersect to furthest point in direction the plane normal is facing
        //         //             if t >= 0.0 && t <= 1.0 { // intersect is in front of the plane
        //         //                 clipped_v1 = intersect; clipped_v2 = *furthest.clone();
        //         //             } 
        //         //         }
        //         //     }
        //         // }else { // The line is fully behind one of the planes.
        //         //     draw_line = false; 
        //         //     break 'a;
        //         // }
            

        //     // if draw_line {
        //     //     draw::draw_line_2d(&project(&clipped_v1), &project(&clipped_v2), res, scale, color, fb);
        //     // }

        //     // if d1 > 0.0 && d2 > 0.0 { // both points are in the half space - then we can just draw it
        //     //     draw::draw_line_2d(&project(&v1), &project(&v2), res, scale,color,  fb);
        //     // }else if d1> 0.0 || d2 > 0.0 { // at least one point is in the half spaced
        //     //     if let Some( (intersect,t)) = vec_intersect_plane(&plane.0, &plane.1, &vec_a, &vec_b) {
        //     //         // Draw from intersect to furthest point in direction the plane normal is facing
        //     //         if t >= 0.0 { // intersect is in front of the plane
        //     //             if t <= 1.0  {
        //     //                 draw::draw_line_2d(&project(&intersect), &project(furthest), res, scale, color, fb);
        //     //             }else {
        //     //                 draw::draw_line_2d(&project(&v1), &project(&v2), res, scale,color,  fb);
        //     //             }
        //     //         } 
        //     //     }
                
        // };
        // // let draw_line = |v1: &Vec3D, v2: &Vec3D, fb: &mut FrameBuf| {
        // //     'a: for plane in &planes {
        // //         let d1 =  dist(&v1, &plane.0, &plane.1);
        // //         let d2 =  dist(&v2, &plane.0, &plane.1);

        // //         let (furthest, other) = if d1 > d2 {(&v1, &v2)} else {(&v2, &v1)};
        // //         let vec_a = furthest;
        // //         let vec_b = other.sub_vec(&vec_a);

        // //         if d1 > 0.0 && d2 > 0.0 { // both points are in the half space - then we can just draw it
        // //             draw::draw_line_2d(&project(&v1), &project(&v2), res, scale,color,  fb);
        // //         }else if d1> 0.0 || d2 > 0.0 { // at least one point is in the half spaced
        // //             if let Some( (intersect,t)) = vec_intersect_plane(&plane.0, &plane.1, &vec_a, &vec_b) {
        // //                 // Draw from intersect to furthest point in direction the plane normal is facing
        // //                 if t >= 0.0 { // intersect is in front of the plane
        // //                     if t <= 1.0  {
        // //                         draw::draw_line_2d(&project(&intersect), &project(furthest), res, scale, color, fb);
        // //                     }else {
        // //                         draw::draw_line_2d(&project(&v1), &project(&v2), res, scale,color,  fb);
        // //                     }
        // //                 } 
        // //             }
        // //         }
        // //     }

        // // };
