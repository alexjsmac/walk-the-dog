use wasm_bindgen::prelude::*;

use crate::engine::GameLoop;
use crate::game::WalkTheDog;
use crate::utils::set_panic_hook;

#[macro_use]
mod browser;
mod engine;
mod game;
mod obstacle;
mod rhb;
mod rhb_state_machine;
mod rhb_states;
mod segments;
mod sound;
mod utils;
mod wtd_state_machine;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    set_panic_hook();

    browser::spawn_local(async move {
        let game = WalkTheDog::new();
        GameLoop::start(game)
            .await
            .expect("Could not start game loop");
    });
    Ok(())
}
