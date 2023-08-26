pub mod turtle_core;
pub mod server;
pub mod scripts;
pub mod entry;

pub type TurtleIdentifier = usize;
pub type TurtleIndex = usize;
pub type DefaultData<'a> = (
    TurtleIdentifier,
    TurtleIndex,
    &'a turtle_core::control::TurtControl<'a>,
    &'a mut turtle_core::navigation::TurtNavigation<'a>,
);

pub const PROGRESS_DIR: &str = "progress";

pub fn init_dirs() {
    std::fs::create_dir_all(turtle_core::navigation::NAV_DIR).unwrap();
    std::fs::create_dir_all(PROGRESS_DIR).unwrap();
}
