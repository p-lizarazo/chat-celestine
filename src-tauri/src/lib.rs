mod commands;
mod llm;

use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let app_state = AppState::new();

  tauri::Builder::default()
    .manage(app_state)
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      app.handle().plugin(tauri_plugin_store::Builder::default().build())?;
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::initialize_provider,
      commands::list_models,
      commands::send_chat_message,
      commands::send_chat_message_stream,
      commands::check_ollama_available,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
