use wasm_bindgen::prelude::wasm_bindgen;

mod actions;
mod game_state;
mod generation;
mod key;
mod keybinds;
mod render;

/* #[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}
*/

#[wasm_bindgen(start)]
fn start() {
    // Set panic hook to display to JS console
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
