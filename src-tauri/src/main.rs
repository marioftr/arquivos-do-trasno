// Evita que se abra unha consola adicional en Windows en modo release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    trasno_lib::run();
}
