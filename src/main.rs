use macroquad::prelude::*;

const WORLD_WIDTH: f32 = 2480.;
const WORLD_HEIGHT: f32 = 3508.;
const WORLD_STATE_VARIANTS: usize = 19;
const ASSET_PATH: &'static str = "assets/";

#[derive(Copy, Clone)]
enum WorldState {
    Egg,
    EggCrack1,
    EggCrack2,
    Chick,
    Duckling,
    Duck,
    Bird,
    Heron,
    BabyTurtle,
    Salamander,
    Dragonmander,
    Turtle,
    TurtleWizard,
    BigEgg,
    BigEggCrack1,
    BigEggCrack2,
    SmallDragon,
    Nessi,
    Kraken,
    Jellyfish,
}

struct World {
    buttons: [Button; 3],
    state_textures: Vec<Texture2D>,
    // state machine
    state: WorldState,
    transition: Option<Transition>,
}

use smallvec::SmallVec;
impl World {
    async fn load_textures() -> Vec<Texture2D> {
        let file_paths: SmallVec<[String; WORLD_STATE_VARIANTS]> = (0..WORLD_STATE_VARIANTS)
            .map(|i| (ASSET_PATH.to_string() + i.to_string().as_str() + ".png"))
            .collect();
        let loaded_textures =
            futures::future::try_join_all(file_paths.iter().map(|path| load_texture(path))).await;
        loaded_textures.unwrap()
    }

    pub async fn new() -> Self {
        Self {
            buttons: Button::create().await,
            state_textures: Self::load_textures().await,
            state: WorldState::Egg,
            transition: None,
        }
    }

    pub fn handle_input(&mut self, cam: &Camera2D) {
        let mut clicked_button = None;
        for button in self.buttons.iter_mut() {
            if button.disabled {
                continue;
            }
            let clicked = button.update_button_state(cam);
            // TODO: handle clicked (by triggering a WorldState transistion and removing the button)
            if clicked {
                clicked_button = Some(button.b_type);
                button.disable();
            }
        }
        if let Some(b_type) = clicked_button {
            self.start_transition(b_type);
        }
    }

    pub fn progress(&mut self, delta_secs: f32) {
        // progress the transition, if there is one
        if let Some(t) = self.transition.as_mut() {
            let next_transition = t.progress(delta_secs);
            if t.completed() {
                self.transition = next_transition;
            }
        }
    }

    fn texture_for_state(&self, state: WorldState) -> &Texture2D {
        &self.state_textures[state as usize]
    }

    /// draws the main image and after that the buttons
    pub fn render(&self, cam: &Camera2D) {
        let params = DrawTextureParams {
            dest_size: Some(Vec2::new(WORLD_WIDTH, WORLD_HEIGHT)),
            ..Default::default()
        };
        // in case of a transition draw both images with their respecting alpha according to the transition
        if let Some(ref t) = self.transition {
            let (color_current, color_next) = t.colors();
            draw_texture_ex(
                self.texture_for_state(self.state),
                0.,
                0.,
                color_current,
                params.clone(),
            );
            draw_texture_ex(
                self.texture_for_state(t.goal_state),
                0.,
                0.,
                color_next,
                params,
            );
        } else {
            draw_texture_ex(self.texture_for_state(self.state), 0., 0., WHITE, params);
            for button in self.buttons.iter() {
                button.draw(cam);
            }
        }
    }

    fn start_transition(&mut self, b_type: ButtonType) {
        use WorldState::*;
        // compute the target
        let goal_state = match self.state {
            Egg => EggCrack1,
            EggCrack1 => panic!("started transition in egg crack!"),
            EggCrack2 => panic!("started transition in egg crack!"),
            Chick => match b_type {
                ButtonType::Sun => panic!("sun no longer available!"),
                ButtonType::Water => Duckling,
                ButtonType::Arrowhead => Bird,
            },
            Duckling => match b_type {
                ButtonType::Sun => panic!("sun no longer available!"),
                ButtonType::Water => panic!("water no longer available!"),
                ButtonType::Arrowhead => Duck,
            },
            Duck => panic!("no buttons available!"),
            Bird => match b_type {
                ButtonType::Sun => panic!("sun no longer available!"),
                ButtonType::Water => Heron,
                ButtonType::Arrowhead => panic!("arrow no longer available!"),
            },
            Heron => panic!("no buttons available!"),
            BabyTurtle => match b_type {
                ButtonType::Sun => Salamander,
                ButtonType::Water => panic!("water no longer available!"),
                ButtonType::Arrowhead => Turtle,
            },
            Salamander => match b_type {
                ButtonType::Sun => panic!("sun no longer available!"),
                ButtonType::Water => panic!("water no longer available!"),
                ButtonType::Arrowhead => Dragonmander,
            },
            Dragonmander => panic!("no buttons available!"),
            Turtle => match b_type {
                ButtonType::Sun => TurtleWizard,
                ButtonType::Water => panic!("water no longer available!"),
                ButtonType::Arrowhead => panic!("arrow no longer available!"),
            },
            TurtleWizard => panic!("no buttons available!"),
            BigEgg => BigEggCrack1,
            BigEggCrack1 => panic!("started transition in egg crack!"),
            BigEggCrack2 => panic!("started transition in egg crack!"),
            SmallDragon => match b_type {
                ButtonType::Sun => panic!("sun no longer available!"),
                ButtonType::Water => Nessi,
                ButtonType::Arrowhead => panic!("arrow no longer available!"),
            },
            Nessi => panic!("no buttons available!"),
            Kraken => match b_type {
                ButtonType::Sun => Jellyfish,
                ButtonType::Water => panic!("water no longer available!"),
                ButtonType::Arrowhead => panic!("arrow no longer available!"),
            },
            Jellyfish => panic!("no buttons available!"),
        };
        // start the new transition
        let t_type = match goal_state {
            EggCrack1 | BigEggCrack1 => TransitionType::EggCracking(b_type),
            _ => TransitionType::Regular,
        };
        let new_transition = Transition::new(goal_state, t_type);
        self.transition = Some(new_transition);
    }
}

/// used to differentiate the two kinds of transitions existing, but also the two sounds in the game
#[derive(Clone, Copy)]
enum TransitionType {
    Regular,
    EggCracking(ButtonType),
}

struct Transition {
    goal_state: WorldState,
    t_type: TransitionType,
    time_progressed: f32,
}

impl Transition {
    pub fn new(goal_state: WorldState, t_type: TransitionType) -> Self {
        Self {
            goal_state,
            t_type,
            time_progressed: 0.,
        }
    }

    /// Progresses the transition and returns None, except if there is a subsequent transition that it continues into.
    /// In that case it starts that transition with the leftover time and returns it.
    pub fn progress(&mut self, delta_time: f32) -> Option<Transition> {
        self.time_progressed += delta_time;
        let total = self.total_duration();
        if self.time_progressed > total {
            let leftover_delta = self.time_progressed - total;
            self.time_progressed = total;
            // the following builds on the assumption that the leftover_delta is not enough to complete the subsequent transition too,
            // which it won't looking at how long transitions are taking in this toy program
            let mut subsequent = self.subsequent_transition();
            if let Some(ref mut t) = subsequent {
                t.progress(leftover_delta);
            }
            return subsequent;
        }
        None
    }

    fn total_duration(&self) -> f32 {
        use TransitionType::*;
        match self.t_type {
            Regular => 5.,
            EggCracking(_) => 9.,
        }
    }

    pub fn colors(&self) -> (Color, Color) {
        let total = self.total_duration();
        let relative_progress = self.time_progressed / total;
        let color_current_alpha = 1. - clamp(relative_progress * 2.2, 0., 1.).powf(1.4);
        let color_next_alpha = clamp((relative_progress - 0.5) * 2.2, 0., 1.).powf(1.4);

        let color_current = Color {
            r: 1.,
            g: 1.,
            b: 1.,
            a: color_current_alpha,
        };
        let color_next = Color {
            r: 1.,
            g: 1.,
            b: 1.,
            a: color_next_alpha,
        };

        (color_current, color_next)
    }

    pub fn sound_to_play(&self) -> Option<TransitionType> {
        None //TODO: add sounds!
    }

    pub fn completed(&self) -> bool {
        self.time_progressed >= self.total_duration()
    }

    /// a subsequent transition only exists for egg crack transitions, which start another crack,
    /// or a regular transition to whatever hatches
    pub fn subsequent_transition(&self) -> Option<Transition> {
        use WorldState::*;
        match self.goal_state {
            EggCrack1 => Some(Transition::new(EggCrack2, self.t_type.clone())),
            EggCrack2 => Some(Transition::new(
                match self.t_type {
                    TransitionType::EggCracking(b_type) => match b_type {
                        ButtonType::Sun => Chick,
                        ButtonType::Water => BabyTurtle,
                        ButtonType::Arrowhead => panic!("arrow in egg?"),
                    },
                    TransitionType::Regular => panic!("transition to EggCrack2 was Regular?"),
                },
                TransitionType::Regular,
            )),
            BigEggCrack1 => Some(Transition::new(BigEggCrack2, self.t_type.clone())),
            BigEggCrack2 => Some(Transition::new(
                match self.t_type {
                    TransitionType::EggCracking(b_type) => match b_type {
                        ButtonType::Sun => SmallDragon,
                        ButtonType::Water => Kraken,
                        ButtonType::Arrowhead => panic!("arrow in big egg?"),
                    },
                    TransitionType::Regular => panic!("transition to BigEggCrack2 was Regular?"),
                },
                TransitionType::Regular,
            )),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
enum ButtonType {
    Sun,
    Water,
    Arrowhead,
}

#[derive(Clone, Copy)]
enum ButtonState {
    Idle,
    Hovered,
    Pressed,
    Released,
}

struct Button {
    pub b_type: ButtonType,
    pub texture: Texture2D,
    pub dest: Rect,
    pub disabled: bool,
    state: ButtonState,
}

impl Button {
    pub async fn create() -> [Button; 3] {
        // let button_textures = futures::future::try_join_all([
        //     load_texture((ASSET_PATH.to_string() + "button_sun.png").as_str()),
        //     load_texture((ASSET_PATH.to_string() + "button_water.png").as_str()),
        //     load_texture((ASSET_PATH.to_string() + "button_arrow.png").as_str()),
        // ]).await.unwrap();

        [
            Button::new(
                ButtonType::Sun,
                load_texture((ASSET_PATH.to_string() + "button_sun.png").as_str())
                    .await
                    .unwrap(),
                Rect::new(20., 2700., 600., 600.),
            ),
            Button::new(
                ButtonType::Water,
                load_texture((ASSET_PATH.to_string() + "button_water.png").as_str())
                    .await
                    .unwrap(),
                Rect::new(700., 2700., 600., 600.),
            ),
            Button::new(
                ButtonType::Arrowhead,
                load_texture((ASSET_PATH.to_string() + "button_arrow.png").as_str())
                    .await
                    .unwrap(),
                Rect::new(1420., 2700., 600., 600.),
            ),
        ]
    }

    fn new(b_type: ButtonType, texture: Texture2D, dest: Rect) -> Button {
        Button {
            b_type,
            texture,
            dest,
            disabled: false,
            state: ButtonState::Idle,
        }
    }

    /// updates the buttons internal state depending on the mouse and returns whether the button was clicked
    pub fn update_button_state(&mut self, camera: &Camera2D) -> bool {
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
        let clicked = matches!(self.state, ButtonState::Released);
        clicked
    }

    /// React to mouse input, draw the button accordingly and return whether the button was clicked.
    ///
    /// Draws the button differently when hovered, not hovered, and pressed down.
    pub fn draw(&self, camera: &Camera2D) {
        if self.disabled {
            return;
        }

        use ButtonState::*;
        let color = match self.state {
            Idle => Color::new(0.8, 0.8, 0.8, 1.),
            Hovered | Released => WHITE,
            Pressed => Color::new(0.4, 0.4, 0.4, 1.),
        };

        draw_texture_ex(
            &self.texture,
            self.dest.x,
            self.dest.y,
            color,
            DrawTextureParams {
                dest_size: Some(Vec2::new(self.dest.w, self.dest.h)),
                ..Default::default()
            },
        );
    }

    pub fn disable(&mut self) {
        self.disabled = true;
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

#[macroquad::main(get_window_conf)]
async fn main() {
    // start of with a loading screen
    let mut cam = Camera2D::from_display_rect(Rect::new(0., 0., WORLD_WIDTH, WORLD_HEIGHT));
    cam.zoom = Vec2::new(cam.zoom.x, -cam.zoom.y); // workaround for https://github.com/not-fl3/macroquad/issues/171
    set_camera(&cam);

    clear_background(Color::default());
    draw_text("Loading...", 760., 1600., 200., WHITE);

    next_frame().await;

    let world = World::new().await;

    // TODO: create buttons!

    loop {
        clear_background(Color::default());

        set_camera(&cam);

        world.render(&cam);

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
