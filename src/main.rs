#![allow(non_snake_case)]
//TODO: Lots of sorting out the value types
//TODO: Also, pre-multiplying a world matrix is prolly more efficient

//? THe for loops could be slightly more inneficient compared to just doing the thing for each vec.
extern crate piston;
extern crate piston_window;
extern crate image;

use std::{collections::{HashMap, VecDeque}, io::empty};

use bml_ml::{NeuralNetwork, enums::ActFn, optimizer::Adam, rl::qnn::{Policy, QNN}};
use piston_window::*;
use image::{Frame, ImageBuffer, io::LimitSupport};

mod draw;
mod lib;
mod game;
mod normal_stuff;

use crate::{Color, FrameBuf, draw::{_blue, _green, _grey, _red, _white, draw_line_2d, draw_rec, draw_triangle, fill_triangle_2d, fill_triangle_texture}, game::{game_obj::{Axis, CollisionBox, FaceInput, GameObj, GameObjCollection, GameObjType, get_norms}, player::{Agent, Inputs, Player}, update::update}, lib::*};

use normal_stuff::{self as funky, triangle_clip_plane, vec_intersect_plane};


////! Customisables
const ROT_SPEED : f64 = 0.05; // 0.05 or 0.02
const BACKROUND_COL : [f32 ; 4]= [0.0,0.0,0.0,0.0];  // Grey : [0.1, 0.1, 0.1, 0.1]      Black : [0.0,0.0,0.0,0.0]

////!
pub const SCREEN_WIDTH  : u32 = 1040; //1480; //1040;
pub const SCREEN_HEIGHT : u32=  540; // 720; // 540;
const F64_SCREEN_WIDTH  : f64 = SCREEN_WIDTH  as f64;
const F64_SCREEN_HEIGHT : f64 = SCREEN_HEIGHT as f64;

const PI : f64 = 3.14159; // or std::f64::consts::PI;
//Mat proj values
const FOV : f64= 90.0;
const Z_FAR : f64 = 1000.0;
const Z_NEAR : f64 = 0.1;
const F_ASPECT_RATIO : f64 = F64_SCREEN_HEIGHT / F64_SCREEN_WIDTH;

const CLIP_COLOR_IT:bool = false;
const USE_PAINTERS_ALGORITHM: bool = false;

const CAMERA_MODE: bool = true; // mode 0 = camera is like a player, mode 1 = camera is like a camera


//----------- Constants -------------//
pub const GRAVITY : f64 = 0.01; pub const TERMINAL_VELOCITY: f64 =1.0;



fn main() {
    


    let cube1_points:Vec<Vec3D> = vec![
        Vec3D::new(-1.,-1.,-1.),
        Vec3D::new(1.,-1.,-1.),
        Vec3D::new(1.,1.,-1.),
        Vec3D::new(-1.,1.,-1.),
        Vec3D::new(-1.,-1.,1.),
        Vec3D::new(1.,-1.,1.),
        Vec3D::new(1.,1.,1.),
        Vec3D::new(-1.,1.,1.),
    ];

    // Should be 6 faces (user needs to enforce winding order)
    let cube_faces:Vec<(Vec<usize>, Vec3D)> = vec![
        (vec![0,1,5,4], Vec3D::new(0.,-1.,0.)), // left face 
        (vec![4,5,6,7], Vec3D::new(0.,0.,1.)),// top face 
        (vec![6,7,3,2], Vec3D::new(0.,1.,0.)), // right face 
        (vec![0,1,2,3], Vec3D::new(0.,0.,-1.)),// base face 

        (vec![1,2,6,5], Vec3D::new(1.,0.,0.) ), // front face 
        (vec![0,3,7,4], Vec3D::new(-1.,0.,0.)), // back face
    ];

    let camera_points:Vec<Vec3D> = cube1_points.iter()
        .map(|p| {
            let  q = p.add_vec(&Vec3D::new(0.,-3.,-10.));
            q
        }).collect();

    let platform_points: Vec<Vec3D> = cube1_points.iter()
        .map(|p| {
            let mut q = p.add_vec(&Vec3D::new(0., 3.0, 0.0));
            if q.z > 0.0 { q.z *= 5.0;}
            q.x *= 5.0;
            q
        })
        .collect();

    let platform2_points: Vec<Vec3D> = cube1_points.iter()
        .map(|p| {
            let mut q = p.add_vec(&Vec3D::new(0., 3.0, 0.0));
            q.x += 2.5;
            q.z *= 5.0;

            q.z += 10.0;
            q
        }).collect();
    let platform3_points: Vec<Vec3D> = platform2_points.iter()
        .map(|p|  {
            let mut q = p.clone();
            q.z -= 10.; q.z /= 5.0; q.z += 10.; 
            q.x -= 2.5; q.x *= 3.5;
            q.z += 6.0;
            // q.x -= 6.0;
            q
        }).collect();
    let platform4_points:Vec<Vec3D> = cube1_points.iter()
        .map(|p|  {
            let mut q = p.clone();
            q.z *= 10.;
            q.z += 25.0;
            q.y += 3.0;
            q.x -= 4.5;


            q
        }).collect();

    


    //---------------------------------------------------------------------------------
    println!("F_ASPECT_RATIO ={}",F_ASPECT_RATIO);




    //? Projection matrix
    let fov_sf : f64 = 1.0f64 / ( (FOV * 0.5f64) / 180.0 * PI ).tan();
    let mut mat_proj : Mat4x4 = Mat4x4::zero(); // [row][collumn] is the standard
    mat_proj.O[0][0] = F_ASPECT_RATIO * fov_sf;
    mat_proj.O[1][1] = fov_sf;
    mat_proj.O[2][2] = Z_FAR / (Z_FAR - Z_NEAR);
    mat_proj.O[3][2] = (-Z_FAR * Z_NEAR) / (Z_FAR - Z_NEAR);
    mat_proj.O[2][3] = 1.0f64;
    mat_proj.O[3][3] = 0.0f64;

    
    //---------------------- GAME OBJECTS -------------------
    let axis = Axis::new([Vec3D::new(1.,0.,0.), Vec3D::new(0.,-1.,0.), Vec3D::new(0.,0.,1.)], [(F64_SCREEN_WIDTH*0.9) as usize, (F64_SCREEN_HEIGHT*0.1) as usize]);

    let cube1 = CollisionBox::new(cube1_points, FaceInput::NormalExplicit(cube_faces.clone()), Some(_blue) );
    let camera_box = CollisionBox::new(camera_points,FaceInput::NormalExplicit(cube_faces.clone()), Some(_blue) );
    let platform = CollisionBox::new(platform_points,  FaceInput::NormalExplicit(cube_faces.clone()), None);
    let platform2 = CollisionBox::new(platform2_points, FaceInput::NormalExplicit(cube_faces.clone()),  None);
    let platform3 = CollisionBox::new(platform3_points, FaceInput::NormalExplicit(cube_faces.clone()), None);
    let platform4 = CollisionBox::new(platform4_points, FaceInput::NormalExplicit(cube_faces), None);
    println!("cube1 norms= {:?}\n", get_norms(&cube1));
    
    //? AI stuff
    let qnn = QNN::new(
        NeuralNetwork::xavier_he(&vec![6,256,200,4], vec![ActFn::ReLu,ActFn::ReLu,ActFn::Identity], false),
        32, Some(100_000), 0.003, 0.9, 
        Policy::EGreedy {eps: 0.7, eps_dec: 5e-5, eps_min: 0.001}
    );
    let optimizer = Adam::new(0.9, 0.99, &qnn.nn);
    let respawn_point = cube1.centre;

    //? File
    let empty_mesh = Mesh::new(vec![]);
    // let obj1 : Mesh     =  Mesh::load_from_obj(r"cuboid.obj").expect("ERR: Failed to load mesh from path 'cow.obj'");
    // let obj2 : Mesh = Mesh::load_from_obj("cuboid.obj").expect("ERR: Failed to load cuboid mesh");

    let player = Player::new(GameObj::new("camera", if CAMERA_MODE{camera_box} else {cube1.clone()}, empty_mesh.clone(), CAMERA_MODE==false) , 0.1, None); // let cube2:GameObj = GameObj::new("cube2", cube2);
    let agent = Agent::new(GameObj::new("agent", cube1, empty_mesh.clone(), true), qnn, optimizer,respawn_point);
    let platform=  GameObj::new("platform", platform, empty_mesh.clone(), true);
    let platform2 = GameObj::new("platform2", platform2,  empty_mesh.clone(), true);
    let platform3 = GameObj::new("platform3", platform3, empty_mesh.clone(), true);
    let platform4 = GameObj::new("platform4", platform4,empty_mesh, true);
    let mut game_objects = GameObjCollection::from_iter(
        vec![("platform", Box::new(platform) as Box<dyn GameObjType> ), ("platform2", Box::new(platform2) as Box<dyn GameObjType> ),  ("platform3", Box::new(platform3) as Box<dyn GameObjType>), ("platform4", Box::new(platform4) as Box<dyn GameObjType>),
        
        ("agent", Box::new(agent) as Box<dyn GameObjType>),
        ("camera", Box::new(player) as Box<dyn GameObjType> )].into_iter()
    );
    // Just basically do current setup but mesh has empty pixels in it
    //-------------------------------------------------------



    let mut window : PistonWindow = WindowSettings::new("3D simulation",[SCREEN_WIDTH, SCREEN_HEIGHT])
        .exit_on_esc(true)
        .resizable(true)
        .transparent(true)
        .build()
        .unwrap_or_else(|e| {panic!("Failed to build PistonWindow {}",e)});
    
    let mut frame_buf: FrameBuf = ImageBuffer::from_pixel( window.size().width as u32,  window.size().height as u32, image::Rgba(_grey));
    // Context for updating textures
    let mut tex_con = piston_window::TextureContext {  factory: window.factory.clone(),encoder: window.factory.create_command_buffer().into() };
    let mut tex = piston_window::Texture::from_image(&mut tex_con, &frame_buf, &TextureSettings::new()).expect("Failed to create texture");


    let mut w_is_pressed : bool = false;  let mut a_is_pressed : bool = false;    let mut i_is_pressed : bool = false;  let mut j_is_pressed : bool = false;  
    let mut s_is_pressed : bool = false;  let mut d_is_pressed : bool = false;    let mut k_is_pressed : bool = false;  let mut l_is_pressed : bool = false;  
    let mut space_is_pressed: bool = false;

    let UP = Vec3D::new(0.0,-1.0,0.0);
    while let Some(e) = window.next(){
        //* Input handling */
        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key{
                Key::W => w_is_pressed = true,
                Key::A => a_is_pressed = true,
                Key::S => s_is_pressed = true,
                Key::D => d_is_pressed = true,

                Key::I => i_is_pressed = true,
                Key::J => j_is_pressed = true,
                Key::K => k_is_pressed = true,
                Key::L => l_is_pressed = true,
                Key::Space => space_is_pressed = true,
                _ => println!("Unknown key pressed"),
            }
        }
        if let Some(Button::Keyboard(key)) =  e.release_args() {
            match key{
                Key::W => w_is_pressed = false,
                Key::A => a_is_pressed = false,
                Key::S => s_is_pressed = false,
                Key::D => d_is_pressed = false,

                Key::I => i_is_pressed = false,
                Key::J => j_is_pressed = false,
                Key::K => k_is_pressed = false,
                Key::L => l_is_pressed = false,
                Key::Space => space_is_pressed = false,
                _ => println!("Unknown key released"),
            }
        }


        window.draw_2d(&e, |c, g, device|{
            tex.update(&mut tex_con, &frame_buf).unwrap();
            tex_con.encoder.flush(device); // Flush context into GPU
            
            clear([0.0, 0.0, 0.0, 0.5], g);   // [0.1, 0.1, 0.1, 0.1]  // Looks cool without the clear()            
            draw::clear_buf(_grey, &mut frame_buf);




            //? Input handling red            
            //---------- Import objects --------------//
            // let mut camera_rc = game_objects.get_object("camera");
            let mat_view: Mat4x4;
            let camera_inputs = Inputs::from_inputs(CAMERA_MODE, j_is_pressed, l_is_pressed, k_is_pressed, i_is_pressed, a_is_pressed, d_is_pressed, w_is_pressed, s_is_pressed, space_is_pressed);
            
            // Get CAMERA view matrix
            let mat_cam_rot_yaw:Mat4x4;
            {
                let camera_obj = game_objects.get_mut_unknown("camera").as_mut();
                let camera = camera_obj.as_mut_any().downcast_mut::<Player>().unwrap();
                let (mat_cam_rot_yaw_, look_dir) = camera.get_look_dir(); mat_cam_rot_yaw=mat_cam_rot_yaw_;
                mat_view = camera.get_view_mat(&look_dir, None);
                camera.update_before_draw(&camera_inputs);   
                println!("cam pos={:?}", camera.get_game_obj().collision_box.centre);
            }
            


            //---------------------------------------//
            let mut i =0;
            for obj in &game_objects.objects {
                let mut obj_clone = obj.get_game_obj().clone(); // OOOF Expensive this is not good
                obj_clone.transform(&mat_view);

                let color = if i == 0 {_red} else {_blue};
                if obj_clone.drawable {
                    obj_clone.get_collision_box().draw(&mat_proj, true, 1.0, 1, &mut frame_buf);
                }
                i+= 1;
            }

            //---------------------------------------//
            //                 Update                //
            //---------------------------------------//
            //? Update camera position
            let ptr = &game_objects as *const GameObjCollection; 
            let camera_obj = game_objects.get_mut_unknown("camera").as_mut();
            let camera = camera_obj.as_mut_any().downcast_mut::<Player>().unwrap();
            camera.update(GRAVITY, TERMINAL_VELOCITY, &UP, false, &camera_inputs, ptr);

            //? Update agent
            let agent = game_objects.get_mut_unknown("agent").as_mut();
            let agent = agent.as_mut_any().downcast_mut::<Agent>().unwrap();
            agent.update(&UP, ptr);


            // Drawing
            let axis_rot = &mat_cam_rot_yaw;
            axis.draw(&axis_rot, &mut frame_buf);


            piston_window::image(&tex, c.transform, g);
        });
    }



    return;








    //---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------









    // let fov_sf : f64 = 1.0f64 / ( (FOV * 0.5f64) / 180.0 * PI ).tan(); // This function takes in radians.
    // //let fov_sf: f64 = 1f64 /  ( (FOV / 2f64).to_radians() ).tan();     //convenience for fov scale factor. The /180 * pi is converting the angle into radians.

    // //? Camera & light
    // let mut camera: Vec3D = Vec3D{x: 0.0, y:0.0, z: 0.0};
    // let mut mlight = DirLight::new( // main_light
    //     Vec3D { x: 0.2, y: 0.5, z: -1.0 }, 
    //     _white,
    // );
    // let UP : Vec3D = Vec3D::new(0.0, -1.0, 0.0); // Constant.
    // let mut yaw : f64 = 0.0; // Y rotation of camera

    // //? Projection matrix
    // let mut mat_proj : Mat4x4 = Mat4x4::zero(); // [row][collumn] is the standard
    // mat_proj.O[0][0] = F_ASPECT_RATIO * fov_sf;
    // mat_proj.O[1][1] = fov_sf;
    // mat_proj.O[2][2] = Z_FAR / (Z_FAR - Z_NEAR);
    // mat_proj.O[3][2] = (-Z_FAR * Z_NEAR) / (Z_FAR - Z_NEAR);
    // mat_proj.O[2][3] = 1.0f64;
    // mat_proj.O[3][3] = 0.0f64;


    // //---------------------- GAME OBJECTS -------------------
    // //? File
    // let cube1 : Mesh     =  Mesh::load_from_obj(r"cuboid.obj").expect("ERR: Failed to load mesh from path 'cow.obj'");
    // let cube2 : Mesh = Mesh::load_from_obj("cuboid.obj").expect("ERR: Failed to load cuboid mesh");
    // let cube1:GameObj = GameObj::new("cube1", cube1); let cube2:GameObj = GameObj::new("cube2", cube2);
    // let mut game_objects: Vec<GameObj> = vec![cube1, cube2];
    // for t in &mut game_objects[1].mesh.triangles {for p in &mut t.p {p.y -= 10.0;}}
    // for obj in &mut game_objects {for t in &mut obj.mesh.triangles {for i in 0..3 {t.p[i].z += 10.0}}}
    // let game_objects_map: HashMap<&str, usize> = HashMap::from_iter( vec![("cube1", 0), ("cube2", 1)].into_iter() );

    // //? Texture file
    // let img = image::open("mreow.png").unwrap();     
    // let img = img.to_rgba8();

    // let axis = Axis::new([Vec3D::new(1.,0.,0.), Vec3D::new(0.,-1.,0.), Vec3D::new(0.,0.,1.)],
    // [(0.9*SCREEN_WIDTH as f32) as usize, (0.1*SCREEN_HEIGHT as f32) as usize]);
    // //-------------------------------------------------------

    // println!("{:?}", mat_proj.O);    
    
    // //? Create a new window, frame_buf
    // let mut window : PistonWindow = WindowSettings::new("3D simulation",[SCREEN_WIDTH, SCREEN_HEIGHT])
    //     .exit_on_esc(true)
    //     .resizable(true)
    //     .transparent(true)
    //     .build()
    //     .unwrap_or_else(|e| {panic!("Failed to build PistonWindow {}",e)});
    
    // let mut frame_buf: FrameBuf = ImageBuffer::from_pixel( window.size().width as u32,  window.size().height as u32, image::Rgba(_grey));
    // // Context for updating textures
    // let mut tex_con = piston_window::TextureContext { 
    //     factory: window.factory.clone(),
    //     encoder: window.factory.create_command_buffer().into(),
    // };

    // let mut tex = piston_window::Texture::from_image(
    //     &mut tex_con, 
    //     &frame_buf, 
    //     &TextureSettings::new(),
    // ).expect("Failed to create texture");

    // //---------------
    // let mut w_is_pressed : bool = false;  let mut a_is_pressed : bool = false;    let mut i_is_pressed : bool = false;  let mut j_is_pressed : bool = false;  
    // let mut s_is_pressed : bool = false;  let mut d_is_pressed : bool = false;    let mut k_is_pressed : bool = false;  let mut l_is_pressed : bool = false;  
    // let mut mid_is_pressed:bool=false; //? Hold for resizing buffer.

    // // window.set_max_fps(10);
    // let mut trig_projected: Triangle = Triangle::zero();    


    // // let mut mat_rotz : Mat4x4 = Mat4x4::init_rotz_matrix(); // Perform rotation transform around a specific axis.
    // // let mut mat_rotx : Mat4x4 = Mat4x4::init_rotx_matrix();
    // // let mut mat_roty : Mat4x4 = Mat4x4::init_roty_matrix();
    // // let mut trans_mat: Mat4x4 = Mat4x4::init_translation_matrix();

    // //? Depth buffer
    // let mut DEPTH_BUF = [0. ; (SCREEN_WIDTH as usize) * SCREEN_HEIGHT as usize]; //[[0. ; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize];
    
    // let dt:f64 =1.0;   

    // while let Some(e) = window.next(){

    //     // Input handling
    //     if let Some(Button::Keyboard(key)) = e.press_args() {
    //         match key{
    //             Key::W => w_is_pressed = true,
    //             Key::A => a_is_pressed = true,
    //             Key::S => s_is_pressed = true,
    //             Key::D => d_is_pressed = true,

    //             Key::I => i_is_pressed = true,
    //             Key::J => j_is_pressed = true,
    //             Key::K => k_is_pressed = true,
    //             Key::L => l_is_pressed = true,
    //             _ => println!("Unknown key pressed"),
    //         }
    //     }
    //     if let Some(Button::Keyboard(key)) =  e.release_args() {
    //         match key{
    //             Key::W => w_is_pressed = false,
    //             Key::A => a_is_pressed = false,
    //             Key::S => s_is_pressed = false,
    //             Key::D => d_is_pressed = false,

    //             Key::I => i_is_pressed = false,
    //             Key::J => j_is_pressed = false,
    //             Key::K => k_is_pressed = false,
    //             Key::L => l_is_pressed = false,
    //             _ => println!("Unknown key released"),
    //         }
    //     }
    //     if let Some(Button::Mouse(MouseButton::Middle)) = e.press_args() { mid_is_pressed = true}
    //     else if let Some(Button::Mouse(MouseButton::Middle)) = e.release_args() { mid_is_pressed = false}


    //     if mid_is_pressed {
    //         let (win_w, win_h) = (window.size().width as usize, window.size().height as usize);
    //         if frame_buf.len()/4 != win_w * win_h { // /4 cos rgba
    //             frame_buf =  ImageBuffer::from_pixel(win_w as u32 , win_h as u32,image::Rgba(_grey));
                
    //             tex = piston_window::Texture::from_image(&mut tex_con, &frame_buf, &TextureSettings::new())
    //                 .expect("Failed to create texture");
    //         }
    //     }
    //     else {
    //         window.draw_2d(&e, |c, g, device|{

    //             tex.update(&mut tex_con, &frame_buf).unwrap();
    //             tex_con.encoder.flush(device); // Flush context into GPU
                
    //             clear([0.0, 0.0, 0.0, 0.5], g);   // [0.1, 0.1, 0.1, 0.1]  // Looks cool without the clear()            
    //             draw::clear_buf(_grey, &mut frame_buf);
    //             for i in 0..DEPTH_BUF.len() {DEPTH_BUF[i] = 0.0} // DEPTH_BUF.iter_mut().map(|_| 0.0);
    //             piston_window::image(&tex, c.transform, g);
                
               


    //             //* UPDATE */
    //             // Gather info for mat_cam
    //             let target_pos : Vec3D =Vec3D::new(0.0, 0.0, 1.0).normalized(); // Unit vector that travels along the direction we want the camera to point.

    //             let mut mat_cam_rot = Mat4x4::init_roty_matrix(); mat_cam_rot.make_roty_matrix(yaw);
    //             // Take a target vec fixed along the z axis, rotate it by yaw, from this we geta  new forward facing vector
    //             let look_dir = vec_multiply_mat(&target_pos, &mat_cam_rot).normalized();
    //             let target_pos : Vec3D = camera.add_vec(&look_dir);
                
    //             update(&mut game_objects, &game_objects_map, dt);


    //             {//* INPUT HANDLING */
    //                 const MOVEMENT_SPEED: f64 = 0.1;
    //                 let forward : Vec3D = look_dir.mult_scalar(MOVEMENT_SPEED); // Times by movement speed ( and dt), we can assume look_dir is normalized
    
    //                 if w_is_pressed{camera= camera.add_vec(&forward)}; if s_is_pressed{camera= camera.sub_vec(&forward)}; // Move forward/backward 

    //                 if i_is_pressed{ camera.y += MOVEMENT_SPEED}; //if a_is_pressed{ camera.x += 0.1};
    //                 if k_is_pressed{ camera.y -= MOVEMENT_SPEED}; //if d_is_pressed{ camera.x -= 0.1};

                    
    //                 //TODO: Strafing, very jerky movement for some reason.
    //                 let mut rot_mat_a:Mat4x4 = Mat4x4::init_rotx_matrix(); rot_mat_a.make_roty_matrix(-PI/2.0);
    //                 let mut rot_mat_b:Mat4x4  = Mat4x4::init_rotx_matrix(); rot_mat_b.make_roty_matrix(PI/2.0);
    //                 let left_strafe_vec:Vec3D  = vec_multiply_mat(&forward, &rot_mat_a); //.normalized().mult_scalar(MOVEMENT_SPEED);
    //                 let right_strafe_vec:Vec3D = vec_multiply_mat(&forward, &rot_mat_b); // .normalized().mult_scalar(MOVEMENT_SPEED);

    //                 if a_is_pressed {camera = camera.add_vec(&left_strafe_vec) }
    //                 if d_is_pressed {camera = camera.add_vec(&right_strafe_vec)}

    //                 // Turn left/right
    //                 if j_is_pressed{ yaw -= 0.05};  if l_is_pressed{ yaw += 0.05}; 
                    
    //             }

    //             let mat_cam : Mat4x4 = funky::matrix_point_at(&camera, &target_pos, &UP);
                
    //             // Make view matrix
    //             let mat_view : Mat4x4 = funky::mat_quick_inverse(&mat_cam); 

    //             let mut depth_buf = Vec::new();

    //             for obj in &game_objects {

    //                 for trig in &obj.mesh.triangles {

    //                     trig_projected = trig.clone();
    //                     trig_projected.col(_white);
                        

    //                     //~~~~~~~~~~~~~~Normal~~~~~~~~~~~~~~r
    //                     let normal : Vec3D = funky::normal(&trig_projected);
    //                     //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~r

    //                     let camera_ray : Vec3D = trig_projected.p[0].sub_vec(&camera); // All points lie on the same plane so doesn't matter

    //                     // Optimise: only do math if we can see it.
    //                     // If ray is alligned with normal, then triangle is visible.
    //                     if  normal.dot(&camera_ray) < 0.0 {   
                            

    //                         //* Illumination *//
    //                         let intensity : f32 = mlight.intensity(&normal); // Intensity = Dot product between mlight.dir and shape normal                        
    //                         // println!("{}",(mlight.col[0] as f32 * intensity) as u8 );
    //                         // println!("{}",(mlight.col[1] as f32 * intensity) as u8 );
    //                         //println!("{}",(mlight.col[2] as f32 * intensity) as u8 );
    //                         let projected_light = [
    //                             (mlight.col[0] as f32 * intensity) as u8,
    //                             (mlight.col[1] as f32 * intensity) as u8,
    //                             (mlight.col[2] as f32 * intensity) as u8,
    //                             mlight.col[3],
    //                         ];
                            
    //                         trig_projected.apply_light(&projected_light); //TODO: Issue here
    //                         //*----------------------------- ------------------ */
    //                         //* Convert World Space to the camera's view space */
    //                         for i in 0..3 {   trig_projected.p[i] = vec_multiply_mat(&trig_projected.p[i], &mat_view)    };

    //                         //* CLIP: Depth clipping */
    //                         let mut clipped: Vec<Triangle> = triangle_clip_plane(
    //                             &Vec3D::new(0.0,0.0,1.0),  // Offset of the plane from our camera
    //                             &Vec3D::new(0.0,0.,1.0), 
    //                             &trig_projected, 
    //                         CLIP_COLOR_IT);


    //                         for n in 0..clipped.len() { // project our clipped triangles
    //                             // //* TEXTURE: Before the transform, backup the z values to the textures before they get corrupted */
    //                             for i in 0..3 {
    //                                 clipped[n].tex[i].u /= clipped[n].p[i].z; clipped[n].tex[i].v /= clipped[n].p[i].z;
    //                                 clipped[n].tex[i].w = 1. / clipped[n].p[i].z; 
    //                             }
                                
    //                             //? Apply transform
    //                             for i in 0..3 {   clipped[n].p[i] = vec_multiply_mat(&clipped[n].p[i], &mat_proj)    }; // TODO: Lol wtf

    //                             //? Shift into view. (already projected so won't care about z)
    //                             ////trans_mat.make_translation_matrix( 1.0, 1.0, 0.0);             for i in  0..3 {    trig_projected.p[i] = vec_multiply_mat(&trig_projected.p[i], &trans_mat)    };
    //                             for i in 0..3 {   clipped[n].p[i].x +=1.0;  clipped[n].p[i].y +=1.0;  };

    //                             //? Scale into view
    //                             for i in 0..3 {   clipped[n].p[i].x *= 0.5f64 * F64_SCREEN_WIDTH;   clipped[n].p[i].y *= 0.5f64 * F64_SCREEN_HEIGHT    };

    //                             //? Store triangles for sorting.
    //                             depth_buf.push(clipped[n].clone() ); // Can't push a reference since it'll die the next loop so must copy.   
                            
    //                         }
    //                     }

    //                 }
    //             }

    //             //* Only draw the trigs when we're actually going to draw them (so we only get the most recent pixels) */
    //             //Sort triangles by z before drawing; painters algoritm (draw from back to front)
    //             if USE_PAINTERS_ALGORITHM {
    //                 depth_buf.sort_by(|t1, t2|{
    //                     let mz_1 : f64 = (t1.p[0].z + t1.p[1].z + t1.p[2].z ) / 3.0; // z Midpoints
    //                     let mz_2 : f64 = (t2.p[0].z + t2.p[1].z + t2.p[2].z ) / 3.0;
    //                     // mz_1 < mz_2  is true then swap
    //                     mz_2.partial_cmp(&mz_1).unwrap() // I'm pretty sure this isn't comparing properly, an it's also ugly so make ur own func plz.
    //                 });
    //             }
                

    //             // Draw the triangles

    //             // Planes, [p, normal]
    //             let PLANES: Vec<[Vec3D; 2]> = vec![  // only needs to be created once but whatevs
    //                 [Vec3D::new(0.0, 1.0, 0.0), Vec3D::new(0.0, 1.0, 0.0)],   
    //                 //---------------------------------------
    //                 [Vec3D::new(0.0, F64_SCREEN_HEIGHT-1.0, 0.0), Vec3D::new(0.0, -1.0, 0.0)],
    //                 //-----------------------------------------------
    //                 [Vec3D::new(1.0, 0.0, 0.0), Vec3D::new(1.0, 0.0, 0.0)], 
    //                 //-----------------------------------------------
    //                 [Vec3D::new(F64_SCREEN_WIDTH-1.0, 0.0, 0.0), Vec3D::new(-1.0, 0.0, 0.0)],
    //             ];


                
            
    //             //* CLIP : Apply triangle clipping */
    //             // println!("\n\n----------NON DEPTH CLIPPING-----------");
    //             // println!("old_trigs={:?\n}", depth_buf);
    //             for trig_to_raster in &depth_buf {
    //                 let mut trig_queue: VecDeque<Triangle> = VecDeque::new();

    //                 trig_queue.push_back( trig_to_raster.clone() );

    //                 let mut n_new_triangles = 1;

    //                 for plane in &PLANES {
    //                     // println!("");
    //                     while n_new_triangles > 0 {
    //                         let test = trig_queue.pop_front().unwrap();

    //                         n_new_triangles -= 1;

    //                         let new_trigs = triangle_clip_plane(&plane[0], &plane[1], &test, CLIP_COLOR_IT);


    //                         new_trigs.iter()
    //                             .for_each(|trig| trig_queue.push_back(trig.clone() ));
    //                     }
    //                     n_new_triangles = trig_queue.len();
    //                 }
    //                 // println!("new_trigs={:?}", trig_queue);
    //                 // println!("-------------------------\n\n");



    //                 //? Draw the triangles

    //                 for trig in &trig_queue{
                        
    //                     draw::fill_triangle_2d(&trig,  trig.col,  &mut frame_buf);
    //                     // draw::old_fill_triangle_2d(&trig,  trig.col,5, 2, &mut frame_buf);
    //                     // draw::efficient_fill_triangle_2d(&trig, 5, 5, trig.col, &mut frame_buf);

    //                     // draw::draw_square(trig.p[0].x, trig.p[0].y, 5, _red, &mut frame_buf);
    //                     // draw::draw_square(trig.p[1].x, trig.p[1].y, 5, _blue, &mut frame_buf);
    //                     // draw::draw_square(trig.p[2].x, trig.p[2].y, 5, _green, &mut frame_buf);

                        
    //                     // Texture 
    //                     // let _ = draw::fill_triangle_texture(&trig, &img, Some(&mut DEPTH_BUF), &mut frame_buf);
    //                     // Wireframe   
    //                     draw::draw_triangle(&trig,  1.0,1, _green, &mut frame_buf);


    //                 }

    //             }

    //             axis.draw(&mat_cam_rot, &mut frame_buf);

    //         });
    //     }
    // }
    
}















































// let cube1_points:Vec<Vec3D> = vec![
//     Vec3D::new(-1.,-1.,-1.),
//     Vec3D::new(1.,-1.,-1.),
//     Vec3D::new(1.,1.,-1.),
//     Vec3D::new(-1.,1.,-1.),
//     Vec3D::new(-1.,-1.,1.),
//     Vec3D::new(1.,-1.,1.),
//     Vec3D::new(1.,1.,1.),
//     Vec3D::new(-1.,1.,1.),
// ];

// // Should be 6 faces (user needs to enforce winding order)
// let cube_faces:Vec<(Vec<usize>, Vec3D)> = vec![
//     (vec![0,1,5,4], Vec3D::new(0.,-1.,0.)), // left face 
//     (vec![4,5,6,7], Vec3D::new(0.,0.,1.)),// top face 
//     (vec![6,7,3,2], Vec3D::new(0.,1.,0.)), // right face 
//     (vec![0,1,2,3], Vec3D::new(0.,0.,-1.)),// base face 

//     (vec![1,2,6,5], Vec3D::new(1.,0.,0.) ), // front face 
//     (vec![0,3,7,4], Vec3D::new(-1.,0.,0.)), // back face
// ];

// let camera_points:Vec<Vec3D> = cube1_points.iter()
//     .map(|p| {
//         let  q = p.add_vec(&Vec3D::new(0.,-3.,-10.));
//         q
//     }).collect();

// let platform_points: Vec<Vec3D> = cube1_points.iter()
//     .map(|p| {
//         let mut q = p.add_vec(&Vec3D::new(0., 3.0, 0.0));
//         if q.z > 0.0 { q.z *= 5.0;}
//         q.x *= 5.0;
//         q
//     })
//     .collect();

// let platform2_points: Vec<Vec3D> = cube1_points.iter()
//     .map(|p| {
//         let mut q = p.add_vec(&Vec3D::new(0., 3.0, 0.0));
//         q.x += 2.5;

//         q.y -= 3.0;
//         q.z += 10.0;
//         q
//     }).collect();
// let platform3_points: Vec<Vec3D> = platform2_points.iter()
//     .map(|p|  {
//         let mut q = p.clone();
//         q.z += 3.0;
//         q.x -= 6.0;
//         q.y -= 3.0;
//         q
//     }).collect();
// let platform4_points:Vec<Vec3D> = platform3_points.iter()
//     .map(|p|  {
//         let mut q = p.clone();
//         q.z += 7.0;
//         q.y -= 3.5;
//         q
//     }).collect();
