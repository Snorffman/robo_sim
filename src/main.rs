//TODO: Lots of sorting out the value types
//TODO: Also, pre-multiplying a world matrix is prolly more efficient

//? THe for loops could be slightly more inneficient compared to just doing the thing for each vec.
extern crate piston;
extern crate piston_window;
extern crate image;

use piston_window::*;
use image::ImageBuffer;

mod draw;
mod lib;
mod normal_stuff;

use crate::{lib::*, draw::fill_triangle_2d, FrameBuf, Color};
use normal_stuff as funky;


////! Customisables
const ROT_SPEED : f64 = 0.05; // 0.05 or 0.02
const BACKROUND_COL : [f32 ; 4]= [0.0,0.0,0.0,0.0];  // Grey : [0.1, 0.1, 0.1, 0.1]      Black : [0.0,0.0,0.0,0.0]

////!
pub const SCREEN_WIDTH  : u32 =  1040;
pub const SCREEN_HEIGHT : u32=   540;
const F64_SCREEN_WIDTH  : f64 = SCREEN_WIDTH  as f64;
const F64_SCREEN_HEIGHT : f64 = SCREEN_HEIGHT as f64;

const PI : f64 = 3.14159; // or std::f64::consts::PI;
//Mat proj values
const FOV : f64= 90.0;
const Z_FAR : f64 = 1000.0;
const Z_NEAR : f64 = 0.1;
const F_ASPECT_RATIO : f64 = F64_SCREEN_HEIGHT / F64_SCREEN_WIDTH;

//type ImageCol = [u8 ; 4];

fn main() {
    let _white: Color = [255, 255, 255, 255];
    let _grey:  Color = [0, 0, 0, 255];
    let _red:   Color = [255, 0, 0, 255];
    let _green: Color = [0, 255, 0, 255];
    let _blue:  Color = [0, 0, 255, 255];
    

    let fov_sf : f64 = 1.0f64 / ( (FOV * 0.5f64) / 180.0 * PI ).tan(); // This function takes in radians.
    //let fov_sf: f64 = 1f64 /  ( (FOV / 2f64).to_radians() ).tan();     //convenience for fov scale factor. The /180 * pi is converting the angle into radians.

    //? Camera & light
    let mut camera: Vec3D = Vec3D{x: 0.0, y:0.0, z: 0.0};
    let mut mlight = DirLight::new( // main_light
        Vec3D { x: 0.0, y: 0.0, z: -1.0 }, 
        _white,
    );
    let UP : Vec3D = Vec3D::new(0.0, -1.0, 0.0); // Constant.
    let mut yaw : f64 = 0.0; // Y rotation of camera

    //? Projection matrix
    let mut mat_proj : Mat4x4 = Mat4x4::zero(); // [row][collumn] is the standard
    mat_proj.O[0][0] = F_ASPECT_RATIO * fov_sf;
    mat_proj.O[1][1] = fov_sf;
    mat_proj.O[2][2] = Z_FAR / (Z_FAR - Z_NEAR);
    mat_proj.O[3][2] = (-Z_FAR * Z_NEAR) / (Z_FAR - Z_NEAR);
    mat_proj.O[2][3] = 1.0f64;
    mat_proj.O[3][3] = 0.0f64;
    
    //? File
    let obj_mesh : Mesh = Mesh::load_from_obj(r"cow.obj").expect("ERR: Failed to load mesh from path 'cow.obj'");
    
    println!("{:?}", mat_proj.O);    
    
    //? Create a new window, frame_buf
    let mut window : PistonWindow = WindowSettings::new("3D simulation",[SCREEN_WIDTH, SCREEN_HEIGHT])
        .exit_on_esc(true)
        .resizable(true)
        .transparent(true)
        .build()
        .unwrap_or_else(|e| {panic!("Failed to build PistonWindow {}",e)});
    
    let size = window.size();
    let mut frame_buf: FrameBuf = ImageBuffer::from_pixel( window.size().width as u32,  window.size().height as u32, image::Rgba(_grey));
    // Context for updating textures
    let mut tex_con = piston_window::TextureContext { 
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };

    let mut tex = piston_window::Texture::from_image(
        &mut tex_con, 
        &frame_buf, 
        &TextureSettings::new(),
    ).expect("Failed to create texture");

    //---------------
    let mut w_is_pressed : bool = false;  let mut a_is_pressed : bool = false;    let mut i_is_pressed : bool = false;  let mut j_is_pressed : bool = false;  
    let mut s_is_pressed : bool = false;  let mut d_is_pressed : bool = false;    let mut k_is_pressed : bool = false;  let mut l_is_pressed : bool = false;  
    let mut mid_is_pressed:bool=false; //? Hold for resizing buffer.

    // window.set_max_fps(10);

    let mut trig_projected: Triangle = Triangle::zero();    


    let mut mat_rotz : Mat4x4 = Mat4x4::init_rotz_matrix(); // Perform rotation transform around a specific axis.
    let mut mat_rotx : Mat4x4 = Mat4x4::init_rotx_matrix();
    let mut mat_roty : Mat4x4 = Mat4x4::init_roty_matrix();
    let mut trans_mat: Mat4x4 = Mat4x4::init_translation_matrix();
    

    let mut theta : f64 = 0.0f64; // Constantly changing angle

    // let mut col_change = 0.0f32;
    // let mut rgb = 0;

    
    while let Some(e) = window.next(){
        // Input handling
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
                _ => println!("Unknown key released"),
            }
        }
        if let Some(Button::Mouse(MouseButton::Middle)) = e.press_args() { mid_is_pressed = true}
        else if let Some(Button::Mouse(MouseButton::Middle)) = e.release_args() { mid_is_pressed = false}

        if w_is_pressed{ camera.y += 0.1}; if a_is_pressed{ camera.x += 0.1};
        if s_is_pressed{ camera.y -= 0.1}; if d_is_pressed{ camera.x -= 0.1};

        if mid_is_pressed {
            let (win_w, win_h) = (window.size().width as usize, window.size().height as usize);
            if frame_buf.len()/4 != win_w * win_h { // /4 cos rgba
                frame_buf =  ImageBuffer::from_pixel(win_w as u32 , win_h as u32,image::Rgba(_grey));
                
                tex = piston_window::Texture::from_image(&mut tex_con, &frame_buf, &TextureSettings::new())
                    .expect("Failed to create texture");
            }
        }
        else {
            window.draw_2d(&e, |c, g, device|{
                tex.update(&mut tex_con, &frame_buf).unwrap();
                tex_con.encoder.flush(device); // Flush context into GPU
                
                clear([0.0, 0.0, 0.0, 0.5], g);   // [0.1, 0.1, 0.1, 0.1]  // Looks cool without the clear()            
                clear_buf(_grey, &mut frame_buf);
                piston_window::image(&tex, c.transform, g);


                //* UPDATE */
                //?~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~Rotation transform ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                theta += ROT_SPEED;
                //frame_buf.put_pixel(theta as u32, theta as u32, image::Rgba(_red));
                
                
                // // Rotation Z
                // mat_rotz.make_rotz_matrix(theta);
                // // Rotation X
                // mat_rotx.make_rotx_matrix(theta);
                // // Rotation Y
                // mat_roty.make_roty_matrix(theta);
                //?~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~         

                // Gather info for mat_cam
                
                let target_pos : Vec3D =Vec3D::new(0.0, 0.0, 1.0); // Unit vector that travels along the direction we want the camera to point.

                let mut mat_cam_rot = Mat4x4::init_roty_matrix(); mat_cam_rot.make_roty_matrix(yaw);
                // Take a target vec fixed along the z axis, rotate it by yaw, from this we geta  new forward facing vector
                let look_dir = vec_multiply_mat(&target_pos, &mat_cam_rot);
                let target_pos : Vec3D = camera.add_vec(&look_dir);

                {//* INPUT HANDLING */
                    let forward : Vec3D = look_dir.mult_scalar(0.1); // Times by movement speed ( and dt), we can assume look_dir is normalized
                
                    if i_is_pressed{camera= camera.add_vec(&forward)}; if k_is_pressed{camera= camera.sub_vec(&forward)}; // Move forward/backward 
                    if j_is_pressed{ yaw -= 0.05};  if l_is_pressed{ yaw += 0.05}; // Turn left/right
                }

                let mat_cam : Mat4x4 = funky::matrix_point_at(&camera, &target_pos, &UP);
                
                // Make view matrix
                let mat_view : Mat4x4 = funky::mat_quick_inverse(&mat_cam); 

                let mut depth_buf = Vec::new();
                // let mut depth_buf : Vec<Triangle> = Vec::with_capacity(obj_mesh.triangles.len() ); 
                for trig in &obj_mesh.triangles{

                    trig_projected = trig.clone();
                    trig_projected.col(_white);
                    //?~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~Apply Rotation Transform~~~~~~~~~~~~~~~~~~~~~~~~~~
                    // // Rotation Z
                    // for i in 0..3 {    trig_projected.p[i] = vec_multiply_mat(&trig_projected.p[i], &mat_rotz)    };
                    // // Rotation x
                    // for i in 0..3 {    trig_projected.p[i] = vec_multiply_mat(&trig_projected.p[i], &mat_rotx)    };
                    // // Rotation Y
                    // for i in 0..3 {    trig_projected.p[i] = vec_multiply_mat(&trig_projected.p[i], &mat_roty)    };
                    //?~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                    
                    
                    ////! Translate away from face, into the screen.
                    for i in 0..3 {    trig_projected.p[i].z += 10.0    };

                    //~~~~~~~~~~~~~~Normal~~~~~~~~~~~~~~r
                    let normal : Vec3D = funky::normal(&trig_projected);
                    //~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~r

                    let camera_ray : Vec3D = trig_projected.p[0].sub_vec(&camera); // All points lie on the same plane so doesn't matter

                    // Optimise: only do math if we can see it.
                    // If ray is alligned with normal, then triangle is visible.
                    if  normal.dot(&camera_ray) < 0.0 {   
                        

                        //* Illumination *//
                        let intensity : f32 = mlight.intensity(&normal); // Intensity = Dot product between mlight.dir and shape normal                        
                        // println!("{}",(mlight.col[0] as f32 * intensity) as u8 );
                        // println!("{}",(mlight.col[1] as f32 * intensity) as u8 );
                        //println!("{}",(mlight.col[2] as f32 * intensity) as u8 );
                        let projected_light = [
                            (mlight.col[0] as f32 * intensity) as u8,
                            (mlight.col[1] as f32 * intensity) as u8,
                            (mlight.col[2] as f32 * intensity) as u8,
                            mlight.col[3],
                        ];
                        
                        trig_projected.apply_light(&projected_light); //TODO: Issue here
                        //*----------------------------------------------- */
                        //* Convert World Space to the camera's view space */
                        for i in 0..3 {   trig_projected.p[i] = vec_multiply_mat(&trig_projected.p[i], &mat_view)    };

                        //? Apply transform
                        for i in 0..3 {   trig_projected.p[i] = vec_multiply_mat(&trig_projected.p[i], &mat_proj)    };

                        //? Shift into view. (already projected so won't care about z)
                        ////trans_mat.make_translation_matrix( 1.0, 1.0, 0.0);             for i in  0..3 {    trig_projected.p[i] = vec_multiply_mat(&trig_projected.p[i], &trans_mat)    };
                        for i in 0..3{  trig_projected.p[i].x +=1.0;  trig_projected.p[i].y +=1.0;  };

                        //? Scale into view
                        for i in 0..3 {    trig_projected.p[i].x *= 0.5f64 * F64_SCREEN_WIDTH;   trig_projected.p[i].y *= 0.5f64 * F64_SCREEN_HEIGHT    };

                        //? Store triangles for sorting.
                        depth_buf.push(trig_projected.clone() ); // Can't push a reference since it'll die the next loop so must copy.   
                    }

                }

                //* Only draw the trigs when we're actually going to draw them (so we only get the most recent pixels) */
                //Sort triangles by z before drawing; painters algoritm (draw from back to front)
                depth_buf.sort_by(|t1, t2|{
                    let mz_1 : f64 = (t1.p[0].z + t1.p[1].z + t1.p[2].z ) / 3.0; // z Midpoints
                    let mz_2 : f64 = (t2.p[0].z + t2.p[1].z + t2.p[2].z ) / 3.0;
                    // mz_1 < mz_2  is true then swap
                    mz_2.partial_cmp(&mz_1).unwrap() // I'm pretty sure this isn't comparing properly, an it's also ugly so make ur own func plz.
                });
                

                // Draw the triangles
                for trig in &depth_buf{
                    draw::fill_triangle_2d(&trig,  trig.col,  &mut frame_buf);
                    //draw::old_fill_triangle_2d(&trig,  trig.col,5, 2, &mut frame_buf);
                    // draw::efficient_fill_triangle_2d(&trig, 5, 5, trig.col, &mut frame_buf);

                    // draw::draw_square(trig.p[0].x, trig.p[0].y, 5, _red, &mut frame_buf);
                    // draw::draw_square(trig.p[1].x, trig.p[1].y, 5, _blue, &mut frame_buf);
                    // draw::draw_square(trig.p[2].x, trig.p[2].y, 5, _green, &mut frame_buf);
                                        
                    //draw::draw_triangle(&trig,  5.0,1, _green, &mut frame_buf);
                }
            });
        }
    }
    


}


fn vec_multiply_mat( v_old : &Vec3D, pmat: &Mat4x4) -> Vec3D{
    
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


fn axis(fb: &mut FrameBuf) {
    let (t_x, t_y) = (1000.0, 50.0); // Translation into a visually nice position.
    
    let mid = Vec3D::new(t_x, t_y, 0.0);
    let z   = Vec3D::new(t_x, t_y, 10.0);
    let y   = Vec3D::new(t_x, t_y + 10.0, 0.0);
    let x   = Vec3D::new(t_x + 10.0, t_y, 0.0);


    draw::draw_line_2d(&mid, &x, 1.0, 1, [1, 0, 0, 1], fb);
    draw::draw_line_2d(&mid, &y, 1.0, 1, [0, 1, 0, 1], fb);
    draw::draw_line_2d(&mid, &z, 1.0, 1, [0, 0, 1, 1], fb);
}

//TODO: Can't tell which one's faster.
fn clear_buf(color: Color, fb: &mut FrameBuf) { // REEALLY BAD
    for pix in fb.pixels_mut() {
        *pix = image::Rgba(color);
    }
    //*fb = image::ImageBuffer::from_pixel(SCREEN_WIDTH, SCREEN_HEIGHT, image::Rgba( color));
}