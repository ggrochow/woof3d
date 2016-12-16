extern crate rustc_serialize;
extern crate sdl2;

mod vec2;

use vec2::Vec2;
use sdl2::pixels::Color;
use rustc_serialize::json;
use std::io::prelude::*;
use std::fs::File;


fn main() {
    /********
        Plans
        each step stays as seperate as possible
        try to get a single input to pass into the next part

        1) Setup
            Load scene
            Open window
            Setup eventpump,etc

        2) Game Loop
            every 1/60th, check what inputs are pressed.
            based on what keys are down, attempt to apply those transformations to the game-scene

        3) Draw
            Takes a clone of the game-state, and draws it to screen.
    ********/


    /********
     ** ONLY STEP THAT CAN PANIC **

        1) Setup
            Load in.json, defines camera-angle, fov, h/w + walls
            start the event-pump, open window.
            start threads, and move things to wherever is needed.
    ********/

    let mut world = load_world_from_json_argv();
    println!("world = {:#?}", world);


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


    /********
        3) Draw
            take input, draw to screen.
            clear, draw top half sky, bottom half ground
            1 draw call for each wall.
    ********/
}

fn load_world_from_json_argv() -> World {
    let file_name = std::env::args().nth(1).expect("1st ARGV must be the desired input file");
    let mut file_as_string = String::new();
    let mut file = File::open(file_name).expect("The file path must be a valid, open-able file")
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

#[derive(Debug)]
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
    width: u32,
    height: u32,
}

#[derive(Debug)]
struct Camera {
    p0: Vec2,
    theta: f64,
    h_fov: f64,
    width: u32,
    height: u32
}

impl From<CameraJSON> for Camera {
    fn from(json: CameraJSON) -> Self {
        Camera {
            p0: Vec2 { x: json.x, y: json.y },
            theta: json.theta,
            h_fov: json.h_fov,
            width: json.width,
            height: json.height
        }
    }
}