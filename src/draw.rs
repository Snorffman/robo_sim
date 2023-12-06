use crate::{lib::{Vec3D, Triangle, FrameBuf, Color}, F64_SCREEN_WIDTH, F64_SCREEN_HEIGHT};
// #[allow(unused)]
pub fn draw_pix(  x: f64, y: f64, color: Color, fb: &mut FrameBuf){
    let x = x as u32;
    let y = y as u32;

    fb.put_pixel(x, y,  image::Rgba(color));
}
pub fn draw_square(  x: f64, y: f64, scale : u32, color: Color, fb: &mut FrameBuf){ // Slower: Has scale & performs checks
    if (x < F64_SCREEN_WIDTH  && x > 0.0) && (y < F64_SCREEN_HEIGHT && y > 0.0) {
        let x = x as u32; //? If negative value, it'll be 0 (which is valid on buffer)
        let y = y as u32;

        for xscale in 0..scale{
            for yscale in 0..scale{
                fb.put_pixel(x + xscale, y + yscale,  image::Rgba(color));
            }
        }
    }

}
pub fn draw_rec(  x: f64, y: f64, xscale: u32, yscale: u32, color: Color, fb: &mut FrameBuf){
    // let x = x as u32;
    // let y = y as u32;

    for xscale in 0..xscale{
        for yscale in 0..yscale{
            draw_square(x + xscale as f64, y + yscale as f64,  1, color, fb);
        }
    }
}

// f(x) = y = mx + c //? Is it more efficient to round to integers rather than float?
pub fn draw_line_2d(v1 : &Vec3D, v2: &Vec3D,  res : f64, scale : u32,  color: Color, fb: &mut FrameBuf){
    let mut line: Vec3D = Vec3D::new(
        v2.x - v1.x,
        v2.y - v1.y,
        0.0
    );

    // Normalize
    let len : f64 = (line.x*line.x + line.y*line.y).sqrt(); // Can optimise z part by removing it completely (it's just 0)
    line.x /= len ; line.y /= len;                         // Another z ignore
    
    line.x *= res; line.y *= res;

    let mut _draw_pos : Vec3D = Vec3D::new(v1.x, v1.y, v1.z);

    let iterations: usize = (len / res) as usize;

    for _i in 0..iterations{  // Or for _v1.y doesn't really matter
        _draw_pos.x += line.x;
        _draw_pos.y += line.y;
        draw_square(_draw_pos.x, _draw_pos.y,   scale, color, fb);
        // draw_pix(_draw_pos.x, _draw_pos.y,   color, fb);
    }

}


pub fn draw_triangle(trig: &Triangle, res: f64,  scale: u32, color : Color, fb: &mut FrameBuf){
    let (_v1, _v2, _v3)= (&trig.p[0], &trig.p[1], &trig.p[2]);
    draw_line_2d(_v1, _v2, res, scale, color, fb);
    draw_line_2d(_v1, _v3, res, scale, color, fb);
    draw_line_2d(_v2, _v3, res, scale, color, fb);
}

//~~~~~~Currently working on
//TODO: This function should prolly have it's own thread

pub fn fill_triangle_2d(trig : &Triangle, color : Color,  fb: &mut FrameBuf){
    let _v1 = &trig.p[0];
    let _v2 = &trig.p[1];
    let _v3 = &trig.p[2];

    //println!("{:.2} : {:.2} : {:.2}", _v1.y, _v2.y, _v3.y);

    //? Order v1,v2,v3 as ascending y's.
    let mut vlow : Vec3D; let mut vmid : Vec3D ; let mut vhigh : Vec3D;

    vlow  = Vec3D::new(_v1.x, _v1.y, _v1.z);
    vhigh = Vec3D::new(_v2.x, _v2.y, _v3.y);
    vmid  = Vec3D::new(_v3.x, _v3.y, _v3.z);

    let swap = |left : &mut Vec3D, right : &mut Vec3D| {
        let temp: [f64 ; 2] = [right.x, right.y];

        right.x = left.x ; right.y = left.y;
        left.x = temp[0] ; left.y = temp[1];
    };

    if vlow.y > vmid.y { swap(&mut vlow, &mut vmid ) };
    if vlow.y > vhigh.y{ swap(&mut vlow, &mut vhigh) };
    if vmid.y > vhigh.y{ swap(&mut vmid, &mut vhigh) };
    //println!("{:.2} : {:.2} : {:.2}", vlow.y, vmid.y, vhigh.y);
    //?~~~~~~~~~~~~~~~~~ Get min / max x values : This is why we ordered it, as the low_high will always just be one line upwards.
    let mut low_mid  : Vec<f64> = x_cords_of_line(&vlow, &vmid);
    let mut mid_high : Vec<f64> = x_cords_of_line(&vmid, &vhigh);
    let mut low_high : Vec<f64> = x_cords_of_line(&vlow, &vhigh);

    // When iterating, it's faster to concatonate low_mid and mid_high beforehand then to check when to look at the other.
    low_mid.append(&mut mid_high);

    ////! Both low_mid and mid_high have the same lengthas y always  += 1 in x_cords_of_line

    //? Draw from max_left to max_right, then increase y & repeat. from top of monitor to bottom. y_min is actually the top since (0,0) top left
    let mut y  = vlow.y ; ////! Usize rounds it to 0 if it's negative (out of view)
    
    low_high.pop();  // Shortened from : if low_high.len() > low_mid.len() { low_high.pop() }

    for i in 0..low_high.len() { // <- Issue here in ymin..y_max  //? This goes from (Your view) bottom to top
        let x_left : f64;   let x_right : f64;        

        if low_mid[i] < low_high[i] {  //// Used to be if low_mid[i * len_sf]
            x_left  = low_mid[i];
            x_right = low_high[i];
        }
        else{
            x_left  = low_high[i];
            x_right = low_mid[i];
        }
        
        draw_rec(x_left  -1.0,  y +1.0 , (x_right - x_left) as u32 + 2 , 3, color, fb); // alternative is a for loop with draw_square, which allows more customisability but prolly less efficient
        
        y += 1.0;
        
    }
    // println!();

}

//~~~~~~~~~~~~~~
fn x_cords_of_line(_v1 : &Vec3D, _v2 : &Vec3D) -> Vec<f64>{ // Might have trouble with res but i don't think so
    let mut x_cords: Vec<f64> = vec![];

    let mut line : Vec3D;  // if _v1.

    if _v1.y < _v2.y{    //? v1_to_v2
        line = Vec3D::new(
            _v2.x - _v1.x,
            _v2.y - _v1.y,
            0.0
        );
    }else{               //? v2_to_v1
        line = Vec3D::new(
            _v1.x - _v2.x,
            _v1.y - _v2.y,
            0.0
        );
    }

    let iterations: usize = line.y as usize; // Original change in line y before scaling to 1. Iterations end when y is reached (or x is reached)


    //? Order by y (xcords[0] is lowest y value - so highest on the monitor)
    ////let mut _draw_pos : Vec3D =  Vec3D::new(_v1.x, _v1.y, _v1.z);
    let mut _draw_pos_x : f64 = _v1.x;

    //? Scale y to 1, scale x acordingly.
    line.x /= line.y;
    //line.y = 1.0;  ////! Actual equation is line.y /= line.y, but this is more efficient.
    
    
    for _i in 0..iterations{
        _draw_pos_x += line.x; // Used to be _draw_pos.x += line.x
        ////_draw_pos.y += line.y;
        
        x_cords.push(_draw_pos_x);

        ////draw_square(_draw_pos.x, _draw_pos.y,   1.0, [0.0, 0.0, 255.0, 1.0], &con, g);
    }

    x_cords
}















//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~#




//TODO: This function should prolly have it's own thread
//TODO: One promising idea is to add a needed_len param on the x_cords_of_line function,
// Make _draw_pos += line.x * len_sf, len_sf = needed_len / len.  Iterations are needed_len.
// If needed_len = 0, then have it behave normally (have needed_len = len) so we can calculate the length of low_high, and feed in low_high.len() into low_mid
// The result of that made the line seem too short however so there's some more things to be thought out

// It should only scale based on y, so for when  y of low_high changes by 1, y of low_mid changes by one and each new x_cord is an increase in y of 1 for both vectors.
#[allow(unused)]
pub fn old_fill_triangle_2d(trig : &Triangle, color : Color, res: usize, scale : u32, fb: &mut FrameBuf) {
    let _v1 = &trig.p[0];
    let _v2 = &trig.p[1];
    let _v3 = &trig.p[2];
    //? Order v1,v2,v3 as ascending y's.
    let mut vlow : Vec3D; let mut vmid : Vec3D ; let mut vhigh : Vec3D;

    vlow  = Vec3D::new(_v1.x, _v1.y, _v1.z);
    vhigh = Vec3D::new(_v2.x, _v2.y, _v3.y);
    vmid  = Vec3D::new(_v3.x, _v3.y, _v3.z);

    let swap = |left : &mut Vec3D, right : &mut Vec3D| {
        let temp: [f64 ; 2] = [right.x, right.y];

        right.x = left.x ; right.y = left.y;
        left.x = temp[0] ; left.y = temp[1];
    };

    if vlow.y > vmid.y { swap(&mut vlow, &mut vmid ) };
    if vlow.y > vhigh.y{ swap(&mut vlow, &mut vhigh) };
    if vmid.y > vhigh.y{ swap(&mut vmid, &mut vhigh) };
    
    // println!("Input : {:.2}, {:.2}, {:.2}",_v1.y, _v2.y, _v3.y);
    // println!("Lowest: {:.2}, {:.2}, {:.2} : Highest", vlow.y, vmid.y, vhigh.y);
    // println!();
    //?~~~~~~~~~~~~~~~~~ Get min / max x values : This is why we ordered it, as the low_high will always just be one line upwards.
    let mut low_mid  : Vec<f64> = x_cords_of_line(&vlow, &vmid);
    let mut mid_high : Vec<f64> = x_cords_of_line(&vmid, &vhigh);
    let mut low_high : Vec<f64> = x_cords_of_line(&vlow, &vhigh);

    // When iterating, it's faster to concatonate low_mid and mid_high beforehand then to check when to look at the other.
    low_mid.append(&mut mid_high);

    ////! Test area is above---------------

    //? Draw from max_left to max_right, then increase y & repeat. from top of monitor to bottom. y_min is actually the top since (0,0) top left
    let mut y : f64 = vlow.y;

    //? Make low_mid have the same length as low_high (solved by having y += 1 every time on x_cords_of_line)

    //println!("low_high len={}  : low_mid len = {} ", low_high.len(), low_mid.len() );

    // let len_sf : usize = low_mid.len() / low_high.len();

    
    low_high.pop();  // Shortened from : if low_high.len() > low_mid.len() { low_high.pop() }

    for i in 0..low_high.len() { // <- Issue here in ymin..y_max  //? This goes from (Your view) bottom to top
        let x_left : f64;   let x_right : f64;        

        if i % res ==0 {

            if low_mid[i] < low_high[i] {  //// Used to be if low_mid[i * len_sf]
                x_left  = low_mid[i];
                x_right = low_high[i];
            }
            else{
                x_left  = low_high[i];
                x_right = low_mid[i];
            }
            
           

            for x in (x_left as usize)..(x_right as usize){ // Could prolly optimize
                if (x - x_left as usize)  % res == 0{ // if (x - x_left as usize)  % res == 0{ to have it directional
                    draw_square(x as f64, y,   scale, color, fb);
                }
            }
        }
        
        y += 1.0;
        
    }

}

#[allow(unused)]
pub fn efficient_fill_triangle_2d(trig : &Triangle, res : usize, scale : u32, color : Color,     fb: &mut FrameBuf){
    let _v1 = &trig.p[0];
    let _v2 = &trig.p[1];
    let _v3 = &trig.p[2];

    //println!("{:.2} : {:.2} : {:.2}", _v1.y, _v2.y, _v3.y);

    //? Order v1,v2,v3 as ascending y's.
    let mut vlow : Vec3D; let mut vmid : Vec3D ; let mut vhigh : Vec3D;

    vlow  = Vec3D::new(_v1.x, _v1.y, _v1.z);
    vhigh = Vec3D::new(_v2.x, _v2.y, _v3.y);
    vmid  = Vec3D::new(_v3.x, _v3.y, _v3.z);

    let swap = |left : &mut Vec3D, right : &mut Vec3D| {
        let temp: [f64 ; 2] = [right.x, right.y];

        right.x = left.x ; right.y = left.y;
        left.x = temp[0] ; left.y = temp[1];
    };

    if vlow.y > vmid.y { swap(&mut vlow, &mut vmid ) };
    if vlow.y > vhigh.y{ swap(&mut vlow, &mut vhigh) };
    if vmid.y > vhigh.y{ swap(&mut vmid, &mut vhigh) };
    //println!("{:.2} : {:.2} : {:.2}", vlow.y, vmid.y, vhigh.y);
    //?~~~~~~~~~~~~~~~~~ Get min / max x values : This is why we ordered it, as the low_high will always just be one line upwards.
    let mut low_mid  : Vec<f64> = x_cords_of_line(&vlow, &vmid);
    let mut mid_high : Vec<f64> = x_cords_of_line(&vmid, &vhigh);
    let mut low_high : Vec<f64> = x_cords_of_line(&vlow, &vhigh);

    // When iterating, it's faster to concatonate low_mid and mid_high beforehand then to check when to look at the other.
    low_mid.append(&mut mid_high);

    ////! Both low_mid and mid_high have the same lengthas y always  += 1 in x_cords_of_line

    //? Draw from max_left to max_right, then increase y & repeat. from top of monitor to bottom. y_min is actually the top since (0,0) top left
    let mut y  = vlow.y ; ////! Usize rounds it to 0 if it's negative (out of view)
    
    low_high.pop();  // Shortened from : if low_high.len() > low_mid.len() { low_high.pop() }
    let iterations : usize = low_high.len() / res;

    for i in 0..iterations { // <- Issue here in ymin..y_max  //? This goes from (Your view) bottom to top
        let x_left : f64;   let x_right : f64;     

        if low_mid[i] < low_high[i] {  //// Used to be if low_mid[i * len_sf]
            x_left  = low_mid[i];
            x_right = low_high[i];
        }
        else{
            x_left  = low_high[i];
            x_right = low_mid[i];
        }
        
        draw_rec(x_left ,  y +1.0 , (x_right - x_left) as u32 , scale * 2, color, fb); // alternative is a for loop with draw_square, which allows more customisability but less efficient
        
        y += 1.0 * res as f64;
        
    }

}



// //~~~~~~~~~~~~~~
// //TODO: Order it by y (bottom to top)
// //TODO: Take out the y stuffs, and contect g stuff
// fn x_cords_of_line(_v1 : &Vec3D, _v2 : &Vec3D) -> Vec<f64>{ // Might have trouble with res but i don't think so
//     let mut x_cords: Vec<f64> = vec![];

//     let mut line : Vec3D;  // if _v1.

//     if _v1.y < _v2.y{    //? v1_to_v2
//         line = Vec3D::new(
//             _v2.x - _v1.x,
//             _v2.y - _v1.y,
//             0.0
//         );
//     }else{               //? v2_to_v1
//         line = Vec3D::new(
//             _v1.x - _v2.x,
//             _v1.y - _v2.y,
//             0.0
//         );
//     }

//     let iterations: usize = line.y as usize; // Original change in line y before scaling to 1. Iterations end when y is reached (or x is reached)


//     //? Order by y (xcords[0] is lowest y value - so highest on the monitor)
//     ////let mut _draw_pos : Vec3D =  Vec3D::new(_v1.x, _v1.y, _v1.z);
//     let mut _draw_pos_x : f64 = _v1.x;

//     //? Scale y to 1, scale x acordingly.
//     line.x /= line.y;
//     //line.y = 1.0;  ////! Actual equation is line.y /= line.y, but this is more efficient.
    
    
//     for _i in 0..iterations{
//         _draw_pos_x += line.x; // Used to be _draw_pos.x += line.x
//         ////_draw_pos.y += line.y;
        
//         x_cords.push(_draw_pos_x);

//         ////draw_square(_draw_pos.x, _draw_pos.y,   1.0, [0.0, 0.0, 255.0, 1.0], &con, g);
//     }

//     x_cords
// }