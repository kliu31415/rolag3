mod rolag3;
mod gfx;
mod sfx;
pub mod util;
pub mod geometry;

fn main() {
    if let Err(e) = rolag3::entry_point::run() {
        log::error!("Rolag3 encountered error: {}", e);
    } else {
        log::info!("Rolag3 finished without errors");
    }
}
