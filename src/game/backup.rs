
pub struct CollisionBox {
    pub points: Vec<Vec3D>,
    pub faces: Vec<Face>,
}
impl CollisionBox {
    pub fn new(points: Vec<Vec3D>, faces: Vec<Vec<usize>>) -> Self {
        assert!( faces.iter().all(|x| x.len() >= 3));
        let mut box_faces:Vec<Face> = Vec::with_capacity(faces.len());

        for f in faces {
            let v1 = points[f[2]].sub_vec( &points[f[1]]);
            let v2 = points[f[1]].sub_vec(&points[f[0]]);
            let norm = v1.cross(&v2).normalized();
            let f = Face{p: f, normal: norm };
            box_faces.push(f);
        }

        CollisionBox { points, faces: box_faces }
    }


    /// Projects and draws the collision box
    pub fn draw(&self, mat_proj: &Mat4x4, res:f64, scale:u32, color:[u8;4], fb: &mut FrameBuf) {
        let norm_size = 10.0;
        // Project the points & norms
        let mut points = Vec::new();
        let mut norms:Vec<[Vec3D;2]> = Vec::new();

        let project = |p: &Vec3D| -> Vec3D {
            let p = Vec3D::new(p.x, p.y, p.z+3.0);
            let mut p = vec_multiply_mat(&p, &mat_proj);
            // Shift into view
            p.x = (p.x + 1.0) * 0.5 * F64_SCREEN_WIDTH;
            p.y = (p.y + 1.0) * 0.5 * F64_SCREEN_HEIGHT;
            p
        };

        for p in &self.points { let p = project(&p); points.push(p); }
        for f in &self.faces   { 
            let avr:Vec3D = f.p.iter().fold(Vec3D::zero(), |acc,curr| {
                acc.add_vec( &self.points[*curr]  )
            }).div_scalar( f.p.len() as f64);
            let n = project(&avr.add_vec(&f.normal)); 
            let avr = project(&avr);

            let diff = n.sub_vec(&avr).normalized();
            norms.push( [avr, diff])
        }

        for f in &self.faces {
            let n:usize = f.p.len();
            
            for i in 0..n{
                let v1:Vec3D = points[ f.p[i]       ]; 
                let v2:Vec3D = points[ f.p[(i+1)%n] ];

                draw::draw_square(v1.x, v1.y, 5, color, fb);
                draw::draw_line_2d(&v1, &v2, res, scale, color, fb);
            }

        }
        // Draw normal from the centre of the face cos why not, will also need to project the norms I believe
        for [avr, diff] in &norms {
            let v2 = avr.add_vec(&diff.mult_scalar(norm_size) );
            draw::draw_line_2d(&avr, &v2, res, scale, color, fb);
            draw::draw_square(v2.x, v2.y, 1, crate::_white, fb);
        }
    }


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
        for p in &self.faces {
            // we enforced the winding in the faces so we use that I guess. - BAD IDEA, some faces reuse vectors, so it's inneficient.
            // This is actually kinda awkward in the current setup.

            // IDEA1 - try and deal with it anyways: Eg if we see 5,4 then ignore subsequent 4,5 accesses. Maybe not too too bad with a set.
            // IDEA2 - Fucking restructure the thing I guess.

        }





        false
    }
}
















//-----------------------------------
// pub struct Face {
//     pub p: Vec<usize>,
//     pub edges: Vec<usize>,
//     pub normal: Vec3D,
// }

// pub struct CollisionBox {
//     pub points: Vec<Vec3D>,
//     edges: Vec<Vec3D>,
//     pub faces: Vec<Face>,
// }
// impl CollisionBox {
    // pub fn new(points: Vec<Vec3D>, faces: Vec<Vec<usize>>) -> Self {
    //     assert!( faces.iter().all(|x| x.len() >= 3));
    //     let mut box_faces:Vec<Face> = Vec::with_capacity(faces.len());

    //     //------------ Storing edges: It's more optimal to store identical dir vecs ------------------
    //     // Assumes the given vectors are normalized. Returns index if it is found, and if not it adds it and returns new index.
    //     let add_if_unique = |stored_v: &mut Vec<Vec3D>, v: &Vec3D| -> usize{
    //         let n = stored_v.len();
    //         for i in 0..n {
    //             if (stored_v[i].dot(v) - 1.0).abs() < 1e-5 {return i;}
    //         }
    //         // It isn't in stored_v, so we must add it ourselves
    //         stored_v.push(*v);
    //         return n;
    //     };
    //     let mut unique_edges: Vec<Vec3D> = vec![]; // Contains direction vectors
    //     //-------------------------------------------------------------------------------------------
    //     for f in faces {
    //         let v1 = points[f[2]].sub_vec(&points[f[1]]).normalized(); // Since it is planar, it shouldn't matter right?
    //         let v2 = points[f[1]].sub_vec(&points[f[0]]).normalized();
    //         let norm = v1.cross(&v2).normalized(); // |axb| = |a||b|sin t = sin t,  ie not guaranteed to be 1 here so need to normalize again.
            
    //         // Add edges
    //         // let mut edges:Vec<Vec3D> = vec![];
    //         // let mut edges_idx: Vec<usize> = vec![];
            
    //         // let nf = f.len();
    //         // for i in 0..=nf {
    //         //     let edge = points[f[(i+1)%nf]].sub_vec(&points[f[i%nf]]).normalized();
    //         //     let i = add_if_unique(&mut unique_edges, &edge);
    //         //     edges.push(edge);    edges_idx.push(i);
    //         // }
    //         // let norm  = edges[1].cross(&edges[0]).normalized();

    //         let f = Face{p: f, edges: edges_idx, normal: norm };
    //         // if is_unique(&unique_edges, &v2) {unique_edges.push(v2)}

    //         box_faces.push(f);
    //     }

    //     CollisionBox { points, edges: unique_edges, faces: box_faces }
    // }


//     /// Projects and draws the collision box
//     pub fn draw(&self, mat_proj: &Mat4x4, res:f64, scale:u32, color:[u8;4], fb: &mut FrameBuf) {
//         let norm_size = 10.0;
//         // Project the points & norms
//         let mut points = Vec::new();
//         let mut norms:Vec<[Vec3D;2]> = Vec::new();

//         let project = |p: &Vec3D| -> Vec3D {
//             let p = Vec3D::new(p.x, p.y, p.z+3.0);
//             let mut p = vec_multiply_mat(&p, &mat_proj);
//             // Shift into view
//             p.x = (p.x + 1.0) * 0.5 * F64_SCREEN_WIDTH;
//             p.y = (p.y + 1.0) * 0.5 * F64_SCREEN_HEIGHT;
//             p
//         };

//         for p in &self.points { let p = project(&p); points.push(p); }
//         for f in &self.faces   { 
//             let avr:Vec3D = f.p.iter().fold(Vec3D::zero(), |acc,curr| {
//                 acc.add_vec( &self.points[*curr]  )
//             }).div_scalar( f.p.len() as f64);
//             let n = project(&avr.add_vec(&f.normal)); 
//             let avr = project(&avr);

//             let diff = n.sub_vec(&avr).normalized();
//             norms.push( [avr, diff])
//         }

//         for f in &self.faces {
//             let n:usize = f.p.len();
            
//             for i in 0..n{
//                 let v1:Vec3D = points[ f.p[i]       ]; 
//                 let v2:Vec3D = points[ f.p[(i+1)%n] ];

//                 draw::draw_square(v1.x, v1.y, 5, color, fb);
//                 draw::draw_line_2d(&v1, &v2, res, scale, color, fb);
//             }

//         }
//         // Draw normal from the centre of the face cos why not, will also need to project the norms I believe
//         for [avr, diff] in &norms {
//             let v2 = avr.add_vec(&diff.mult_scalar(norm_size) );
//             draw::draw_line_2d(&avr, &v2, res, scale, color, fb);
//             draw::draw_square(v2.x, v2.y, 1, crate::_white, fb);
//         }
//     }

//     // TODO: If a normal is parallel to another one, we don't need project with that again it'll just waste time (especially important with cubes    // TODO: If a normal is parallel, we don't need to do that again it'll just waste time, saves lots of time)
//     pub fn intersect_sat(&self, other: &CollisionBox) -> bool {
//         //? 1. For each face normal, project the points of the other box onto it.
//         let test_face_norms = |faces: &Vec<Face>| -> bool {
//             let mut overlaps = true;
//             'a: for f1 in faces {
//                 let f1_n = &f1.normal;

//                 let mut min_shadow2:f64 = f64::MAX; let mut max_shadow2:f64 = f64::MIN;

//                 for v2 in &other.points {
//                     let d = f1_n.dot(v2);
//                     min_shadow2 = min_shadow2.min(d);
//                     max_shadow2 = max_shadow2.max(d);
//                 }
//                 //---------------
//                 let mut min_shadow1:f64 = f64::MAX; let mut max_shadow1:f64 = f64::MIN;

//                 for v1 in &self.points {
//                     let d = f1_n.dot(v1);
//                     min_shadow1 = min_shadow1.min(d);    
//                     max_shadow1 = max_shadow1.max(d);
//                 }
//                 let intersects:bool = !( max_shadow1 < min_shadow2 || max_shadow2 < min_shadow1 );
//                 if !intersects {overlaps = false; break 'a;} // found a plane where there is a gap.
//             }
//             overlaps
//         };

//         // Find min / max shadows using faces of object 1
//         if !test_face_norms(&self.faces) {return false};
//         // Repeat but for object2 faces. We can use a pointer to simplify it
//         if !test_face_norms(&other.faces) {return false};

//         //? 2. The friggin cross product case thing
//         for p in &self.faces {
//             // we enforced the winding in the faces so we use that I guess. - BAD IDEA, some faces reuse vectors, so it's inneficient.
//             // This is actually kinda awkward in the current setup.
//             // IDEA1 - try and deal with it anyways: Eg if we see 5,4 then ignore subsequent 4,5 accesses. Maybe not too too bad with a set.
//             // IDEA2 - Fucking restructure the thing I guess.


//             //? 2.1 Task 1 is to find the edges for each cuboid.
//             // let a = p.p';
//         }





//         false
//     }
// }









































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
            NormalImplicit(v) => v.len(),
            NormalExplicit(v) => v.len(),
        }
    }

    pub fn into_iter(&self) ->  IntoIter< Either<&Vec<usize>, &(Vec<usize>, Vec3D) > > {
        let mut buf = vec![];
        match self {
            NormalImplicit(v) => {
                for e in v {
                    buf.push(Either::Left(e));
                }
            }
            NormalExplicit(v) => {
                for e in v {
                    buf.push(Either::Right(e));
                }
            }
        }

        buf.into_iter()
    }


}
// impl Iterator for FaceInput {
//     type Item = Either<Vec<Vec<usize>>, Vec<(Vec<usize>, Vec3D)> >;

//     fn next(&mut self) -> Option<Self::Item> {



//     }
// }


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
    pub edges: Vec<Vec3D>,
    pub faces: Vec<Face>,
}
impl CollisionBox {
    pub fn new(points: Vec<Vec3D>, faces: FaceInput) -> Self {
        // let faces = match faces {
        //     NormalImplicit(f) => f,
        //     _ => vec![],
        // };

        let (unique_edges, box_faces) = Self::recalculate_normals(&points, faces);
        println!("n={}, unique_edges = {:?}\n\n", unique_edges.len(), unique_edges);

        CollisionBox { points, edges: unique_edges, faces: box_faces }
    }
    


    /// Projects and draws the collision box
    pub fn draw(&self, mat_proj: &Mat4x4, res:f64, scale:u32, color:[u8;4], fb: &mut FrameBuf) {
        let norm_size = 10.0;
        // Project the points & norms
        let mut points = Vec::new();
        let mut norms:Vec<[Vec3D;2]> = Vec::new();

        let project = |p: &Vec3D| -> Vec3D {
            let p = Vec3D::new(p.x, p.y, p.z+3.0);
            let mut p = vec_multiply_mat(&p, &mat_proj);
            // Shift into view
            p.x = (p.x + 1.0) * 0.5 * F64_SCREEN_WIDTH;
            p.y = (p.y + 1.0) * 0.5 * F64_SCREEN_HEIGHT;
            p
        };


        for p in &self.points { let p = project(&p); points.push(p); }
        for f in &self.faces   { 
            let avr:Vec3D = f.p.iter().fold(Vec3D::zero(), |acc,curr| {
                acc.add_vec( &self.points[*curr]  )
            }).div_scalar( f.p.len() as f64);
            let n = project(&avr.add_vec(&f.normal)); 
            let avr = project(&avr);

            let diff = n.sub_vec(&avr).normalized();



            norms.push( [avr, diff])
        }

        for f in &self.faces {
            let n:usize = f.p.len();
            
            for i in 0..n{
                let v1:Vec3D = points[ f.p[i]       ]; 
                let v2:Vec3D = points[ f.p[(i+1)%n] ];

                draw::draw_square(v1.x, v1.y, 5, color, fb);
                draw::draw_line_2d(&v1, &v2, res, scale, color, fb);
            }

        }
        // Draw normal from the centre of the face cos why not, will also need to project the norms I believe
        for [avr, diff] in &norms {
            let v2 = avr.add_vec(&diff.mult_scalar(norm_size) );
            draw::draw_line_2d(&avr, &v2, res, scale, color, fb);
            draw::draw_square(v2.x, v2.y, 1, crate::_white, fb);
        }
    }


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


















    pub fn transform(&mut self, mat: &Mat4x4) {
        // 1) Transform the points
        for p in &mut self.points {
            *p = vec_multiply_mat(p, mat);
        }
        // 2) Recalculate normals & direction vectors
        let face_points:Vec<Vec<usize>> = self.faces.iter()
            .map(|f| f.p.clone() ).collect();

        let (edges,faces) = Self::recalculate_normals(&self.points, FaceInput::NormalImplicit(face_points) );
        self.faces = faces;
        self.edges = edges;
    }


    /// Recalculates the normals and vectors, eg after a transform to the points.
    fn recalculate_normals(points: &Vec<Vec3D>, faces: FaceInput ) -> (Vec<Vec3D>, Vec<Face> ) {
        match &faces {
            FaceInput::NormalExplicit(v) =>  assert!( v.iter().all(|x| x.0.len() >= 3)),       FaceInput::NormalImplicit(v) =>  assert!( v.iter().all(|x| x.len() >= 3)),
        }
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
        for f_either in faces.into_iter() {
            let (f, explicit_norm) = match f_either {
                Either::Left(v) => (v, None),
                Either::Right(v) => (&v.0, Some(v.1.clone()) ),
            };

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

            // If the normal is not given, calculate it ourselves with cross product.
            let norm = explicit_norm.unwrap_or_else(|| {
                edges[1].cross(&edges[0]).normalized()
            });


            let f = Face{p: f.to_vec(), edges: edges_idx, normal: norm };
            box_faces.push(f);
        }
        return (unique_edges, box_faces)
    }

}