use serde::{Deserialize, Serialize};
use std::{env, fs, path::Path};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use lazy_static::lazy_static;
use tauri::{command, AppHandle};
use crate::settings::Step::StreamTextToScreen;

lazy_static! {
    pub static ref SETTINGS: Mutex<Settings> = Mutex::new(Settings::default());
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub environment: HashMap<String, String>,
    pub processes: Vec<ProcessType>,
    pub prompts: Vec<CustomPrompt>,
    pub triggers: Vec<Trigger>,
}

impl Settings {
    pub fn add_env_var(&mut self, key: String, value: String) {
        env::set_var(key.clone(), value.clone());
        self.environment.insert(key, value);
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            environment: HashMap::from([
                ("OLLAMA_MODEL".to_string(), "openhermes2.5-mistral".to_string())
            ]),
            processes: vec![
                ProcessType::Ollama,
                ProcessType::Command(
                    ["bash", "/path/to/gpt.sh"].iter().map(|s| s.to_string()).collect()
                ),
            ],
            prompts: vec![
                CustomPrompt {
                    name: "default basic".to_string(),
                    prompt: "$SELECTION".to_string(),
                },
                CustomPrompt {
                    name: "default with context".to_string(),
                    prompt: "I will ask you to do something. Below is some extra context to help do what I ask. --------- $CLIPBOARD --------- Given the above context, please, $SELECTION. DO NOT OUTPUT ANYTHING ELSE.".to_string(),
                },
            ],
            triggers: vec![
                Trigger {
                    trigger_with_shortcut: Some({
                        #[cfg(target_os = "macos")]
                        {
                            // For Mac, use Command key
                            "Command+Shift+.".to_string()
                        }
                        #[cfg(not(target_os = "macos"))]
                        {
                            "Ctrl+Shift+.".to_string()
                        }
                    }),
                    process: 0,
                    prompt: 0,
                    next_steps: vec![StreamTextToScreen],
                    ..Trigger::default()
                },
                Trigger {
                    trigger_with_shortcut: Some({
                        #[cfg(target_os = "macos")]
                        {
                            // For Mac, use Command key
                            "Command+Shift+/".to_string()
                        }
                        #[cfg(not(target_os = "macos"))]
                        {
                            "Ctrl+Shift+/".to_string()
                        }
                    }),
                    process: 0,
                    prompt: 1,
                    next_steps: vec![StreamTextToScreen],
                    ..Trigger::default()
                },
            ],
        }
    }
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
        Settings::default()
    };
    for (key, value) in settings.environment.iter() {
        { SETTINGS.lock().unwrap().add_env_var(key.clone(), value.clone()); }
    }
    // Ensures any newly introduced fields are stored in the settings file
    save_settings(app_handle, &settings)?;
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Ollama {
    pub model: String,
}

impl Default for Ollama {
    fn default() -> Self {
        Self {
            model: "openhermes2.5-mistral".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProcessType {
    Ollama,
    Command(Vec<String>),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    StreamTextToScreen,
    WriteFinalTextToScreen,
    WriteImageToScreen,
    StoreAsEnvVar(String),
    Trigger(usize),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SelectionAction {
    Remove,
    Newline,
    Nothing,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Trigger {
    pub trigger_with_shortcut: Option<String>,
    pub process: usize,
    pub prompt: usize,
    pub next_steps: Vec<Step>,
    pub selection_action: Option<SelectionAction>,
}

impl Default for Trigger {
    fn default() -> Self {
        Self {
            trigger_with_shortcut: None,
            process: 0,
            prompt: 0,
            next_steps: vec![StreamTextToScreen],
            selection_action: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CustomPrompt {
    pub name: String,
    pub prompt: String,
}

