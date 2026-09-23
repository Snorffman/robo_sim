
extern crate image;
use piston_window::*;
use image::{ImageBuffer};


use crate::{Color, F_ASPECT_RATIO, FOV, FrameBuf, PI, SCREEN_HEIGHT, SCREEN_WIDTH, Z_FAR, Z_NEAR, draw::{self, _blue, _grey, _red, clear_buf, fill_triangle_2d, fill_triangle_texture}, game::game_obj::{CollisionBox, FaceInput}, lib::*};

pub fn image_test() {
    const _GREY:  Color = [0, 0, 0, 255];
    const _BLUE:  Color = [0, 0, 255, 255];

    //------------------- Load the texture----------------------------
    let img = image::open("mreow.png").unwrap();
    let img: ImageBuffer<image::Rgba<u8>, Vec<u8>> = img.to_rgba8();

    // let trig = Triangle::new_full( 
    //     Vec3D::new(100.,100.,0.0), 
    //     Vec3D::new(300.,100.0,0.0), 
    //     Vec3D::new(300.,300.0,0.0), 
    //     Vec2D::new(0.,1.), 
    //     Vec2D::new(0.,0.), 
    //     Vec2D::new(1.,0.) 
    // );

    let trig = Triangle::new_full( 
    Vec3D::new(520.,270.,0.0), 
    Vec3D::new(520.,243.0,0.0), 
    Vec3D::new(493.,243.0,0.0), 
    Tex2D::new(0.,1.), 
    Tex2D::new(0.,0.), 
    Tex2D::new(1.,0.) 
    );





    //--------------------------------------------------------------    
    //? Create a new window, frame_buf
    let mut window : PistonWindow = WindowSettings::new("3D simulation",[SCREEN_WIDTH, SCREEN_HEIGHT])
        .exit_on_esc(true)
        .resizable(true)
        .transparent(true)
        .build()
        .unwrap_or_else(|e| {panic!("Failed to build PistonWindow {}",e)});
    
    let size = window.size();
    let mut frame_buf: FrameBuf = ImageBuffer::from_pixel( window.size().width as u32,  window.size().height as u32, image::Rgba(_GREY));
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

    fill_triangle_2d(&trig, _BLUE, &mut frame_buf);

    let _ = fill_triangle_texture(&trig, &img, None, &mut frame_buf);

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g, device|{
            tex.update(&mut tex_con, &frame_buf).unwrap();
            tex_con.encoder.flush(device); // Flush context into GPU
            
            clear([0.0, 0.0, 0.0, 0.5], g);   // [0.1, 0.1, 0.1, 0.1]  // Looks cool without the clear()            
            clear_buf(_GREY, &mut frame_buf);
            piston_window::image(&tex, c.transform, g);



            let _ = fill_triangle_texture(&trig, &img, None, &mut frame_buf);
            // fill_triangle_2d(&trig, _BLUE, &mut frame_buf);
    
        });
    }



    // p.get_pixel()
}





pub fn rotate_collision_test() {
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


    // let mut t = Mat4x4::translation_matrix(1.55, 1.55, 0.0);
    // t.mat_mult_mat(&Mat4x4::roty_matrix(PI/4.));
    let mut t =Mat4x4::roty_matrix(PI/4.);
    t.mat_mult_mat(&Mat4x4::translation_matrix(1.55, 0.0, 1.55));

    let cube2_points: Vec<Vec3D> = cube1_points.iter()
        .map(|p| vec_multiply_mat(p, &t)).collect();
    // drop(t);
    println!("cube2_points={:?}", cube2_points);


    // Should be 6 faces (user needs to enforce winding order)
    let cube_faces:Vec<(Vec<usize>, Vec3D)> = vec![
        (vec![0,1,5,4], Vec3D::new(0.,-1.,0.)), // left face 
        (vec![4,5,6,7], Vec3D::new(0.,0.,1.)),// top face 
        (vec![6,7,3,2], Vec3D::new(0.,1.,0.)), // right face 
        (vec![0,1,2,3], Vec3D::new(0.,0.,-1.)),// base face 

        (vec![1,2,6,5], Vec3D::new(1.,0.,0.) ), // front face 
        (vec![0,3,7,4], Vec3D::new(-1.,0.,0.)), // back face
    ];

    let mut cube1 = CollisionBox::new(cube1_points, FaceInput::NormalExplicit(cube_faces.clone()) , Some(_red));
    let mut cube2 = CollisionBox::new(cube2_points, FaceInput::NormalExplicit(cube_faces) , None);

    //? Projection matrix
    let fov_sf : f64 = 1.0f64 / ( (FOV * 0.5f64) / 180.0 * PI ).tan();
    let mut mat_proj : Mat4x4 = Mat4x4::zero(); // [row][collumn] is the standard
    mat_proj.O[0][0] = F_ASPECT_RATIO * fov_sf;
    mat_proj.O[1][1] = fov_sf;
    mat_proj.O[2][2] = Z_FAR / (Z_FAR - Z_NEAR);
    mat_proj.O[3][2] = (-Z_FAR * Z_NEAR) / (Z_FAR - Z_NEAR);
    mat_proj.O[2][3] = 1.0f64;
    mat_proj.O[3][3] = 0.0f64;

    

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


    // cube1.draw(&mat_proj, 1.0, 1, _blue, &mut frame_buf);

    let mut t=0.01;
    while let Some(e) = window.next(){
        window.draw_2d(&e, |c, g, device|{
            tex.update(&mut tex_con, &frame_buf).unwrap();
            tex_con.encoder.flush(device); // Flush context into GPU
            
            clear([0.0, 0.0, 0.0, 0.5], g);   // [0.1, 0.1, 0.1, 0.1]  // Looks cool without the clear()            
            draw::clear_buf(_grey, &mut frame_buf);

            cube1.draw(&mat_proj, true, 1.0, 1, _blue, &mut frame_buf);
            cube2.draw(&mat_proj, true, 1.0, 1, _red,  &mut frame_buf);
            cube1.transform( &Mat4x4::roty_matrix(t));
            // t += 0.00001;
            // println!("collision = {}", cube1.intersect_sat(&cube2));
            println!("collision = {}", cube1.static_sat(&cube2).is_some());
            piston_window::image(&tex, c.transform, g);
        });
    }
}





fn weird_inverse_collision_test() {
    
}