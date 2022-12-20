#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use acas::parse::parse_into_expression;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn parse(expression: &str) -> Result<String, String> {
    parse_into_expression(expression).map_err(|x| format!("{x:?}")).map(|x| {
        x.simplify().map(|x| acas::print::to_latex(&x)).unwrap_or_else(|_| "undefined".into())
    })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![parse])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
