use crate::{
    input::{Input, Key}, 
    math::{Vec2, Vec3, Quat},
    render3d::{Camera},
    time::{Time},
};

pub fn camera_debug_move(
    camera: &mut Camera,
    input: &Input,
    time: &Time,
) {
    let move_speed = 5.0 * time.delta_seconds();
    let rotate_speed = 2.0 * time.delta_seconds();

    let mut movement = Vec3::ZERO;
    let mut rotation = Vec3::ZERO;

    if input.key_down(Key::A) {
        movement.x -= 1.0;
    }

    if input.key_down(Key::D) {
        movement.x += 1.0;
    }

    if input.key_down(Key::W) {
        movement.z += 1.0;
    }

    if input.key_down(Key::S) {
        movement.z -= 1.0;
    }

    if input.key_down(Key::R) {
        movement.y += 1.0;
    }

    if input.key_down(Key::F) {
        movement.y -= 1.0;
    }

    if input.key_down(Key::Left) {
        rotation.y -= 1.0;
    }

    if input.key_down(Key::Right) {
        rotation.y += 1.0;
    }

    if input.key_down(Key::Up) {
        rotation.x -= 1.0;
    }

    if input.key_down(Key::Down) {
        rotation.x += 1.0;
    }

    if input.key_down(Key::Q) {
        rotation.z += 1.0;
    }

    if input.key_down(Key::E) {
        rotation.z -= 1.0;
    }

    if movement.length_squared() > 0.0 {
        movement = movement.normalized();
    }

    let world_movement =
        camera.rotation.rotate_vec3(movement);

    camera.position += world_movement * move_speed;

    let pitch = Quat::from_axis_angle(
        Vec3::new(1.0, 0.0, 0.0),
        rotation.x * rotate_speed,
    );

    let yaw = Quat::from_axis_angle(
        Vec3::new(0.0, 1.0, 0.0),
        rotation.y * rotate_speed,
    );

    let roll = Quat::from_axis_angle(
        Vec3::new(0.0, 0.0, 1.0),
        rotation.z * rotate_speed,
    );

    camera.rotation *= yaw;
    camera.rotation *= pitch;
    camera.rotation *= roll;

    camera.rotation = camera.rotation.normalized();
}