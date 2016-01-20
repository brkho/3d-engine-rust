// An example program using the 3D Rust game engine.
//
// Brian Ho
// brian@brkho.com

extern crate mmo;
extern crate time;

use mmo::gfx::color;
use mmo::gfx::camera;
use mmo::gfx::camera::Camera;
use mmo::gfx::camera::EuclideanVector;
use mmo::gfx::game_window::*;
use mmo::gfx::light;
use mmo::gfx::material;
use mmo::gfx::model;
use mmo::gfx::types::*;
use mmo::util::{rmod, obj};

use std::path;
use std::process;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

// Macro to easily get asset strings.
macro_rules! asset { ($s:expr) => {{
    let mut path = path::PathBuf::from("assets");
    path.push($s);
    &path.to_str().unwrap().to_string() }};}

// Driver test program.
fn main() {
    let ground = obj::decode_obj(asset!("ground.obj")).unwrap();
    // let bunny = obj::decode_obj(asset!("plane.obj")).unwrap();
    // let budda = obj::decode_obj("budda.obj").unwrap();
    // let dragon = obj::decode_obj("dragon.obj").unwrap();
    let mut window = GameWindow::new(800, 600, "Engine Test".to_string()).unwrap();
    window.bg_color = color::Color::new_rgb(0.2, 0.2, 0.2);

    let camera1 = camera::PerspectiveCamera::new(
            Vector3D::new(17.0, 17.0, 17.0), Vector3D::new(0.0, 0.0, 0.0),
            window.get_aspect_ratio(), 45.0, 0.1, 100.0);
    let camera2 = camera::PerspectiveCamera::new(
            Vector3D::new(0.0001, 0.0, 30.0), Vector3D::new(0.0, 0.0, 0.0),
            window.get_aspect_ratio(), 45.0, 0.1, 100.0);
    let main_camera = window.attach_camera(camera1);
    let secondary_camera = window.attach_camera(camera2);
    window.set_active_camera(main_camera).unwrap();

    // let bunny_mat = material::Material::new_with_color(Some(asset!("stone_diffuse.bmp")),
    //         Some(asset!("stone_specular.bmp")), Some(asset!("stone_normal.bmp")),
    //         color::Color::new_rgb(1.0, 1.0, 1.0), 75.0);
    let bunny = rmod::decode_rmod(asset!("bunny.rmod")).unwrap();
    let bunny_info = Rc::new(model::ModelInfo::from_rmod(&bunny));
    let mut bunny_inst = model::ModelInstance::from(bunny_info.clone());
    bunny_inst.scale = 30.0;
    bunny_inst.pos = Vector3D::new(0.0, 0.0, 0.0);
    bunny_inst.update();

    let ground = rmod::decode_rmod(asset!("plane.rmod")).unwrap();
    let ground_info = Rc::new(model::ModelInfo::from_rmod(&ground));
    let mut ground_inst = model::ModelInstance::from(ground_info.clone());
    ground_inst.scale = 20.0;
    ground_inst.update();

    // let dragon_mat = material::Material::new_with_color(Some(asset!("uvs.bmp")),
    //     None, None,
    //     color::Color::new_rgb(1.0, 1.0, 1.0), 175.0);
    // let dragon_info = Rc::new(model::ModelInfo::from_obj(&dragon, dragon_mat));
    // let mut dragon_inst = model::ModelInstance::from(dragon_info.clone());
    // dragon_inst.scale = 0.6;
    // dragon_inst.pos = Vector3D::new(4.0, -4.0, 0.0);
    // dragon_inst.update();

    // let budda_mat = material::Material::new_with_color(Some(asset!("brian.bmp")),
    //         None, None,
    //         color::Color::new_rgb(1.0, 1.0, 1.0), 175.0);
    // let budda_info = Rc::new(model::ModelInfo::from_obj(&budda, budda_mat));
    // let mut budda_inst = model::ModelInstance::from(budda_info.clone());
    // budda_inst.pos = Vector3D::new(3.5, 3.5, 1.0);
    // budda_inst.update();

    let lb_mat = material::Material::new_with_color(None,
            None, None,
            color::Color::new_rgb(0.0, 0.0, 0.0), 75.0);
    let lb = Rc::new(model::ModelInfo::new_box(1.0, 1.0, 1.0, lb_mat));
    let mut lb1_inst = model::ModelInstance::from(lb.clone());
    lb1_inst.update();

    let mut lb2_inst = model::ModelInstance::from(lb.clone());
    lb2_inst.update();

    // let spot_obj = light::SpotLight::new(color::Color::new_rgb(0.3, 0.3, 0.3),
    //         Vector3D::new(0.0, 15.0, 15.0), Vector3D::new(0.0, -1.0, -1.0), 1.0, 0.0, 0.0, 0.4,
    //         42.0);
    // let spot_handle = window.attach_spot_light(spot_obj);

    // let dir_obj = light::DirectionalLight::new(color::Color::new_rgb(0.5, 0.5, 0.5),
    //         Vector3D::new(0.0, 0.0, -1.0));
    // let dir_handle = window.attach_directional_light(dir_obj);

    let pl1_obj = light::PointLight::new(color::Color::new_rgb(1.0, 1.0, 1.0),
            Vector3D::new(3.0, 3.0, 1.0), 1.0, 0.03, 0.004);
    let pl1_handle = window.attach_point_light(pl1_obj);

    // let point_light2 = window.attach_point_light(
    //         color::Color::new_rgb(1.0, 1.0, 1.0),
    //         Vector3D::new(3.0, 3.0, 1.0), 1.0, 0.06, 0.008).unwrap();

    let mut left_pressed = 0;
    let mut right_pressed = 0;
    let mut up_pressed = 0;
    let mut down_pressed = 0;
    let mut shift_pressed = 0;
    let mut last_time = time::now().to_timespec();
    let mut elapsed_time = 0.0;
    let mut frame_count = 0;
    loop {
        frame_count += 1;
        let curr_time = time::now().to_timespec();
        let elapsed_msec = (curr_time - last_time).num_microseconds().unwrap();
        let dt = elapsed_msec as f32 / 1000000.0;
        elapsed_time += dt;
        last_time = curr_time;
        if ((elapsed_time - dt) % 3.0) > (elapsed_time % 3.0) {
            println!("AVERAGE FPS: {}", frame_count as f32 / elapsed_time);
        }

        // Update Camera.
        {   if shift_pressed == 0 {
                window.set_active_camera(main_camera).unwrap();
            } else {
                window.set_active_camera(secondary_camera).unwrap();
                // window.clear_vertex_buffers();
            }
            let x_dir = (right_pressed - left_pressed) as f32 * 5.0 * dt;
            let y_dir = (up_pressed - down_pressed) as f32 * 5.0 * dt;
            let mut camera = window.get_active_camera_mut().unwrap();
            let cam_dir = camera.get_fwd();
            let fwd = Vector3D::new(cam_dir[0], cam_dir[1], 0.0).normalize();
            let right = camera.get_right();
            let dir = right * x_dir + fwd * -y_dir;
            camera.pos = camera.pos + dir;
            camera.target = camera.target; }
        window.update_active_camera();

        // Update Objects.
        lb1_inst.pos = Vector3D::new(10.0 * elapsed_time.cos(), 10.0 * elapsed_time.sin(), 1.0);
        lb1_inst.update();
        lb2_inst.pos = Vector3D::new(0.0, 10.0 * (1.43 * elapsed_time).sin(),
                10.0 * (1.43 * elapsed_time).cos());
        lb2_inst.update();

        // {   let mut light = window.get_spot_light_mut(spot_handle);
        //     let intensity = ((elapsed_time * 10.0).sin() + 1.0) / 2.0;
        //     light.intensity = color::Color::new_rgb(intensity, intensity, intensity); }
        // window.update_spot_light(spot_handle);

        // {   let mut light = window.get_directional_light_mut(dir_handle);
        //     let intensity = ((elapsed_time * 5.0).sin() + 1.0) / 2.0;
        //     light.intensity = color::Color::new_rgb(intensity, intensity, intensity); }
        // window.update_directional_light(dir_handle);

        {   let mut light = window.get_point_light_mut(pl1_handle);
            let lpos = Vector3D::new(10.0 * elapsed_time.cos(), 10.0 * elapsed_time.sin(), 1.0);
            light.position = lpos; }
        window.update_point_light(pl1_handle);

        // {   let mut light = window.get_point_light(point_light2);
        //     let lpos = Vector3D::new(0.0, 10.0 * (1.43 * elapsed_time).sin(),
        //             10.0 * (1.43 * elapsed_time).cos());
        //     light.position = lpos;
        //     light.update(program); }
        // window.update_point_light(pl2_handle);

        // Draw Objects.
        window.clear();
        window.draw_instance(&bunny_inst);
        window.draw_instance(&lb1_inst);
        // window.draw_instance(&lb2_inst);
        window.draw_instance(&ground_inst);
        // window.draw_instance(&budda_inst);
        // window.draw_instance(&dragon_inst);
        window.swap_buffers();

        for event in window.poll_events() {
            match event {
                Event::KeyboardInput(state, _, Some(key)) => {
                    let pressed = if state == ElementState::Pressed { 1 } else { 0 };
                    match key {
                        VirtualKeyCode::Left => left_pressed = pressed,
                        VirtualKeyCode::Right => right_pressed = pressed,
                        VirtualKeyCode::Up => up_pressed = pressed,
                        VirtualKeyCode::Down => down_pressed = pressed,
                        VirtualKeyCode::LShift => shift_pressed = pressed,
                        _ => (),
                    }
                }
                Event::Closed => process::exit(0),
                _ => ()
            }
        }
        // sleep(Duration::from_millis(500));
    }
}