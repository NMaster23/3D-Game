use macroquad::prelude::*;
// use glam::vec3;

const MOVE_SPEED: f32 = 0.1;
const LOOK_SPEED: f32 = 0.1;

fn conf() -> Conf {
    Conf {
        window_title: String::from("Macroquad"),
        window_width: 1260,
        window_height: 768,
        fullscreen: false,
        ..Default::default()
    }
}

fn color_from(hex: u32) -> Color {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    let output = Color::new(r, g, b, 1.0);
    return output;
}

#[macroquad::main(conf)]
async fn main() {
    let sky_color = color_from(0x87CEEB);
    let _delta = get_frame_time();
    let rust_logo = load_texture("OIP (1).png").await.unwrap();
    let plane_texture = load_texture("Plane.png").await.unwrap();
    let grass_texture = load_texture("Grass.png").await.unwrap();
    let mut x = 0.0;
    let mut switch = false;
    let bounds = 8.0;

    let world_up = vec3(0.0, 1.0, 0.0);
    let mut yaw: f32 = 1.18;
    let mut pitch: f32 = 0.0;

    let mut front = vec3(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    )
    .normalize();
    let mut right = front.cross(world_up).normalize();
    let mut up = right.cross(front).normalize();

    let mut position = vec3(0.0, 1.0, 0.0);
    let mut is_jumping = false;
    let mut jump_velocity = 0.0;
    let mut last_mouse_position: Vec2 = mouse_position().into();

    let mut grabbed = true;
    set_cursor_grab(grabbed);
    show_mouse(false);

    loop {
        let delta = get_frame_time();

        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::Tab) {
            grabbed = !grabbed;
            set_cursor_grab(grabbed);
            show_mouse(!grabbed);
        }

        if is_key_down(KeyCode::Up) {
            position += front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::Down) {
            position -= front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::Left) {
            position -= right * MOVE_SPEED;
        }
        if is_key_down(KeyCode::Right) {
            position += right * MOVE_SPEED;
        }
        if is_key_down(KeyCode::W) {
            position += front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::S) {
            position -= front * MOVE_SPEED;
        }
        if is_key_down(KeyCode::A) {
            position -= right * MOVE_SPEED;
        }
        if is_key_down(KeyCode::D) {
            position += right * MOVE_SPEED;
        }
        if is_key_pressed(KeyCode::Space) && !is_jumping {
            is_jumping = true;
            jump_velocity = 0.25;
        }
        if is_mouse_button_down(MouseButton::Left) {
            println!("Left mouse button is down!");
        }
        if is_key_down(KeyCode::LeftShift) {
            position += front * MOVE_SPEED * 1.5;
        }
        if is_key_down(KeyCode::LeftControl) {
            position -= front * MOVE_SPEED * 0.5;
            position.y = 0.5;
        }
        if is_jumping {
            position.y += jump_velocity;
            jump_velocity -= 0.01;
            if position.y <= 1.0 {
                position.y = 1.0;
                is_jumping = false;
            }
        }
        let mouse_position: Vec2 = mouse_position().into();
        let mouse_delta = mouse_position - last_mouse_position;

        last_mouse_position = mouse_position;
        if grabbed {
            yaw += mouse_delta.x * delta * LOOK_SPEED;
            pitch += mouse_delta.y * delta * -LOOK_SPEED;

            pitch = if pitch > 1.5 { 1.5 } else { pitch };
            pitch = if pitch < -1.5 { -1.5 } else { pitch };

            front = vec3(
                yaw.cos() * pitch.cos(),
                pitch.sin(),
                yaw.sin() * pitch.cos(),
            )
            .normalize();
            right = front.cross(world_up).normalize();
            up = right.cross(front).normalize();

            x += if switch { 0.04 } else { -0.04 };
            if x >= bounds || x <= -bounds {
                switch = !switch;
            }
        }
        if position.y <= 1.0 && !is_key_down(KeyCode::LeftControl) {
            position.y = 1.0;
        }
        if !is_jumping && !is_key_down(KeyCode::LeftControl) {
            position.y = 1.0;
        }
        clear_background(sky_color);
        // Going 3d!

        set_camera(&Camera3D {
            position: position,
            up: up,
            target: position + front,
            ..Default::default()
        });


        draw_line_3d(
            vec3(x, 0.0, x),
            vec3(5.0, 5.0, 5.0),
            Color::new(1.0, 1.0, 0.0, 1.0),
        );
        draw_cube(vec3(0., 1., -6.), vec3(2., 2., 2.), Some(&rust_logo), GREEN);
        draw_cube(vec3(0., 1., 6.), vec3(2., 2., 2.), Some(&rust_logo), BLUE);
        draw_cube(vec3(2., 1., 2.), vec3(2., 2., 2.), Some(&rust_logo), RED);
        draw_plane(vec3(-8., 0., -8.), vec2(25., 25.), Some(&grass_texture), GREEN);

        // Back to screen space, render some text

        set_default_camera();
        draw_text("First Person Camera", 10.0, 20.0, 30.0, BLACK);

        draw_text(
            format!("X: {} Y: {} Z: {}", position.x, position.y, position.z).as_str(),
            10.0,
            48.0 + 18.0,
            30.0,
            BLACK,
        );
        draw_text(
            format!("Press <TAB> to toggle mouse grab: {grabbed}").as_str(),
            10.0,
            48.0 + 42.0,
            30.0,
            BLACK,
        );
        next_frame().await
    }
}