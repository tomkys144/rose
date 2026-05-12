use crate::gui::Message;
use crate::models::config::{PipelineConfig, PipelineStep};
use crate::models::context::PipelineContext;
use crate::models::message::{Message as LogMsg, MessageType};
use directories::ProjectDirs;
use iced::futures::{SinkExt, channel};
use iced::{futures, stream};
use std::path::PathBuf;
use std::{fs, path};

mod entrez;
mod uniprot;
mod parser;

macro_rules! send_log {
    ($output:expr, $msg_type:expr, $($arg:tt)*) => {
        let _ = $output.send(Message::LogReceived(LogMsg {
            msg: format!($($arg)*),
            msg_type: $msg_type,
        })).await;
    };
}

pub fn run_pipeline(
    source: String,
    target: String,
    config: PipelineConfig,
) -> impl futures::Stream<Item = Message> {
    stream::channel(
        100,
        move |mut output: channel::mpsc::Sender<Message>| async move {
            send_log!(output, MessageType::Info, "Initializing ROSE pipeline...");

            let mut context = PipelineContext {
                source,
                target,
                ..Default::default()
            };

            for step in config.steps {
                match step {
                    PipelineStep::FindReferenceGenome(cfg) => {
                        send_log!(
                            output,
                            MessageType::Info,
                            "Searching NCBI for '{}' reference genome...",
                            context.target
                        );

                        let mut thread_ctx = context;
                        let (res, returned_ctx) = tokio::task::spawn_blocking(move || {
                            let r = crate::proc::entrez::find_ref_genome(&mut thread_ctx, &cfg);
                            (r, thread_ctx)
                        })
                        .await
                        .unwrap();

                        context = returned_ctx;

                        if let Err(e) = res {
                            send_log!(output, MessageType::Error, "{}", e);
                            break;
                        }
                        send_log!(
                            output,
                            MessageType::Info,
                            "Found Source Genome IDs: {:?}\nFound Target Genome IDs: {:?}",
                            context.src_genome_ids,
                            context.tgt_genome_ids
                        );
                    }

                    PipelineStep::FetchGenomeAnnotations(cfg) => {
                        send_log!(output, MessageType::Info, "Downloading XML Annotations...");

                        let mut thread_ctx = context;
                        let (res, returned_ctx) = tokio::task::spawn_blocking(move || {
                            let genome_dir = get_genome_dir();
                            let r = crate::proc::entrez::fetch_genome_xml(
                                &mut thread_ctx,
                                &cfg,
                                &genome_dir,
                            );
                            (r, thread_ctx)
                        })
                        .await
                        .unwrap();

                        context = returned_ctx;

                        if let Err(e) = res {
                            send_log!(output, MessageType::Error, "{}", e);
                            break;
                        }

                        send_log!(output, MessageType::Info, "XML Downloaded successfully!");
                    }

                    _ => {
                        send_log!(
                            output,
                            MessageType::Warning,
                            "Skipping unimplemented test step..."
                        );
                    }
                }
            }

            send_log!(
                output,
                MessageType::Info,
                "Test Pipeline execution finished."
            );
            let _ = output.send(Message::SearchCompleted(context.results)).await;
        },
    )
}

fn get_genome_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        let path = PathBuf::from("data/genomes");
        let _ = fs::create_dir_all(&path);
        path
    } else {
        if let Some(proj_dirs) = ProjectDirs::from("cz", "tkysela", "ROSE") {
            let data_dir = proj_dirs.data_dir().join("data").join("genomes");
            let _ = fs::create_dir_all(&data_dir);
            data_dir
        } else {
            let path = PathBuf::from("data/genomes");
            let _ = fs::create_dir_all(&path);
            path
        }
    }
}
