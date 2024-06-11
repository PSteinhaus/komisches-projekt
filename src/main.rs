use macroquad::prelude::*;

const WORLD_WIDTH: f32 = 2480.;
const WORLD_HEIGHT: f32 = 3508.;

enum ButtonState {
    Idle,
    Hovered,
    Pressed,
    Released,
}

struct Button<'a> {
    pub texture: &'a Texture2D,
    pub dest: Rect,
    state: ButtonState,
}

impl Button<'_> {
    pub fn new(texture: &Texture2D, dest: Rect) -> Button {
        Button {
            texture,
            dest,
            state: ButtonState::Idle,
        }
    }

    fn update_button_state(&mut self, camera: &Camera2D) {
        // start off pressed or idle, depending on whether you've been pressed in the previous frame
        let pressed_before = matches!(self.state, ButtonState::Pressed);
        let mut new_state = if pressed_before {
            ButtonState::Pressed
        } else {
            ButtonState::Idle
        };

        // first get the mouse state and whether it's above you
        let mouse_pos = mouse_world_pos(camera);
        let mouse_pressed = macroquad::input::is_mouse_button_down(MouseButton::Left);
        if self.dest.contains(mouse_pos) {
            if !pressed_before {
                new_state = ButtonState::Hovered;
            }
            // if you have been pressed check for release
            else if !mouse_pressed {
                // you have been pressed down and have now been released (while over your actual position)
                new_state = ButtonState::Released;
            }
            // if you have been pressed, but weren't released just stay pressed (see start of function)
        } else {
            // lastly make sure that releasing the button elsewhere ALSO resets your "pressed" state
            if !mouse_pressed {
                new_state = ButtonState::Idle;
            }
        }
        self.state = new_state;
    }

    /// React to mouse input, draw the button accordingly and return whether the button was clicked.
    ///
    /// Draws the button differently when hovered, not hovered, and pressed down.
    pub fn draw_and_get_clicked(&mut self, camera: &Camera2D) -> bool {
        self.update_button_state(camera);

        use ButtonState::*;
        let color = match self.state {
            Idle => Color::new(0.8, 0.8, 0.8, 1.),
            Hovered | Released => WHITE,
            Pressed => Color::new(0.4, 0.4, 0.4, 1.),
        };
        let clicked = matches!(self.state, Released);

        draw_texture_ex(
            self.texture,
            self.dest.x,
            self.dest.y,
            color,
            DrawTextureParams {
                dest_size: Some(Vec2::new(self.dest.w, self.dest.h)),
                ..Default::default()
            },
        );

        clicked
    }
}

fn mouse_world_pos(camera: &Camera2D) -> Vec2 {
    let mouse_screen_pos = Vec2::from(macroquad::input::mouse_position());
    camera.screen_to_world(mouse_screen_pos)
}

fn get_window_conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        // I just like it when things are blurry...
        high_dpi: true,
        window_width: (WORLD_WIDTH / 6.) as i32,
        window_height: (WORLD_HEIGHT / 6.) as i32,
        ..Default::default()
    }
}

use futures::future::try_join_all;
use smallvec::SmallVec;
async fn load_textures() -> Vec<Texture2D> {
    let file_paths: SmallVec<[String; 36]> = (1..5).map(|i| (i.to_string() + ".png")).collect();
    let loaded_textures = try_join_all(file_paths.iter().map(|path| load_texture(path))).await;
    loaded_textures.unwrap()
}

#[macroquad::main(get_window_conf)]
async fn main() {
    let textures: Vec<Texture2D> = load_textures().await;

    let mut cam = Camera2D::from_display_rect(Rect::new(0., 0., WORLD_WIDTH, WORLD_HEIGHT));
    cam.zoom = Vec2::new(cam.zoom.x, -cam.zoom.y); // workaround for https://github.com/not-fl3/macroquad/issues/171

    // TODO: create buttons!

    loop {
        clear_background(Color::default());

        set_camera(&cam);

        let params = DrawTextureParams {
            dest_size: Some(Vec2::new(WORLD_WIDTH, WORLD_HEIGHT)),
            ..Default::default()
        };

        draw_texture_ex(&textures[0], 0., 0., WHITE, params);

        set_default_camera();

        draw_text(format!("FPS: {}", get_fps()).as_str(), 0., 16., 32., WHITE);
        draw_text(
            format!("width: {}", screen_width()).as_str(),
            0.,
            32.,
            32.,
            WHITE,
        );
        draw_text(
            format!("height: {}", screen_height()).as_str(),
            0.,
            48.,
            32.,
            WHITE,
        );

        next_frame().await
    }
}
