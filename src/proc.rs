use crate::gui::Message;
use crate::models::config::{PipelineConfig, PipelineStep};
use crate::models::context::PipelineContext;
use crate::models::message::{Message as LogMsg, MessageType};
use crate::proc::align::align_all;
use directories::ProjectDirs;
use iced::futures::future::BoxFuture;
use iced::futures::{FutureExt, SinkExt, channel};
use iced::{futures, stream};
use std::path::PathBuf;
use std::{fs, path};

mod align;
mod entrez;
mod parser;
mod uniprot;

macro_rules! send_log {
    ($output:expr, $msg_type:expr, $branch:expr , $($arg:tt)*) => {
        let _ = $output.send(Message::LogReceived(LogMsg {
            msg: format!($($arg)*),
            msg_type: $msg_type,
            branch: $branch.map(|s| s.to_string()),
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
            let mut context = PipelineContext {
                source,
                target,
                ..Default::default()
            };

            execute_steps(config.steps, None, &mut context, &mut output).await;

            let _ = output.send(Message::SearchCompleted(context)).await;
        },
    )
}

fn execute_steps<'a>(
    steps: Vec<PipelineStep>,
    branch_name: Option<&'a str>,
    context: &'a mut PipelineContext,
    output: &'a mut channel::mpsc::Sender<Message>,
) -> BoxFuture<'a, ()> {
    async move {
        for step in steps {
            match step {
                PipelineStep::FindReferenceGenome(cfg) => {
                    send_log!(
                        output,
                        MessageType::Info,
                        branch_name,
                        "Fetching reference genome from NCBI..."
                    );

                    let mut thread_ctx = std::mem::take(context);
                    let (res, returned_ctx) = tokio::task::spawn_blocking(move || {
                        let r = entrez::find_ref_genome(&mut thread_ctx, &cfg);
                        (r, thread_ctx)
                    })
                    .await
                    .unwrap();

                    *context = returned_ctx;

                    if let Err(e) = res {
                        send_log!(
                            output,
                            MessageType::Error,
                            branch_name,
                            "{}",
                            e
                        );
                        break;
                    }

                    send_log!(
                        output,
                        MessageType::Info,
                        branch_name,
                        "Found Source Genome IDs: {:?}\nFound Target Genome IDs: {:?}",
                        context.src_genome_ids,
                        context.tgt_genome_ids
                    );
                }

                PipelineStep::FetchGenomeAnnotations(cfg) => {
                    send_log!(
                        output,
                        MessageType::Info,
                        branch_name,
                        "Downloading genome annotations from NCBI..."
                    );

                    let mut thread_ctx = std::mem::take(context);
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

                    *context = returned_ctx;

                    if let Err(e) = res {
                        send_log!(
                            output,
                            MessageType::Error,
                            branch_name,
                            "{}",
                            e
                        );
                        break;
                    }

                    send_log!(
                        output,
                        MessageType::Info,
                        branch_name,
                        "XML Downloaded successfully!"
                    );
                }

                PipelineStep::ParseXmlAnnotations() => {
                    send_log!(
                        output,
                        MessageType::Info,
                        branch_name,
                        "Parsing XML Annotations..."
                    );

                    let mut thread_ctx = std::mem::take(context);
                    let (res, returned_ctx) = tokio::task::spawn_blocking(move || {
                        let genome_dir = get_genome_dir();
                        let r = parser::find_similar(&mut thread_ctx, &genome_dir);
                        (r, thread_ctx)
                    })
                    .await
                    .unwrap();

                    *context = returned_ctx;

                    if let Err(e) = res {
                        send_log!(
                            output,
                            MessageType::Error,
                            branch_name,
                            "{}",
                            e
                        );
                        break;
                    }

                    send_log!(
                        output,
                        MessageType::Info,
                        branch_name,
                        "XML Parsing finished!"
                    );
                }

                PipelineStep::FetchMissingUniprot() => {
                    send_log!(
                        output,
                        MessageType::Info,
                        branch_name,
                        "Searching Uniprot for missing gene annotations..."
                    );

                    let mut thread_ctx = std::mem::take(context);
                    let (res, returned_ctx) = tokio::task::spawn_blocking(move || {
                        let r = uniprot::fetch_missing_sequences(&mut thread_ctx);
                        (r, thread_ctx)
                    })
                    .await
                    .unwrap();

                    *context = returned_ctx;

                    if let Err(e) = res {
                        send_log!(
                            output,
                            MessageType::Error,
                            branch_name,
                            "{}",
                            e
                        );
                        break;
                    }

                    send_log!(
                        output,
                        MessageType::Info,
                        branch_name,
                        "Searching finished!"
                    );
                }

                PipelineStep::AlignFound(cfg) => {
                    send_log!(
                        output,
                        MessageType::Info,
                        branch_name,
                        "Calculating alignment scores for found genes..."
                    );

                    let mut thread_ctx = std::mem::take(context);
                    let (res, returned_ctx) = tokio::task::spawn_blocking(move || {
                        let r = align::align_matches(&mut thread_ctx, cfg);
                        (r, thread_ctx)
                    })
                    .await
                    .unwrap();

                    *context = returned_ctx;

                    if let Err(e) = res {
                        send_log!(
                            output,
                            MessageType::Error,
                            branch_name,
                            "{}",
                            e
                        );
                        break;
                    }

                    send_log!(
                        output,
                        MessageType::Info,
                        branch_name,
                        "Calculation finished!"
                    );
                }

                PipelineStep::RunAlignment(cfg) => {
                    send_log!(
                        output,
                        MessageType::Info,
                        branch_name,
                        "Calculating alignment scores for all genes..."
                    );
                    let mut thread_ctx = std::mem::take(context);
                    let (res, returned_ctx) = tokio::task::spawn_blocking(move || {
                        let genome_dir = get_genome_dir();
                        let _ = parser::find_similar(&mut thread_ctx, &genome_dir);
                        thread_ctx.results.clear();
                        let r2 = align_all(&mut thread_ctx, cfg);
                        (r2, thread_ctx)
                    })
                    .await
                    .unwrap();

                    *context = returned_ctx;

                    if let Err(e) = res {
                        send_log!(
                            output,
                            MessageType::Error,
                            branch_name,
                            "{}",
                            e
                        );
                        break;
                    }

                    send_log!(
                        output,
                        MessageType::Info,
                        branch_name,
                        "Alignment finished!"
                    );
                }

                PipelineStep::ParallelBranches(branches) => {
                    let mut handles = Vec::new();

                    for (i, branch) in branches.into_iter().enumerate() {
                        let mut branch_ctx = context.clone();
                        let mut branch_output = output.clone();
                        
                        let  b_name = format!("Branch-{}", i);
                        
                        handles.push(tokio::spawn(async move {
                            execute_steps(branch, Some(&b_name), &mut branch_ctx, &mut branch_output).await;
                            branch_ctx
                        }));
                    }

                    for handle in handles {
                        match handle.await {
                            Ok(ctx) => merge_results(context, ctx),
                            Err(e) => {
                                send_log!(
                                    output,
                                    MessageType::Error,
                                    branch_name,
                                    "Parallel branch execution failed: {}",
                                    e
                                );
                            }
                        }
                    }
                }

                _ => {
                    send_log!(
                        output,
                        MessageType::Warning,
                        branch_name,
                        "Skipping unimplemented test step..."
                    );
                }
            }
        }
    }
    .boxed()
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

fn merge_results(primary: &mut PipelineContext, secondary: PipelineContext) {
    for (key, secondary_val) in secondary.results.into_iter() {
        primary
            .results
            .entry(key)
            .or_default()
            .extend(secondary_val);
    }

    primary.src_proteome.extend(secondary.src_proteome);
    primary.tgt_proteome.extend(secondary.tgt_proteome);
}
