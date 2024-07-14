use std::rc::Rc;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rand::{Rng, thread_rng};
use web_sys::HtmlImageElement;

use crate::{browser, engine};
use crate::engine::{Audio, Game, Image, KeyState, Point, Rect, Renderer, Sheet, SpriteSheet};
use crate::obstacle::{Obstacle, rightmost};
use crate::rhb::RedHatBoy;
use crate::segments::{platform_and_stone, stone_and_platform};
use crate::wtd_state_machine::WalkTheDogStateMachine;

pub const HEIGHT: i16 = 600;
pub const TIMELINE_MINIMUM: i16 = 1000;
const OBSTACLE_BUFFER: i16 = 20;

pub struct WalkTheDog {
    machine: Option<WalkTheDogStateMachine>,
}

impl WalkTheDog {
    pub fn new() -> Self {
        WalkTheDog { machine: None }
    }
}

pub struct Walk {
    obstacle_sheet: Rc<SpriteSheet>,
    pub boy: RedHatBoy,
    pub backgrounds: [Image; 2],
    pub obstacles: Vec<Box<dyn Obstacle>>,
    stone: HtmlImageElement,
    pub timeline: i16,
}

impl Walk {
    pub fn knocked_out(&self) -> bool {
        self.boy.knocked_out()
    }

    pub fn reset(walk: Self) -> Self {
        let starting_obstacles =
            stone_and_platform(walk.stone.clone(), walk.obstacle_sheet.clone(), 0);
        let timeline = rightmost(&starting_obstacles);

        Walk {
            boy: RedHatBoy::reset(walk.boy),
            backgrounds: walk.backgrounds,
            obstacles: starting_obstacles,
            obstacle_sheet: walk.obstacle_sheet,
            stone: walk.stone,
            timeline,
        }
    }

    pub fn draw(&self, renderer: &Renderer) {
        self.backgrounds.iter().for_each(|background| {
            background.draw(renderer);
        });
        self.boy.draw(renderer);
        self.obstacles.iter().for_each(|obstacle| {
            obstacle.draw(renderer);
        });
    }

    pub fn velocity(&self) -> i16 {
        -self.boy.walking_speed()
    }

    pub fn generate_next_segment(&mut self) {
        let mut rng = thread_rng();
        let next_segment = rng.gen_range(0..2);
        let mut next_obstacles = match next_segment {
            0 => stone_and_platform(
                self.stone.clone(),
                self.obstacle_sheet.clone(),
                self.timeline + OBSTACLE_BUFFER,
            ),
            1 => platform_and_stone(
                self.stone.clone(),
                self.obstacle_sheet.clone(),
                self.timeline + OBSTACLE_BUFFER,
            ),
            _ => vec![],
        };
        self.timeline = rightmost(&next_obstacles);
        self.obstacles.append(&mut next_obstacles);
    }
}

#[async_trait(?Send)]
impl Game for WalkTheDog {
    async fn initialize(&self) -> Result<Box<dyn Game>> {
        match self.machine {
            None => {
                let tiles: Option<Sheet> =
                    serde_wasm_bindgen::from_value(browser::fetch_json("assets/tiles.json").await?)
                        .expect("Could not deserialize tiles.json");
                let sprite_sheet = Rc::new(SpriteSheet::new(
                    tiles.ok_or_else(|| anyhow!("No Sheet Present"))?,
                    engine::load_image("assets/tiles.png").await?,
                ));

                let sheet: Option<Sheet> = serde_wasm_bindgen::from_value(
                    browser::fetch_json("assets/rhb_trimmed.json").await?,
                )
                .expect("Could not deserialize rhb.json");

                let audio = Audio::new()?;
                let sound = audio.load_sound("assets/SFX_Jump_23.mp3").await?;
                let background_music = audio.load_sound("assets/background_song.mp3").await?;
                audio.play_looping_sound(&background_music)?;

                let rhb = RedHatBoy::new(
                    sheet.clone().ok_or_else(|| anyhow!("No Sheet Present"))?,
                    engine::load_image("assets/rhb_trimmed.png").await?,
                    audio,
                    sound,
                );

                let background = engine::load_image("assets/BG.png").await?;
                let background_width = background.width() as i16;

                let stone = engine::load_image("assets/Stone.png").await?;
                let starting_obstacles = stone_and_platform(stone.clone(), sprite_sheet.clone(), 0);
                let timeline = rightmost(&starting_obstacles);

                let machine = WalkTheDogStateMachine::new(Walk {
                    boy: rhb,
                    backgrounds: [
                        Image::new(background.clone(), Point { x: 0, y: 0 }),
                        Image::new(
                            background,
                            Point {
                                x: background_width,
                                y: 0,
                            },
                        ),
                    ],
                    obstacles: starting_obstacles,
                    obstacle_sheet: sprite_sheet,
                    stone,
                    timeline,
                });
                Ok(Box::new(WalkTheDog {
                    machine: Some(machine),
                }))
            }
            Some(_) => Err(anyhow!("Error: Game is already initialized!")),
        }
    }

    fn update(&mut self, keystate: &KeyState) {
        if let Some(machine) = self.machine.take() {
            self.machine.replace(machine.update(keystate));
        }
        assert!(self.machine.is_some());
    }

    fn draw(&self, renderer: &Renderer) {
        renderer.clear(&Rect::new_from_x_y(0, 0, 600, HEIGHT));

        if let Some(machine) = &self.machine {
            machine.draw(renderer);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use futures::channel::mpsc::unbounded;
    use wasm_bindgen_test::wasm_bindgen_test;
    use web_sys::{AudioBuffer, AudioBufferOptions, HtmlImageElement};

    use crate::browser::document;
    use crate::engine::Sound;
    use crate::wtd_state_machine::{GameOver, WalkTheDogState};

    use super::*;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
    #[wasm_bindgen_test]
    fn test_transition_from_game_over_to_new_game() {
        let (_, receiver) = unbounded();
        let image = HtmlImageElement::new().unwrap();
        let audio = Audio::new().unwrap();
        let options = AudioBufferOptions::new(1, 44100.0);
        let sound = Sound {
            buffer: AudioBuffer::new(&options).unwrap(),
        };
        let rhb = RedHatBoy::new(
            Sheet {
                frames: HashMap::new(),
            },
            image.clone(),
            audio,
            sound,
        );
        let sprite_sheet = SpriteSheet::new(
            Sheet {
                frames: HashMap::new(),
            },
            image.clone(),
        );
        let walk = Walk {
            boy: rhb,
            backgrounds: [
                Image::new(image.clone(), Point { x: 0, y: 0 }),
                Image::new(image.clone(), Point { x: 0, y: 0 }),
            ],
            obstacles: vec![],
            obstacle_sheet: Rc::new(sprite_sheet),
            stone: image.clone(),
            timeline: 0,
        };

        let document = document().unwrap();
        document
            .body()
            .unwrap()
            .insert_adjacent_html(
                "afterbegin",
                "<div id='ui'></div><canvas id='canvas'></canvas>",
            )
            .unwrap();
        browser::draw_ui("<p>This is the UI</p>").unwrap();
        let state = WalkTheDogState {
            _state: GameOver {
                new_game_event: receiver,
            },
            walk,
        };

        state.new_game();

        let ui = browser::find_html_element_by_id("ui").unwrap();
        assert_eq!(ui.child_element_count(), 0);
    }
}
