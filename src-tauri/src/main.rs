// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::generator::TextGeneratorType;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use rdev::{listen, EventType, Key as RdevKey};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::{sync::Arc, thread};
use tauri::{AppHandle, CustomMenuItem, GlobalShortcutManager, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowEvent};
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;
use crate::settings::SETTINGS;

#[cfg(feature = "ocr")]
mod ocr;

mod generator;
mod settings;

fn make_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let load_settings = CustomMenuItem::new("load_settings".to_string(), "Load Settings");
    let tray_menu = SystemTrayMenu::new()
        .add_item(load_settings)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    SystemTray::new().with_menu(tray_menu)
}

fn main() {
    let exit_flag = Arc::new(AtomicBool::new(false));
    let trigger_flag = Arc::new(AtomicBool::new(false));
    let use_clipboard_flag = Arc::new(AtomicBool::new(false));

    let trigger_flag_clone = trigger_flag.clone();
    let use_clipboard_flag_listen_clone = use_clipboard_flag.clone();
    let use_clipboard_flag_clone = use_clipboard_flag.clone();

    let old_clipboard = Arc::new(Mutex::new(String::new()));

    let trigger_flag_listen_clone = trigger_flag.clone();
    let exit_flag_listen_clone = exit_flag.clone();

    let rt = Arc::new(Runtime::new().unwrap());
    let rt_clone = Arc::clone(&rt);

    let app_handle: Arc<Mutex<Option<AppHandle>>> = Arc::new(Mutex::new(None));
    let app_handle_clone = app_handle.clone();

    thread::spawn(|| {
        let pressed_keys = Arc::new(Mutex::new(HashSet::new()));

        listen(move |event| {
            let pressed_keys_clone = pressed_keys.clone();
            let mut pressed_keys = pressed_keys.lock().unwrap();
            let mut escape_pressed = false;

            {
                match event.event_type {
                    EventType::KeyPress(key) => {
                        pressed_keys.insert(key);
                        if key == RdevKey::Escape {
                            escape_pressed = true;
                        }
                    }
                    EventType::KeyRelease(key) => {
                        pressed_keys.remove(&key);
                    }
                    _ => {}
                }
            }

            if escape_pressed {
                exit_flag_listen_clone.store(true, Ordering::SeqCst);
                return;
            }

            if !pressed_keys.is_empty() {
                return;
            }

            if trigger_flag_listen_clone.load(Ordering::SeqCst) {
                println!("tried to trigger");
                let should_use_clipboard_as_context =
                    use_clipboard_flag_listen_clone.load(Ordering::SeqCst);
                // Reset clipboard
                use_clipboard_flag_listen_clone.store(false, Ordering::SeqCst);
                // If no keys are pressed, trigger the action
                trigger_flag_listen_clone.store(false, Ordering::SeqCst);
                // Reset exit flag
                exit_flag_listen_clone.store(false, Ordering::SeqCst);

                let original_clipboard_contents = {
                    let mut handle = app_handle_clone.lock().unwrap();
                    handle
                        .as_mut()
                        .unwrap()
                        .clipboard_manager()
                        .clipboard
                        .lock()
                        .unwrap()
                        .get_text()
                        .expect("Failed to get clipboard contents.")
                };
                let cloned_contents = original_clipboard_contents.clone();
                {
                    *old_clipboard.lock().unwrap() = cloned_contents;
                }
                trigger_action(
                    app_handle_clone.clone(),
                    original_clipboard_contents,
                    rt_clone.clone(),
                    exit_flag_listen_clone.clone(),
                    pressed_keys_clone.clone(),
                    should_use_clipboard_as_context,
                );
            }
        })
        .expect("Failed to listen to keypresses.");
    });

    tauri::Builder::default()
        .setup(move |app| {
            let _ = settings::ensure_local_data_dir(app.app_handle())
                .expect("Failed to create local data dir");
            let _ = settings::load_settings(app.app_handle());

            #[cfg(target_os = "macos")]
            {
                use tauri::ActivationPolicy;
                app.set_activation_policy(ActivationPolicy::Accessory);
            }

            {
                app_handle.lock().unwrap().replace(app.handle().clone());
            }

            let trigger_flag_second_clone = trigger_flag_clone.clone();
            let basic_shortcut = { SETTINGS.lock().unwrap().shortcuts.basic.clone() };
            let with_context_shortcut = { SETTINGS.lock().unwrap().shortcuts.with_context.clone() };

            app.global_shortcut_manager()
                .register(&basic_shortcut, move || {
                    trigger_flag_second_clone.store(true, Ordering::SeqCst);
                })
                .expect("Failed to register global shortcut");

            app.global_shortcut_manager()
                .register(&with_context_shortcut, move || {
                    use_clipboard_flag_clone.store(true, Ordering::SeqCst);
                    trigger_flag_clone.store(true, Ordering::SeqCst);
                })
                .expect("Failed to register global shortcut");

            Ok(())
        })
        .system_tray(make_tray())
        .on_system_tray_event({
            move |app, event| {
                if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                    match id.as_str() {
                        "quit" => std::process::exit(0),
                        "load_settings" => {
                            settings::load_settings(
                                app.app_handle().clone()
                            ).expect("Failed to load settings.");
                        }
                        _ => {}
                    }
                }
            }
        })
        .on_window_event(|event| {
            if let WindowEvent::CloseRequested { api, .. } = event.event() {
                event.window().hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn copy_and_remove_selected_text() {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Release).unwrap();
        // copy
        enigo.key(Key::Meta, Direction::Press).unwrap();
        enigo.key(Key::Unicode('c'), Direction::Click).unwrap();
        enigo.key(Key::Meta, Direction::Release).unwrap();
        enigo.key(Key::Backspace, Direction::Click).unwrap();
    }

    #[cfg(not(target_os = "macos"))]
    {
        // For Windows and Linux, use Ctrl key
        enigo.key(Key::LControl, Direction::Press).unwrap();
        enigo.key(Key::Unicode('c'), Direction::Click).unwrap();
        enigo.key(Key::LControl, Direction::Release).unwrap();
        enigo.key(Key::Backspace, Direction::Click).unwrap();
    }
}

fn get_context(
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    original_clipboard_contents: String,
    should_use_clipboard_as_context: bool,
) -> String {
    println!("preparing to copy text...");
    copy_and_remove_selected_text();

    let user_prompt = {
        let mut handle = app_handle.lock().unwrap();
        handle
            .as_mut()
            .unwrap()
            .clipboard_manager()
            .clipboard
            .lock()
            .unwrap()
            .get_text()
            .expect("Failed to restore clipboard.")
    };

    if !should_use_clipboard_as_context {
        let basic_prompt = {
            let settings = SETTINGS.lock().unwrap();
            let index = settings.custom_prompts.basic_index;
            settings.custom_prompts.custom_prompts[index].prompt.clone()
        };
        return basic_prompt.replace("{}", &user_prompt);
    }

    println!("copied... {}", user_prompt);

    let with_context_prompt = {
        let settings = SETTINGS.lock().unwrap();
        let index = settings.custom_prompts.with_context_index;
        settings.custom_prompts.custom_prompts[index].prompt.clone()
    };

    let context = {
        #[cfg(not(feature = "ocr"))]
        {
            original_clipboard_contents
        }

        #[cfg(feature = "ocr")]
        {
            use crate::ocr::get_text_on_screen;
            let text_on_screen = match get_text_on_screen() {
                Ok(contents) => contents,
                Err(e) => {
                    panic!("Failed to get text on screen: {}", e);
                }
            };
            format!("{}\n\n{}", original_clipboard_contents, text_on_screen)
        }
    };

    let mut pieces = with_context_prompt.split("{}");
    let mut final_prompt = "".to_string();
    // Before {}
    final_prompt.push_str(pieces.next().unwrap());
    // Replace first {} with context
    final_prompt.push_str(&context);
    // After first {}, before second {}
    final_prompt.push_str(pieces.next().unwrap());
    // Replace second {} with user_prompt
    final_prompt.push_str(&user_prompt);
    // After second {}
    final_prompt.push_str(pieces.next().unwrap());
    final_prompt
}

fn trigger_action(
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    original_clipboard_contents: String,
    rt: Arc<Runtime>,
    exit_flag: Arc<AtomicBool>,
    pressed_keys: Arc<Mutex<HashSet<RdevKey>>>,
    should_use_clipboard_as_context: bool,
) {
    let exit_flag_thread = exit_flag.clone();
    let original_clipboard_contents_clone = original_clipboard_contents.clone();

    let context = get_context(
        app_handle.clone(),
        original_clipboard_contents,
        should_use_clipboard_as_context,
    );
    println!("Context: {}", context);

    let use_ollama = { SETTINGS.lock().unwrap().ollama.enabled };
    let generator = if use_ollama {
        TextGeneratorType::OllamaGenerator
    } else {
        TextGeneratorType::ShellScriptGenerator
    };

    rt.spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async {
            let mut enigo = Enigo::new(&Settings::default()).unwrap();
            let mut response_stream = generator.generate(context).await;

            let mut buffer = Vec::new();
            let mut did_exit = false;
            while let Some(response) = response_stream.next().await {
                buffer.push(response);

                if buffer.len() > 4 {
                    let output = buffer.join("");
                    buffer.clear();
                    enigo.fast_text(&output).expect("Failed to type out text");
                }

                // Exit loop if child process has finished or exit flag is set
                if exit_flag_thread.load(Ordering::SeqCst) {
                    did_exit = true;
                    break;
                }
            }

            if !did_exit && !buffer.is_empty() {
                let output = buffer.join("");
                enigo.fast_text(&output).expect("Failed to type out text");
            }

            exit_flag_thread.store(false, Ordering::SeqCst);

            {
                let mut handle = app_handle.lock().unwrap();
                handle
                    .as_mut()
                    .unwrap()
                    .clipboard_manager()
                    .clipboard
                    .lock()
                    .unwrap()
                    .set_text(original_clipboard_contents_clone)
                    .expect("Failed to restore clipboard.");
            }

            pressed_keys.lock().unwrap().clear();
        });
    });
}
