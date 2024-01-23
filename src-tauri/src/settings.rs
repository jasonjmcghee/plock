
use serde::{Deserialize, Serialize};
use std::{env, fs, path::Path};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use lazy_static::lazy_static;
use tauri::{command, AppHandle};

lazy_static! {
    pub static ref SETTINGS: Mutex<Settings> = Mutex::new(Settings::default());
}

#[derive(Serialize, Deserialize)]
pub struct Ollama {
    pub enabled: bool,
    pub ollama_model: String,
}

#[derive(Serialize, Deserialize)]
pub struct Shortcuts {
    pub basic: String,
    pub with_context: String,
}

#[derive(Serialize, Deserialize)]
pub struct CustomCommand {
    pub name: String,
    pub command: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CustomPrompt {
    pub name: String,
    pub prompt: String,
}

#[derive(Serialize, Deserialize)]
pub struct CustomPrompts {
    pub basic_index: usize,
    pub with_context_index: usize,
    pub custom_prompts: Vec<CustomPrompt>,
}

#[derive(Serialize, Deserialize)]
pub struct CustomCommands {
    pub index: usize,
    pub custom_commands: Vec<CustomCommand>,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub environment: HashMap<String, String>,
    pub ollama: Ollama,
    pub custom_commands: CustomCommands,
    pub custom_prompts: CustomPrompts,
    pub shortcuts: Shortcuts,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            environment: HashMap::new(),
            ollama: Ollama {
                enabled: true,
                ollama_model: "openhermes2.5-mistral".to_string(),
            },
            custom_commands: CustomCommands {
                index: 0,
                custom_commands: vec![
                    CustomCommand {
                        name: "gpt".to_string(),
                        command: ["bash", "/path/to/gpt.sh"].iter().map(|s| s.to_string()).collect(),
                    },
                ],
            },
            custom_prompts: CustomPrompts {
                basic_index: 0,
                with_context_index: 1,
                custom_prompts: vec![
                    CustomPrompt {
                        name: "default basic".to_string(),
                        prompt: "{}".to_string(),
                    },
                    CustomPrompt {
                        name: "default with context".to_string(),
                        prompt: "I will ask you to do something. Below is some extra context to help do what I ask. --------- {} --------- Given the above context, please, {}. DO NOT OUTPUT ANYTHING ELSE.".to_string(),
                    },
                ],
            },
            shortcuts: Shortcuts {
                basic: {
                    #[cfg(target_os = "macos")]
                    {
                        // For Mac, use Command key
                        "Command+Shift+.".to_string()
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        "Ctrl+Shift+.".to_string()
                    }
                },
                with_context: {
                    #[cfg(target_os = "macos")]
                    {
                        // For Mac, use Command key
                        "Command+Shift+/".to_string()
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        "Ctrl+Shift+/".to_string()
                    }
                },
            },
        }
    }
}

pub fn save_current_settings(app_handle: AppHandle) -> Result<(), String> {
    save_settings(app_handle, &SETTINGS.lock().unwrap())?;
    Ok(())
}

#[command]
pub fn save_settings(app_handle: AppHandle, settings: &Settings) -> Result<(), String> {
    let path = get_settings_path(app_handle)?;
    let data = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn load_settings(app_handle: AppHandle) -> Result<(), String> {
    let path = get_settings_path(app_handle.clone())?;
    let settings = if path.exists() {
        let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&data).map_err(|e| e.to_string())?
    } else {
        save_current_settings(app_handle)?;
        Settings::default()
    };
    for (key, value) in settings.environment.iter() {
        env::set_var(key, value);
    }
    *SETTINGS.lock().unwrap() = settings;
    Ok(())
}

pub fn get_settings_path(app_handle: AppHandle) -> Result<PathBuf, String> {
    let path = app_handle.path_resolver().app_local_data_dir().ok_or(
        "Failed to get local data dir".to_string()
    )?;
    Ok(path.join(Path::new("settings.json")))
}

pub fn ensure_local_data_dir(app_handle: AppHandle) -> Result<String, ()> {
    let local_data_dir = app_handle.path_resolver().app_local_data_dir();
    if let Some(dir) = local_data_dir.clone() {
        let path = dir.to_string_lossy().to_string();
        if let Ok(()) = fs::create_dir_all(path.clone()) {
            return Ok(path);
        }
    }
    Err(())
}