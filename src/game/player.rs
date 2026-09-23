use std::{any::Any, f64::consts::PI, time::{Duration, Instant, SystemTime}};

use bml_ml::{NeuralNetwork, optimizer::Adam, rl::qnn::{Policy, QNN}};
use piston::Key::L;

use crate::{GRAVITY, TERMINAL_VELOCITY, game::{game_obj::{self, GameObj, GameObjCollection, GameObjType}, player}, lib::{Mat4x4, Vec3D, vec_multiply_mat}};

/// A game object with an attribute to store if we can jump or not
pub struct Player {
    game_obj: GameObj,
    pub is_camera:bool,
    pub speed:f64,
    pub velocity: Vec3D,
    pub jump_speed: f64,
    pub turn_speed:f64,

    pub yaw:f64,
}
impl Player {
    pub fn new(game_obj: GameObj, speed: f64, turn_speed: Option<f64>) -> Self {
        let is_camera = game_obj.drawable == false;
        Self { game_obj,is_camera, speed, velocity: Vec3D::zero(), jump_speed: 0.3, turn_speed:turn_speed.unwrap_or(0.05), yaw:0.0}
    }

    pub fn get_look_dir(&self) -> (Mat4x4, Vec3D) {
        let target_pos : Vec3D =Vec3D::new(0.0, 0.0, 1.0).normalized(); // Unit vector that travels along the direction we want the camera to point.
        let mat_cam_rot_yaw = Mat4x4::roty_matrix(self.yaw);
        // Take a target vec fixed along the z axis, rotate it by yaw, from this we geta  new forward facing vector
        let look_dir = vec_multiply_mat(&target_pos, &mat_cam_rot_yaw).normalized();
        return (mat_cam_rot_yaw, look_dir)
    }

    /// Separate to the update function because it looks nice to do it before the draw function
    pub fn update_before_draw(&mut self, inputs: &Inputs) {
        self.transform(&Mat4x4::translation_matrix_(&self.velocity) );
        // Rotate player so it faces the camera- If you do this after the inputs it has a nice effect where you can see a bit of cube tilt.
        let mut yaw_change = self.yaw;
        if inputs.turn_left {self.yaw += self.turn_speed}; if inputs.turn_right {self.yaw -= self.turn_speed}; yaw_change = yaw_change - self.yaw; 
        let c = self.get_collision_box().centre;
        self.transform(&Mat4x4::translation_matrix_(&c.mult_scalar(-1.0)));
        self.transform(&Mat4x4::roty_matrix(-yaw_change));
        self.transform(&Mat4x4::translation_matrix_(&c));
    }

    fn handle_inputs(&mut self,  inputs: &Inputs) {
        //*--- Movement and inputs handling ---- */
        let MOVEMENT_SPEED = self.speed;
        // Take a target vec fixed along the z axis, rotate it by yaw, from this we geta  new forward facing vector
        let (_,look_dir) = self.get_look_dir();
        let forward = look_dir.mult_scalar(MOVEMENT_SPEED);
        let left_strafe  = vec_multiply_mat( &forward, &Mat4x4::roty_matrix(-PI/2.0) );
        let right_strafe = vec_multiply_mat( &forward, &Mat4x4::roty_matrix(PI/2.0) );        

        if self.is_camera {
            if inputs.fly_up{ self.transform(&Mat4x4::translation_matrix(0., MOVEMENT_SPEED, 0.));}
            if inputs.fly_down {self.transform(&Mat4x4::translation_matrix(0., -MOVEMENT_SPEED, 0.));}
        }
        if inputs.left { self.transform(&Mat4x4::translation_matrix(right_strafe.x, right_strafe.y, right_strafe.z)); }
        if inputs.right{ self.transform(&Mat4x4::translation_matrix(left_strafe.x, left_strafe.y, left_strafe.z)); }
        if inputs.forward { self.transform(&Mat4x4::translation_matrix(forward.x, forward.y, forward.z)); }
        if inputs.backward { self.transform(&Mat4x4::translation_matrix(-forward.x, -forward.y, -forward.z)); }
    }

    pub fn update(&mut self, gravity: f64, terminal_velocity: f64,up:&Vec3D, update_before_draw:bool,     inputs: &Inputs, game_objects: *const GameObjCollection) {
        if update_before_draw == true {
            // Rotate player so it faces the camera- If you do this after the inputs it has a nice effect where you can see a bit of cube tilt.
            self.update_before_draw(inputs);
        }
        self.handle_inputs(inputs);

        //------------ Collide against all other objects -----------//
        if self.is_camera {return;}

        let mut normal_force_ = Vec3D::zero();
        let game_objects_ptr = unsafe{&*game_objects};
        for obj in &game_objects_ptr.objects {
            let obj = obj.as_ref().get_game_obj();
            let name = &obj.name;
            if name != &self.game_obj.name  {
                if let Some(r) = self.game_obj.get_mut_game_obj().static_sat(&obj) {
                    normal_force_ = normal_force_.add_vec(&r);
                }
            }
        }
        let normal_force : Option<Vec3D>;
        if normal_force_.len() == 0.0 {normal_force = None;}
        else {normal_force = Some(normal_force_)}

        //* Handling jumping ; You can jump if you are on a wall */
        'a: { if let Some(r) = &normal_force {
            self.velocity = Vec3D::zero();
            
            if r.len() == 0.0 {break 'a;}
            let r = r.normalized();

            // Jumping
            if r.dot(up) >= 0.0 {
                if inputs.jump {
                    self.velocity = Vec3D::new(0., -self.jump_speed, 0.0);
                }
            }
        } }


        // Update player using its velocity
        //? Gravity
        let ACC = Vec3D::new(0., gravity, 0.); 

        // Use terminal velocity on the y component
        self.velocity = self.velocity.add_vec(&ACC);
        self.velocity.y = f64::min(self.velocity.y, terminal_velocity);
        // self.transform(&Mat4x4::translation_matrix_(&self.velocity) );
    }


}
impl GameObjType for Player {
    fn get_game_obj(&self) -> &GameObj {&self.game_obj}
    fn get_mut_game_obj(&mut self) -> &mut GameObj {&mut self.game_obj}
    fn as_any(&self) -> &dyn Any {self}
    fn as_mut_any(&mut self) -> &mut dyn Any {self}
}


/// An ai 
pub struct Agent {
    player: Player,
    qnn: QNN,
    optimizer: Adam,

    //------ These attributes are moreso part of the environment ------//
    respawn_point: Vec3D, prev_respawn_time: Instant, truncate_period: Duration,
    episode: usize, progress:usize,
}
impl Agent {
    pub fn new(game_obj: GameObj, qnn:QNN, optimizer:Adam, respawn_point: Vec3D ) -> Self {
        let player = Player::new(game_obj, 0.5, Some(3. *PI/ 180.));
        
        let now = Instant::now();
        let truncate_period=  Duration::from_secs(15);
        
        Self { player, qnn, optimizer, respawn_point, prev_respawn_time: now, truncate_period, episode:0, progress:0}
    }

    fn get_state(&self) -> (Vec3D,Vec<f64>) {
        let pos = self.player.get_collision_box().centre;
        let vel = self.player.velocity;
        // let rot = self.player.yaw;

        // (pos, vec![pos.x,pos.y,pos.z, vel.x,vel.y,vel.z, rot.cos(), rot.sin()])
        (pos, vec![pos.x,pos.y,pos.z, vel.x,vel.y,vel.z])
    }

    pub fn update(&mut self, up: &Vec3D, game_objects: *const GameObjCollection) {
        //? 1) Make the action based on the current state
        // ACTIONS: [forward,backward,turn_left,turn_right, jump]
        let (state_pos,state)= self.get_state();
        let action = self.qnn.choose_action(&state);

        let mut input = Inputs::init();

        match action {
            0 => input.forward = true,
            1 => input.backward = true,
            2 => input.left = true,
            3 => input.right = true,
            // 4 => input.jump = true, 
            _ => {},
        }
        let gravity=GRAVITY; let terminal_velocity= TERMINAL_VELOCITY;
        self.player.update(gravity, terminal_velocity, up, true, &input, game_objects);
        let (new_state_pos,new_state) = self.get_state();

        //? 2) Kill the agent if they die
        let elapsed = self.prev_respawn_time.elapsed().as_secs();

        let truncated = elapsed > self.truncate_period.as_secs();
        let terminated = new_state_pos.y > 1.5 || new_state_pos.z > 39.;
    
        if terminated || truncated { // Reset position
            let centre = self.player.get_collision_box().centre;
            let goto_respawn = self.respawn_point.sub_vec(&centre);
            self.player.transform(&Mat4x4::translation_matrix_(&goto_respawn));
            self.prev_respawn_time = Instant::now();
            self.progress = 0;
        }

        //?---------------- Get score ------------------
        println!("Episode={}", self.episode);
        println!("yaw={},{}", self.player.yaw.cos(), self.player.yaw.sin());
        let is_terminal = terminated || truncated;

        // let landmark_reward = if  new_state_pos.x < 0.0 && new_state_pos.z > 7.0 {1.5} else if  new_state_pos.z > 7.0 {1.0} else {0.0};

        let landmarks = [Vec3D::new(2.5,1.,16.), Vec3D::new(-4.5,1.,16.),Vec3D::new(-4.5,1.,34.)];
        let dist_to_landmark = new_state_pos.sub_vec(&landmarks[self.progress]).len();
        let prev_dist_to_landmark = state_pos.sub_vec(&landmarks[self.progress]).len();
        if dist_to_landmark < 2.0 {self.progress = usize::min(landmarks.len()-1, self.progress+1);}
        println!("dist={} < 1?", dist_to_landmark);

        // let landmark_reward = if  new_state_pos.x < 0.0 && new_state_pos.z > 7.0 {1.5} 
        //     else if  new_state_pos.z > 7.0 {1.0} else {0.0};
        

        let reward = if terminated && new_state_pos.y > 1.5 {-10.0}
            else if terminated && new_state_pos.z > 35. {20.} // goal reached
            else if truncated{-5.}
            else { - (dist_to_landmark - prev_dist_to_landmark) };

            // else {new_state_pos.z - state_pos.z + landmark_reward + 0.1};

        println!("r={}, pos={}, progress={}", reward, new_state_pos.z, self.progress);
        self.qnn.store_transition(state, new_state, reward, action, is_terminal);

        let eps = match self.qnn.policy {
            Policy::EGreedy { eps, eps_dec:_, eps_min:_ } => eps, 
        };
        println!("eps= {}\n",  eps);
        //?--------------- Train agent -----------------
        self.qnn.train(&mut self.optimizer);
        self.episode += 1;

        if self.episode % 100 == 0 {
            // Save weights
            let _ = self.qnn.nn.save_weights("weights/weights.json");
        }
        
    }
    

}
impl GameObjType for Agent {
    fn get_game_obj(&self) -> &GameObj {&self.player.game_obj}
    fn get_mut_game_obj(&mut self) -> &mut GameObj {&mut self.player.game_obj}
    fn as_any(&self) -> &dyn Any {self}
    fn as_mut_any(&mut self) -> &mut dyn Any {self}
}




pub struct Inputs {
    pub fly_up: bool, pub fly_down: bool,
    pub turn_left:bool, pub turn_right:bool,
    //----------//
    pub jump: bool,
    pub left: bool,
    pub right: bool,
    pub forward: bool,
    pub backward: bool,

}
impl Inputs {
    pub fn init() -> Self {
        Inputs {fly_up:false,fly_down:false, turn_left:false,turn_right:false, jump: false, left: false, right: false, forward: false, backward: false }
    }
    pub fn from_inputs(camera_mode:bool,j_is_pressed:bool, l_is_pressed:bool, k_is_pressed:bool,i_is_pressed:bool,a_is_pressed:bool,d_is_pressed:bool,w_is_pressed:bool,s_is_pressed:bool,space_is_pressed:bool) -> Self {
        let mut camera_inputs = Inputs::init();
        if j_is_pressed {camera_inputs.turn_left = true;} if l_is_pressed {camera_inputs.turn_right = true;}
        if camera_mode {
            if k_is_pressed{ camera_inputs.fly_up=true;}// camera.transform(&Mat4x4::translation_matrix(0., MOVEMENT_SPEED, 0.));}
            if i_is_pressed{ camera_inputs.fly_down=true;}//camera.transform(&Mat4x4::translation_matrix(0., -MOVEMENT_SPEED, 0.));}
        }
        if a_is_pressed { camera_inputs.left=true;}// camera.transform(&Mat4x4::translation_matrix(right_strafe.x, right_strafe.y, right_strafe.z)); }
        if d_is_pressed { camera_inputs.right=true; }//camera.transform(&Mat4x4::translation_matrix(left_strafe.x, left_strafe.y, left_strafe.z)); }
        if w_is_pressed { camera_inputs.forward=true;}// camera.transform(&Mat4x4::translation_matrix(forward.x, forward.y, forward.z)); }
        if s_is_pressed { camera_inputs.backward=true;}// camera.transform(&Mat4x4::translation_matrix(-forward.x, -forward.y, -forward.z)); }
        if space_is_pressed {camera_inputs.jump = true;}
        camera_inputs
    }
}
