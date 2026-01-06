use std::sync::Arc;

use async_channel::{Receiver, Sender};
use clap::Parser;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

enum Message {
    PdfPath(String),
    Shutdown,
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Password file with a passwords on each new line
    #[arg(short, long, value_name = "FILE")]
    passord_file: String,

    /// Single PDF file to process
    #[arg(
        short = 'f',
        long,
        value_name = "FILE",
        required_unless_present = "directory",
        conflicts_with = "directory"
    )]
    pdf_file: Option<String>,

    /// Directory containing PDF files to process
    #[arg(short, long, value_name = "DIR", conflicts_with = "pdf_file")]
    directory: Option<String>,

    /// Scan directory recursively to process every PDF file
    #[arg(short, long, requires = "directory", default_value_t = false)]
    recursive: bool,

    /// Show on screen a name of pdf file and a password used to open it
    #[arg(short, long, default_value_t = false)]
    show_password: bool,

    /// Number of worker threads to use for processing
    #[arg(short, long)]
    workers: Option<usize>,
}

async fn read_password_file(filename: &str) -> Vec<String> {
    let file = File::open(filename).await.unwrap();
    let reader = BufReader::new(file);

    let mut passwords: Vec<String> = Vec::new();

    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await.unwrap() {
        passwords.push(line);
    }

    passwords
}

async fn worker(
    id: String,
    passwords: Arc<Vec<String>>,
    input_receiver: Receiver<Message>,
    result_sender: Sender<String>,
) {
    println!("Worker started: {}", id);

    while let Ok(msg) = input_receiver.recv().await {
        match msg {
            Message::PdfPath(pdf_path) => println!("Received pdf path: {}", pdf_path),
            Message::Shutdown => break,
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let passwords = read_password_file(&args.passord_file).await;
    let passwords = Arc::new(passwords);

    // Get number of workers (from CLI arg or default to number of CPUs)
    let num_workers = args.workers.unwrap_or_else(|| num_cpus::get());
    println!("Starting {} workers", num_workers);

    // Create channels for communication
    let (task_sender, task_receiver) = async_channel::unbounded::<Message>();
    let (result_sender, result_receiver) = async_channel::unbounded::<String>();

    // Spawn workers
    let mut handles = Vec::new();
    for i in 0..num_workers {
        let worker_id = format!("worker-{}", i);
        let passwords_clone = Arc::clone(&passwords);
        let task_receiver_clone = task_receiver.clone();
        let result_sender_clone = result_sender.clone();

        let handle = tokio::spawn(async move {
            worker(
                worker_id,
                passwords_clone,
                task_receiver_clone,
                result_sender_clone,
            )
            .await;
        });

        handles.push(handle);
    }

    println!("All workers started");

    // TODO: Send PDF paths to workers via task_sender
    // For now, just send shutdown messages
    for _ in 0..num_workers {
        task_sender.send(Message::Shutdown).await.unwrap();
    }

    // Wait for all workers to complete
    for handle in handles {
        handle.await.unwrap();
    }

    println!("All workers completed");
}
