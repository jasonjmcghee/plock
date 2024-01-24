use crate::settings::{ProcessType, SETTINGS};
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;
use std::env;
use std::pin::Pin;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio_stream::{Stream, StreamExt};

pub(crate) async fn generate(
    context: String,
    process: ProcessType,
) -> Pin<Box<dyn Stream<Item=String>>> {
    let final_context = {
        let mut partial = context;
        let settings = SETTINGS.lock().unwrap();
        for key in settings.environment.keys() {
            partial = partial.replace(
                &format!("${}", key),
                env::var_os(key).unwrap().to_str().unwrap(),
            );
        }
        partial
    };

    match process {
        ProcessType::Ollama => {
            let model = if let Some(s) = env::var_os("OLLAMA_MODEL") {
                s.to_string_lossy().to_string()
            } else {
                "openhermes2.5-mistral".to_string()
            };

            let request = GenerationRequest::new(
                model, final_context,
            );
            Box::pin(async_stream::stream! {
                let ollama = Ollama::default();
                let mut stream = ollama.generate_stream(request).await.unwrap();

                while let Some(Ok(res)) = stream.next().await {
                    for out in res.into_iter() {
                        yield out.response;
                    }
                }
            })
        }
        ProcessType::Command(custom_command) => {
            let environment = { SETTINGS.lock().unwrap().environment.clone() };
            let child = if custom_command.is_empty() {
                #[cfg(target_os = "windows")] {
                    Command::new("cmd")
                        .envs(environment)
                        .arg("/C")
                        .arg(final_context)
                        .stdout(Stdio::piped())
                        .spawn()
                        .expect("Failed to spawn child process")
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Command::new("sh")
                        .envs(environment)
                        .arg("-c")
                        .arg(final_context)
                        .stdout(Stdio::piped())
                        .spawn()
                        .expect("Failed to spawn child process")
                }
            } else {
                Command::new(&custom_command[0])
                    .envs(environment)
                    .args(&custom_command[1..])
                    .arg(&final_context)
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("Failed to spawn child process")
            };

            let stdout = BufReader::new(child.stdout.expect("Failed to take stdout of child"));

            let stream = async_stream::stream! {
                let mut reader = stdout;
                let mut buffer = Vec::new();

                loop {
                    buffer.clear();
                    let mut temp_buf = [0; 1024]; // Temporary buffer for each read
                    match reader.read(&mut temp_buf).await {
                        Ok(0) => break, // EOF reached
                        Ok(size) => {
                            buffer.extend_from_slice(&temp_buf[..size]);
                            yield String::from_utf8_lossy(&buffer).to_string();
                        },
                        Err(e) => {
                            eprintln!("Error reading from stdout: {}", e);
                            break;
                        }
                    }
                }
            };

            Box::pin(stream)
        }
    }
}
