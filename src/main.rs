extern crate rustc_serialize;
extern crate sdl2;
extern crate rand;

mod vec2;
mod world;
mod maze;

use vec2::Vec2;
use sdl2::pixels::Color;
use std::collections::HashSet;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Renderer;
use sdl2::rect::Rect;
use std::time::Duration;
use world::{World, Wall, Camera};

fn main() {

    /********
     ** ONLY STEP THAT CAN PANIC **

        1) Setup
            generate maze, convert to points
            start the event-pump, open window.
            start threads, and move things to wherever is needed.
    ********/

    let mut world = World::default();
    let maze = maze::Maze::generate_maze(10, 10);
    world.walls = maze.to_walls(1);
    println!("{}\nEnjoy your maze!\nYou start in the top-left", maze);
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

        GameObject
            walls,
            camera { x, y, angle, fov, height, width },
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
                Keycode::A => world.camera.theta += 0.02,
                Keycode::D => world.camera.theta -= 0.02,
                Keycode::W => {
                    world.move_forward();
                },
                Keycode::Z => {
                    world.camera.horizon += 10;
                },
                Keycode::X => {
                    world.camera.horizon -= 10;
                }
                _ => {},
            }
        }

    /******
     
     3) Presentation
        Draw game-world to screen
        sleep a little to limit FPS

    ******/

        draw_world(&mut renderer, &world);
        std::thread::sleep(Duration::from_millis(10));
    }


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
    // TODO: bugfix - it is possible for v0 + v1 to the same
    // easily reproduce by setting camera-default pos to 1,1
    // probably just draw it at full height, though we shoudln't really ever let this happen
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
    // TODO: refactor this with line-line intersection
    // we can use the same function up until we have to check s0/s1
    // so we can just return those and save duplicate
    let v1 = p2.minus(&p1);

    let v0_cross_v1 = v0.cross(&v1);

    if v0_cross_v1 == 0.0 {
        return None;
    }

    let p1_minus_p0 = p1.minus(&p0);

    let s0 = p1_minus_p0.cross(&v1) / v0_cross_v1;
    let s1 = p1_minus_p0.cross(&v0) / v0_cross_v1;

    if s0 >= 0.0 && s1 <= 1.0 && s1 >= 0.0 {
        Some(s0)
    } else {
        None
    }
}


pub fn get_distance_to_line_line_intersection(p0: &Vec2, v0: &Vec2, p1: &Vec2, p2: &Vec2) -> Option<f64> {
    let v1 = p2.minus(&p1);

    let v0_cross_v1 = v0.cross(&v1);

    if v0_cross_v1 == 0.0 {
        return None;
    }

    let p1_minus_p0 = p1.minus(&p0);

    let s0 = p1_minus_p0.cross(&v1) / v0_cross_v1;
    let s1 = p1_minus_p0.cross(&v0) / v0_cross_v1;

    if s0 <= 1.0 && s0 >= 0.0 && s1 <= 1.0 && s1 >= 0.0 {
        Some(s0)
    } else {
        None
    }
}
