extern crate rustc_serialize;
extern crate sdl2;

mod vec2;

use vec2::Vec2;
use sdl2::pixels::Color;
use rustc_serialize::json;
use std::io::prelude::*;
use std::fs::File;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Renderer;
use sdl2::rect::Rect;
use std::collections::HashSet;
use std::time::Duration;

fn main() {

    /********
     ** ONLY STEP THAT CAN PANIC **

        1) Setup
            Load in.json, defines camera-angle, fov, h/w + walls
            start the event-pump, open window.
            start threads, and move things to wherever is needed.
    ********/

    let mut world = load_world_from_json_argv();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let _window = video_subsystem.window("Woof-3D", world.camera.width as u32, world.camera.height as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut events = sdl_context.event_pump().unwrap();
    let mut renderer = _window.renderer().build().unwrap();
    /********
        2) Game Loop
            poll on event-pump, check inputs, take action

            p1
                no-input, just spins
            p2
                wasd, forward backwards + rotation.
                no collsion detection
            p4
                collsion detection

        GameObject
            walls,
            camera { x, y, angle, fov, height, width },
            ?
    ********/

    'running: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit {..} => break 'running,
                _ => ()
            }
        }

        // Create a set of pressed Keys.
        let keys: HashSet<_> = events.keyboard_state().pressed_scancodes().filter_map(Keycode::from_scancode).collect();
        for key in keys {
            match key {
                Keycode::A => world.camera.theta += 0.015,
                Keycode::D => world.camera.theta -= 0.015,
                Keycode::W => {
                    let move_vec = Vec2 {
                        x: world.camera.theta.cos(),
                        y: world.camera.theta.sin()
                    };
                    world.camera.p0 = world.camera.p0.plus(&move_vec.multiply(0.05))
                },
                Keycode::S => {
                    let move_vec = Vec2 {
                        x: world.camera.theta.cos(),
                        y: world.camera.theta.sin()
                    };
                    world.camera.p0 = world.camera.p0.minus(&move_vec.multiply(0.05))
                }
                Keycode::Z => {
                    world.camera.horizon += 10;
                },
                Keycode::X => {
                    world.camera.horizon -= 10;
                }
                _ => {},
            }
        }

        draw_world(&mut renderer, &world);
        std::thread::sleep(Duration::from_millis(10));
    }

    /********
        3) Draw
            take input, draw to screen.
            clear, draw top half sky, bottom half ground
            1 draw call for each wall.
    ********/

}

fn draw_world(renderer: &mut Renderer, world: &World) {
    renderer.clear();

    let ref camera = world.camera;

    renderer.set_draw_color(world.sky_color);
    renderer.fill_rect(
        Rect::new(0, 0, camera.width as u32, camera.height as u32)
    );

    renderer.set_draw_color(world.ground_color);
    renderer.fill_rect(
        Rect::new(0, std::cmp::max(camera.horizon, 0) as i32, camera.width as u32, camera.height as u32)
    );


    // camera point + 1 vec for each edge of FOV gets 2 points.
    // get dist between those points
    // if we divide the length of that distance by the number of pixels,
    // we can use the center-point of these segments to get the radians for the center of each pixel.

    let ray_vecs = get_ray_vecs(camera);
    for (i, ray_vec) in ray_vecs.iter().enumerate() {
        let camera_vec = ray_vec.minus(&camera.p0);

        let mut closest_dist = std::f64::MAX;
        let mut closest_wall: Option<&Wall> = None;

        for wall in &world.walls {
            if let Some(distance) = get_distance_to_ray_line_intersection(&camera.p0, &camera_vec, &wall.p0, &wall.p1) {
                if distance < closest_dist {
                    closest_dist = distance;
                    closest_wall = Some(&wall);
                }
            }
        }

        if let Some(wall) = closest_wall {

            let wall_pixel_height = get_wall_pixel_height(&camera_vec, closest_dist, camera);

            draw_wall(renderer, i as i32, wall_pixel_height, camera.horizon, wall.color);
        }
    }

    renderer.present();
}

fn get_wall_pixel_height(v0: &Vec2, distance: f64, camera: &Camera) -> i32 {
    let v_fov = camera.h_fov * camera.height / camera.width;
    let half_fov = v_fov / 2.0;

    // p0 = intersection point
    let p0 = camera.p0.plus(&v0.multiply(distance));

    let v1 = Vec2 {
        x: (camera.theta + half_fov).cos(),
        y: (camera.theta + half_fov).sin()
    };
    // p1 = top point of cameras v-fov
    let p1_dist = distance / half_fov.sin();
    let p1 = camera.p0.plus(&v1.multiply(p1_dist));

    // ground -> top camera at intersection point
    let wall_dist = p0.distance(&p1);
    let percentage = get_draw_plane_height(camera) / wall_dist;

    // TODO: if the wall-height changes, we will need to do some additional math

    ((camera.height) * percentage) as i32
}

fn draw_wall(renderer: &mut Renderer, x: i32, height: i32, horizon: i32, color: Color) {
    renderer.set_draw_color(color);

    renderer.fill_rect(
        Rect::new(
            x, horizon - height, 1, (height * 2) as u32
        )
    );
}

fn get_draw_plane_width(camera: &Camera) -> f64 {
    let (p0, p1) = get_draw_plane_points(camera);
    p0.distance(&p1)
}

fn get_draw_plane_height(camera: &Camera) -> f64 {
    let v_fov = camera.h_fov * camera.height / camera.width;
    let half_fov = v_fov / 2.0;

    let p0 = camera.p0.clone();
    let p0_vec = Vec2 {
        x: (camera.theta + half_fov).cos(),
        y: (camera.theta + half_fov).sin()
    };

    let p1 = camera.p0.clone();
    let p1_vec = Vec2 {
        x: (camera.theta - half_fov).cos(),
        y: (camera.theta - half_fov).sin()
    };

    p0.plus(&p0_vec).distance(&p1.plus(&p1_vec))
}

fn get_draw_plane_points(camera: &Camera) -> (Vec2, Vec2) {
    let half_fov = camera.h_fov / 2.0;

    let p0 = camera.p0.clone();
    let p0_vec = Vec2 {
        x: (camera.theta + half_fov).cos(),
        y: (camera.theta + half_fov).sin()
    };

    let p1 = camera.p0.clone();
    let p1_vec = Vec2 {
        x: (camera.theta - half_fov).cos(),
        y: (camera.theta - half_fov).sin()
    };

    ( p0.plus(&p0_vec), p1.plus(&p1_vec) )
}

fn get_ray_vecs(camera: &Camera) -> Vec<Vec2> {
    let (p0, p1) = get_draw_plane_points(camera);

    let dist = p0.distance(&p1);
    let pixel_dist = dist / camera.width;

    let distance_vec = Vec2 {
        x: (p1.x - p0.x) / dist,
        y: (p1.y - p0.y) / dist,
    };

    let offset = p0.plus(&distance_vec.multiply(pixel_dist / 2.0));

    let mut ray_vecs = Vec::new();
    for i in 0..camera.width as i32 {
        let ray_vec = offset.plus(&distance_vec.multiply(pixel_dist * i as f64));
        ray_vecs.push(ray_vec);
    }

    ray_vecs
}

fn get_distance_to_ray_line_intersection(p0: &Vec2, v0: &Vec2, p1: &Vec2, p2: &Vec2) -> Option<f64> {
    let v1 = p2.minus(&p1);

    let v0_cross_v1 = v0.cross(&v1);

    //    println!("p0 = {:?}, v0 = {:?}, p1 = {:?}, v1 = {:?}", p0, v0, p1, v1);
    if v0_cross_v1 == 0.0 {
        //    if (v0_cross_v1 * 100000.0).round() / 100000.0 == 0.0 {
        // rounding to deal with the fact that our angles aren't perfect due to input-radians-accuracy
        // Segments are parallel / co-linear
        // in our case we don't care about co-linear collisions
        return None;
    }

    let p1_minus_p0 = p1.minus(&p0);

    let s0 = p1_minus_p0.cross(&v1) / v0_cross_v1;
    let s1 = p1_minus_p0.cross(&v0) / v0_cross_v1;

    if s0 >= 0.0 && s1 <= 1.0 && s1 >= 0.0 {
        // because v0 is of 1 distance,
        // s0 = distance to collision
        //        println!("{:?}", p0.plus(&v0.multiply(s0)));
        Some(s0)
    } else {
        None
    }
}

fn load_world_from_json_argv() -> World {
    let file_name = std::env::args().nth(1).expect("1st ARGV must be the desired input file");
    let mut file_as_string = String::new();
    File::open(file_name).expect("The file path must be a valid, open-able file")
        .read_to_string(&mut file_as_string).expect("The file must be valid utf-8");

    let json: WorldJSON = json::decode(&file_as_string).expect("File must be a valid input.json file");

    json.into()
}

fn vec_to_color(color_vec: Vec<u8>) -> Color {
    Color::RGB(
        color_vec[0],
        color_vec[1],
        color_vec[2],
    )
}

#[derive(RustcDecodable, Debug)]
struct WorldJSON {
    walls: Vec<WallJSON>,
    ground_color: Vec<u8>,
    sky_color: Vec<u8>,
    camera: CameraJSON
}

#[derive(Debug)]
struct World {
    walls: Vec<Wall>,
    ground_color: Color,
    sky_color: Color,
    camera: Camera
}

impl From<WorldJSON> for World {
    fn from(json: WorldJSON) -> Self {

        let mut walls: Vec<Wall> = Vec::new();
        for wall in json.walls {
            walls.push(wall.into())
        }

        World {
            walls: walls,
            ground_color: vec_to_color(json.ground_color),
            sky_color: vec_to_color(json.sky_color),
            camera: json.camera.into()
        }
    }
}

#[derive(RustcDecodable, Debug)]
struct WallJSON {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    color: Vec<u8>
}

#[derive(Debug, Clone)]
struct Wall {
    p0: Vec2,
    p1: Vec2,
    color: Color,
}

impl From<WallJSON> for Wall {
    fn from(json: WallJSON) -> Self {
        Wall{
            p0: Vec2 { x: json.x0, y: json.y0 },
            p1: Vec2 { x: json.x1, y: json.y1 },
            color: vec_to_color(json.color)
        }
    }
}

#[derive(RustcDecodable, Debug)]
struct CameraJSON {
    x: f64,
    y: f64,
    theta: f64,
    h_fov: f64,
    width: f64,
    height: f64,
}

#[derive(Debug)]
struct Camera {
    p0: Vec2,
    theta: f64,
    h_fov: f64,
    width: f64,
    height: f64,
    horizon: i32,
}

impl From<CameraJSON> for Camera {
    fn from(json: CameraJSON) -> Self {
        let horizon = if json.width % 2.0 == 0.0 {
            json.height/ 2.0
        } else {
            (json.height / 2.0).round()
        };

        Camera {
            p0: Vec2 { x: json.x, y: json.y },
            theta: json.theta,
            h_fov: json.h_fov,
            width: json.width,
            height: json.height,
            horizon: horizon as i32,
        }
    }
}