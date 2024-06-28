use std::f32::consts::PI;

use collections::storage;
use coroutines::start_coroutine;
use macroquad::{
    audio::{self, play_sound_once, PlaySoundParams, Sound},
    prelude::*,
};

const WORLD_WIDTH: f32 = 2480.;
const WORLD_HEIGHT: f32 = 3508.;
const WORLD_STATE_VARIANTS: usize = 20;
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

#[derive(Clone, Copy)]
enum SoundIndex {
    Crack1,
    Crack2,
    Scale1,
    Scale2,
}

struct World {
    buttons: [Button; 4],
    state_textures: Vec<Texture2D>,
    sounds: [Sound; 4],
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

    async fn load_sounds() -> [Sound; 4] {
        [
            audio::load_sound((ASSET_PATH.to_string() + "crack1.mp3").as_str())
                .await
                .unwrap(),
            audio::load_sound((ASSET_PATH.to_string() + "crack2.mp3").as_str())
                .await
                .unwrap(),
            audio::load_sound((ASSET_PATH.to_string() + "scale-d6.mp3").as_str())
                .await
                .unwrap(),
            audio::load_sound((ASSET_PATH.to_string() + "scale-e6.mp3").as_str())
                .await
                .unwrap(),
        ]
    }

    pub async fn new() -> Self {
        Self {
            buttons: Button::create().await,
            state_textures: Self::load_textures().await,
            sounds: Self::load_sounds().await,
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
        if let Some(mut t) = self.transition.take() {
            let next_transition = t.progress(delta_secs);
            if let Some(sound_index) = t.sound_to_play() {
                self.play_sound(sound_index);
            }
            if t.completed() {
                self.finish_transition(&t, next_transition);
            } else {
                // its slightly weird to but back the transition, but who knows, maybe the compiler is smart enough to make this free, maybe not
                self.transition = Some(t);
            }
        }
    }

    fn play_sound(&self, sound_index: SoundIndex) {
        use SoundIndex::*;
        let volume = match sound_index {
            Scale1 | Scale2 => 0.7,
            Crack1 | Crack2 => 1.1,
        };
        macroquad::audio::play_sound(
            &self.sounds[sound_index as usize],
            PlaySoundParams {
                looped: false,
                volume,
            },
        );
    }

    /// Some transitions require a final action, such as the restart or enabling the restart button
    fn finish_transition(&mut self, t: &Transition, next_transition: Option<Transition>) {
        use WorldState::*;
        match t.goal_state {
            Egg => self.init_buttons(),
            Duck | Heron | Dragonmander | TurtleWizard | Nessi | Jellyfish => {
                self.buttons[3].disabled = false;
            }
            _ => {}
        };
        self.state = t.goal_state;

        // this whole process of continuing from one transition into the next is dirty, but for what I'm doing now it works
        if let Some(ref new_t) = next_transition {
            if let Some(sound_index) = new_t.sound_to_play() {
                play_sound_once(&self.sounds[sound_index as usize]);
            }
        }
        self.transition = next_transition;
    }

    fn texture_for_state(&self, state: WorldState) -> &Texture2D {
        &self.state_textures[state as usize]
    }

    /// draws the main image and after that the buttons
    pub fn render(&self) {
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
                button.draw();
            }
        }
    }

    fn start_transition(&mut self, b_type: ButtonType) {
        use WorldState::*;
        // compute the target
        let goal_state = match self.state {
            Egg => match b_type {
                ButtonType::Sun | ButtonType::Water => EggCrack1,
                ButtonType::Arrowhead => BigEgg,
                _ => Egg,
            },
            EggCrack1 => panic!("started transition in egg crack!"),
            EggCrack2 => panic!("started transition in egg crack!"),
            Chick => match b_type {
                ButtonType::Sun => panic!("sun no longer available!"),
                ButtonType::Water => Duckling,
                ButtonType::Arrowhead => Bird,
                _ => Egg,
            },
            Duckling => match b_type {
                ButtonType::Sun => panic!("sun no longer available!"),
                ButtonType::Water => panic!("water no longer available!"),
                ButtonType::Arrowhead => Duck,
                _ => Egg,
            },
            Duck => Egg,
            Bird => match b_type {
                ButtonType::Sun => panic!("sun no longer available!"),
                ButtonType::Water => Heron,
                ButtonType::Arrowhead => panic!("arrow no longer available!"),
                _ => Egg,
            },
            Heron => Egg,
            BabyTurtle => match b_type {
                ButtonType::Sun => Salamander,
                ButtonType::Water => panic!("water no longer available!"),
                ButtonType::Arrowhead => Turtle,
                _ => Egg,
            },
            Salamander => match b_type {
                ButtonType::Sun => panic!("sun no longer available!"),
                ButtonType::Water => panic!("water no longer available!"),
                ButtonType::Arrowhead => Dragonmander,
                _ => Egg,
            },
            Dragonmander => Egg,
            Turtle => match b_type {
                ButtonType::Sun => TurtleWizard,
                ButtonType::Water => panic!("water no longer available!"),
                ButtonType::Arrowhead => panic!("arrow no longer available!"),
                _ => Egg,
            },
            TurtleWizard => Egg,
            BigEgg => BigEggCrack1,
            BigEggCrack1 => panic!("started transition in egg crack!"),
            BigEggCrack2 => panic!("started transition in egg crack!"),
            SmallDragon => match b_type {
                ButtonType::Sun => panic!("sun no longer available!"),
                ButtonType::Water => Nessi,
                ButtonType::Arrowhead => panic!("arrow no longer available!"),
                _ => Egg,
            },
            Nessi => Egg,
            Kraken => match b_type {
                ButtonType::Sun => Jellyfish,
                ButtonType::Water => panic!("water no longer available!"),
                ButtonType::Arrowhead => panic!("arrow no longer available!"),
                _ => Egg,
            },
            Jellyfish => Egg,
        };
        // start the new transition
        let t_type = match goal_state {
            EggCrack1 | BigEggCrack1 => TransitionType::EggCracking(b_type),
            _ => TransitionType::Regular,
        };
        let new_transition = Transition::new(goal_state, t_type);
        self.transition = Some(new_transition);
    }

    fn init_buttons(&mut self) {
        let buttons = &mut self.buttons;
        buttons[0].disabled = false;
        buttons[1].disabled = false;
        buttons[2].disabled = false;
        buttons[3].disabled = true;
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
    sound_trigger: bool,
}

impl Transition {
    pub fn new(goal_state: WorldState, t_type: TransitionType) -> Self {
        Self {
            goal_state,
            t_type,
            time_progressed: 0.,
            sound_trigger: false,
        }
    }

    /// Progresses the transition and returns None, except if there is a subsequent transition that it continues into.
    /// In that case it starts that transition with the leftover time and returns it.
    pub fn progress(&mut self, delta_time: f32) -> Option<Transition> {
        let time_old = self.time_progressed;
        self.time_progressed += delta_time;
        // check for sound to play
        self.update_sound_to_play(time_old, self.time_progressed);

        if self.completed() {
            let total = self.total_duration();
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
            Regular => 9.3,
            EggCracking(_) => 3.0,
        }
    }

    pub fn colors(&self) -> (Color, Color) {
        let color_current_alpha;
        let color_next_alpha;
        match self.t_type {
            TransitionType::Regular => {
                let total = self.total_duration();
                let relative_progress = self.time_progressed / total;
                let alpha = if relative_progress <= 1. / 7. || relative_progress >= 6. / 7. {
                    1.
                } else {
                    ((relative_progress * 2.8 * PI - 0.4 * PI).cos() + 1.) / 2.
                };
                (color_current_alpha, color_next_alpha) = if relative_progress <= 0.5 {
                    (alpha, 0.)
                } else {
                    (0., alpha)
                };
            }
            TransitionType::EggCracking(_) => {
                color_current_alpha = 1.;
                color_next_alpha = 0.;
            }
        }

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

    fn update_sound_to_play(&mut self, time_old: f32, time_new: f32) {
        let sound_start = match self.t_type {
            TransitionType::Regular => self.total_duration() / 1.9,
            TransitionType::EggCracking(_) => self.total_duration(),
        };
        // the check on self.sound_trigger is to make sure that the sound isn't triggered twice in edge cases
        self.sound_trigger =
            if time_old <= sound_start && sound_start <= time_new && !self.sound_trigger {
                true
            } else {
                false
            };
    }

    pub fn sound_to_play(&self) -> Option<SoundIndex> {
        if self.sound_trigger {
            match self.t_type {
                TransitionType::Regular => Some(if macroquad::rand::rand() % 2 == 0 {
                    SoundIndex::Scale1
                } else {
                    SoundIndex::Scale2
                }),
                TransitionType::EggCracking(_) => match self.goal_state {
                    WorldState::BigEggCrack1 | WorldState::EggCrack1 => Some(SoundIndex::Crack1),
                    WorldState::BigEggCrack2 | WorldState::EggCrack2 => Some(SoundIndex::Crack2),
                    _ => panic!("sound for crack requested but goal is no crack"),
                },
            }
        } else {
            None
        }
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
                        _ => panic!("restart?"),
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
                        _ => panic!("restart?"),
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
    Restart,
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
    pub async fn create() -> [Button; 4] {
        let x_step = WORLD_WIDTH / 4.;
        let y = 2700.;
        let size = 600.;
        let restart_size = 400.;
        let border_offset = 180.;

        let mut buttons = [
            Button::new(
                ButtonType::Sun,
                load_texture((ASSET_PATH.to_string() + "button_sun.png").as_str())
                    .await
                    .unwrap(),
                Rect::new((x_step - size / 2.) - border_offset, y, size, size),
            ),
            Button::new(
                ButtonType::Water,
                load_texture((ASSET_PATH.to_string() + "button_water.png").as_str())
                    .await
                    .unwrap(),
                Rect::new(x_step * 2. - size / 2., y, size, size),
            ),
            Button::new(
                ButtonType::Arrowhead,
                load_texture((ASSET_PATH.to_string() + "button_arrow.png").as_str())
                    .await
                    .unwrap(),
                Rect::new((x_step * 3. - size / 2.) + border_offset, y, size, size),
            ),
            Button::new(
                ButtonType::Restart,
                load_texture((ASSET_PATH.to_string() + "button_restart.png").as_str())
                    .await
                    .unwrap(),
                Rect::new(
                    x_step * 2. - restart_size / 2.,
                    y + restart_size / 2.,
                    restart_size,
                    restart_size,
                ),
            ),
        ];

        // restart button is disabled at the start
        buttons[3].disable();

        buttons
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
                if macroquad::input::is_mouse_button_pressed(MouseButton::Left) {
                    new_state = ButtonState::Pressed;
                } else {
                    new_state = ButtonState::Hovered;
                }
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
    pub fn draw(&self) {
        if self.disabled {
            return;
        }

        use ButtonState::*;
        let color = match self.state {
            Idle => Color::new(0.7, 0.7, 0.7, 1.),
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

    // LOADING
    let world_loading = start_coroutine(async move {
        storage::store(World::new().await);
    });
    while !world_loading.is_done() {
        clear_background(Color::default());
        let secs = get_time();
        let dots = ".".repeat(secs as usize % 4);
        draw_text(
            format!("Loading{}", dots).as_str(),
            760.,
            1600.,
            200.,
            WHITE,
        );

        next_frame().await;
    }

    let mut world = storage::get_mut::<World>();

    loop {
        clear_background(Color::default());

        set_camera(&cam);

        world.handle_input(&cam);

        let delta = get_frame_time();
        world.progress(delta);

        world.render();

        set_default_camera();

        // draw_text(format!("FPS: {}", get_fps()).as_str(), 0., 16., 32., WHITE);
        // draw_text(
        //     format!("width: {}", screen_width()).as_str(),
        //     0.,
        //     32.,
        //     32.,
        //     WHITE,
        // );
        // draw_text(
        //     format!("height: {}", screen_height()).as_str(),
        //     0.,
        //     48.,
        //     32.,
        //     WHITE,
        // );

        next_frame().await
    }
}
