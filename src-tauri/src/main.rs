// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate core;

use crate::generator::generate;
use crate::settings::{SelectionAction, Step, SETTINGS};
use arboard::ImageData;
use base64::decode;
use enigo::{Direction, Enigo, InputResult, Key, Keyboard};
use image::{load_from_memory, EncodableLayout};
use rdev::{listen, EventType, Key as RdevKey};
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::{env, sync::Arc, thread};
use tauri::{
    AppHandle, CustomMenuItem, GlobalShortcutManager, Manager, SystemTray, SystemTrayEvent,
    SystemTrayMenu, SystemTrayMenuItem, WindowEvent,
};
use tokio::runtime::Runtime;
use tokio_stream::StreamExt;

#[cfg(feature = "ocr")]
mod ocr;

mod generator;
mod settings;

fn make_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let load_settings = CustomMenuItem::new("load_settings".to_string(), "Load Settings");
    let settings_location =
        CustomMenuItem::new("settings_location".to_string(), "<Settings Location>").disabled();
    let tray_menu = SystemTrayMenu::new()
        .add_item(load_settings)
        .add_item(settings_location)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    SystemTray::new().with_menu(tray_menu)
}

fn main() {
    let exit_flag = Arc::new(AtomicBool::new(false));
    let trigger_flag = Arc::new(AtomicBool::new(false));
    let trigger_index = Arc::new(AtomicUsize::new(0));

    let old_clipboard = Arc::new(Mutex::new(String::new()));

    let trigger_flag_clone = trigger_flag.clone();
    let trigger_index_clone = trigger_index.clone();

    let trigger_flag_system_tray_clone = trigger_flag.clone();
    let trigger_index_system_tray_clone = trigger_index.clone();

    let trigger_flag_listen_clone = trigger_flag.clone();
    let trigger_index_listen_clone = trigger_index.clone();

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
                        .unwrap_or("".to_string())
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
                    trigger_index_listen_clone.clone(),
                );
            }
        })
        .expect("Failed to listen to keypresses.");
    });

    tauri::Builder::default()
        .setup(move |app| {
            let path = settings::ensure_local_data_dir(app.app_handle())
                .expect("Failed to create local data dir");
            app.tray_handle()
                .get_item("settings_location")
                .set_title(path)
                .unwrap();

            let trigger_index_clone = trigger_index_clone.clone();
            let trigger_flag_second_clone = trigger_flag_clone.clone();

            let _ = settings::load_settings(
                app.app_handle(),
                trigger_index_clone.clone(),
                trigger_flag_second_clone.clone()
            );

            #[cfg(target_os = "macos")]
            {
                use tauri::ActivationPolicy;
                app.set_activation_policy(ActivationPolicy::Accessory);
            }

            {
                app_handle.lock().unwrap().replace(app.handle().clone());
            }
            Ok(())
        })
        .system_tray(make_tray())
        .on_system_tray_event({
            move |app, event| {
                if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                    match id.as_str() {
                        "quit" => std::process::exit(0),
                        "load_settings" => {
                            let trigger_index_clone = trigger_index_system_tray_clone.clone();
                            let trigger_flag_second_clone = trigger_flag_system_tray_clone.clone();

                            settings::load_settings(
                                app.app_handle().clone(),
                                trigger_index_clone.clone(),
                                trigger_flag_second_clone.clone()
                            )
                                .expect("Failed to load settings.")
                        },
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

fn handle_selection(selection_action: SelectionAction) {
    let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();
    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Release).unwrap();
        // copy
        enigo.key(Key::Meta, Direction::Press).unwrap();
        // enigo.key(Key::Unicode('c'), Direction::Click).unwrap();
        enigo.raw(8, Direction::Click).unwrap();
        enigo.key(Key::Meta, Direction::Release).unwrap();
    }

    #[cfg(not(target_os = "macos"))]
    {
        // For Windows and Linux, use Ctrl key
        enigo.key(Key::LControl, Direction::Press).unwrap();
        enigo.key(Key::Unicode('c'), Direction::Click).unwrap();
        enigo.key(Key::LControl, Direction::Release).unwrap();
    }

    match selection_action {
        SelectionAction::Remove => {
            enigo.key(Key::Backspace, Direction::Click).unwrap();
        }
        SelectionAction::Newline => {
            // Why are we deleting and rewriting? something strange with enigo or something is getting locked up
            enigo.key(Key::Backspace, Direction::Click).unwrap();
            // enigo.key(Key::RightArrow, Direction::Click).unwrap();
            // enigo.text("\n\n").unwrap();
        }
        SelectionAction::Nothing => {
        }
    }
}

fn get_context(app_handle: Arc<Mutex<Option<AppHandle>>>, pipeline_index: Arc<AtomicUsize>) {
    println!("preparing to copy text...");
    let selection_action = {
        let settings = SETTINGS.lock().unwrap();
        let i = pipeline_index.load(Ordering::SeqCst);
        let trigger = settings.triggers[i].clone();
        trigger.selection_action.unwrap_or(SelectionAction::Remove)
    };
    handle_selection(selection_action);

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
            .expect("Failed to get clipboard.")
    };

    {
        SETTINGS
            .lock()
            .unwrap()
            .add_env_var("SELECTION".to_string(), user_prompt.clone());
    }

    println!("copied... {}", user_prompt);

    #[cfg(feature = "ocr")]
    {
        use crate::ocr::get_text_on_screen;
        let text_on_screen = match get_text_on_screen() {
            Ok(contents) => contents,
            Err(e) => {
                panic!("Failed to get text on screen: {}", e);
            }
        };
        {
            SETTINGS
                .lock()
                .unwrap()
                .add_env_var("OCR".to_string(), text_on_screen.clone());
        }
    }
}

fn trigger_action(
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    original_clipboard_contents: String,
    rt: Arc<Runtime>,
    exit_flag: Arc<AtomicBool>,
    pressed_keys: Arc<Mutex<HashSet<RdevKey>>>,
    pipeline_index: Arc<AtomicUsize>,
) {
    let exit_flag_thread = exit_flag.clone();
    {
        SETTINGS
            .lock()
            .unwrap()
            .add_env_var("CLIPBOARD".to_string(), original_clipboard_contents.clone());
    }

    get_context(app_handle.clone(), pipeline_index.clone());
    println!("CLIPBOARD: {:?}", env::var_os("CLIPBOARD"));
    println!("SELECTION: {:?}", env::var_os("SELECTION"));

    let app_handle_clone = app_handle.clone();

    rt.spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async {
            let index = pipeline_index.clone();
            let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();

            loop {
                // Create a pipe here so that we can shove output to other processes
                let i = index.load(Ordering::SeqCst);

                let (trigger, process_type, prompt) = {
                    let settings = SETTINGS.lock().unwrap();
                    let trigger = settings.triggers[i].clone();
                    (
                        trigger.clone(),
                        settings.processes[trigger.process].clone(),
                        settings.prompts[trigger.prompt].clone(),
                    )
                };

                let mut response_stream = generate(prompt.prompt, process_type).await;

                let mut whole_buffer = Vec::new();
                let mut delta_buffer = Vec::new();

                let mut did_exit = false;

                {
                    while let Some(response) = response_stream.next().await {

                        let delta_output = {
                            if delta_buffer.len() > 4 && !response.starts_with('\n')
                            // workaround for a bug in enigo, it doesn't like leading newlines
                            {
                                let s = delta_buffer.clone().join("");
                                delta_buffer.clear();
                                s
                            } else {
                                "".to_string()
                            }
                        };

                        // move this to after the delta_output so we can not start a delta_buffer with a newline
                        whole_buffer.push(response.clone());
                        delta_buffer.push(response);

                        for step in trigger.next_steps.clone() {
                            if let Step::StreamTextToScreen = step {
                                enigo.text(&delta_output).expect("Failed to type out text");
                            }
                        }

                        // Exit loop if child process has finished or exit flag is set
                        if exit_flag_thread.load(Ordering::SeqCst) {
                            did_exit = true;
                            break;
                        }
                    }
                }

                let mut should_continue = false;
                if !did_exit {
                    let delta_output = delta_buffer.join("");
                    let whole_output = whole_buffer.join("");
                    println!("Whole output: {}", whole_output);

                    for step in trigger.next_steps {
                        match step {
                            Step::StreamTextToScreen => {
                                if !delta_buffer.is_empty() {
                                    enigo.text(&delta_output).expect("Failed to type out text");
                                }
                            }
                            Step::StoreAsEnvVar(key) => {
                                SETTINGS
                                    .lock()
                                    .unwrap()
                                    .add_env_var(key, whole_output.clone());
                            }
                            Step::Trigger(i) => {
                                index.store(i, Ordering::SeqCst);
                                should_continue = true;
                            }
                            Step::WriteFinalTextToScreen => {
                                let mut final_buffer = whole_buffer.clone();
                                // Why are we deleting and rewriting? something strange with enigo or something is getting locked up
                                if let SelectionAction::Newline = trigger
                                    .selection_action
                                    .clone()
                                    .unwrap_or(SelectionAction::Remove)
                                {
                                    let to_insert = format!(
                                        "{}\n\n",
                                        env::var_os("SELECTION").unwrap().to_string_lossy()
                                    );
                                    final_buffer.insert(0, to_insert.clone());
                                }
                                let final_buffer_output = final_buffer.join("");

                                {
                                    let mut handle = app_handle_clone.lock().unwrap();
                                    handle
                                        .as_mut()
                                        .unwrap()
                                        .clipboard_manager()
                                        .clipboard
                                        .lock()
                                        .unwrap()
                                        .set_text(&final_buffer_output)
                                        .expect("Failed to copy image to clipboard");
                                };

                                paste(&mut enigo);
                            }
                            Step::WriteImageToScreen => {
                                let image_data = decode(&whole_output.trim())
                                    .expect("Failed to decode base64 string");

                                // Convert binary data to an image format (PNG)
                                let img = load_from_memory(&image_data)
                                    .expect("Failed to load image from memory");
                                // let mut cursor = Cursor::new(Vec::new());
                                let rgba = img.into_rgba8();
                                let image = ImageData {
                                    bytes: Cow::from(rgba.as_bytes()),
                                    width: rgba.width() as usize,
                                    height: rgba.height() as usize,
                                };

                                // Copy the image to the clipboard
                                {
                                    let mut handle = app_handle_clone.lock().unwrap();
                                    handle
                                        .as_mut()
                                        .unwrap()
                                        .clipboard_manager()
                                        .clipboard
                                        .lock()
                                        .unwrap()
                                        .set_image(image)
                                        .expect("Failed to copy image to clipboard");
                                };
                                paste(&mut enigo);
                            }
                        }
                    }
                }

                if !should_continue {
                    break;
                }
            }

            exit_flag_thread.store(false, Ordering::SeqCst);

            {
                let mut handle = app_handle.lock().unwrap();
                if let Some(old_clipboard) = env::var_os("CLIPBOARD") {
                    handle
                        .as_mut()
                        .unwrap()
                        .clipboard_manager()
                        .clipboard
                        .lock()
                        .unwrap()
                        .set_text(old_clipboard.to_string_lossy().to_string())
                        .expect("Failed to restore clipboard.");
                }
            }

            pressed_keys.lock().unwrap().clear();
        });
    });
}

fn paste(enigo: &mut Enigo) {
    enigo
        .key(Key::Meta, Direction::Release)
        .expect("Failed to paste text");
    enigo
        .key(Key::Meta, Direction::Press)
        .expect("Failed to paste text");
    // This keeps causing a bad access in `unsafe`: enigo-0.2.0-rc2/src/macos/macos_impl.rs:631
    // enigo.key(Key::Unicode('v'), Direction::Click).expect("Failed to paste text");
    enigo
        .raw(9, Direction::Click)
        .expect("Failed to paste text");
    enigo
        .key(Key::Meta, Direction::Release)
        .expect("Failed to paste text");
}
