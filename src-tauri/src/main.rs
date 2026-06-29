// Hide the console window on Windows release builds; required, do not remove.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    file_lens_lib::run()
}
