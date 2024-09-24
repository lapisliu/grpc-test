use tonic::Request;
use data_transfer::data_transfer_client::DataTransferClient;
use data_transfer::Chunk;
use tokio::time::Instant;
use tokio_stream::wrappers::ReceiverStream;
use std::env;

pub mod data_transfer {
    tonic::include_proto!("datatransfer");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if the IP address is passed as an argument
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: client <server-ip>");
        std::process::exit(1);
    }

    let server_ip = &args[1];
    let server_url = format!("http://{}:50051", server_ip);

    // Create the gRPC client, connecting to the server at the provided IP address
    let mut client = DataTransferClient::connect(server_url).await?;

    // Create a channel for streaming chunks
    let (tx, rx) = tokio::sync::mpsc::channel::<Chunk>(4);

    // Spawn a task to send 256MB of data in chunks of 64KB
    tokio::spawn(async move {
        let message_size = 256 * 1024 * 1024; // 256MB
        let chunk_size = 64 * 1024; // 64KB chunks
        let num_chunks = message_size / chunk_size;

        for _ in 0..num_chunks {
            let chunk = Chunk {
                data: vec![0u8; chunk_size], // 64KB of zeroed data
            };

            // Send the chunk over the channel
            tx.send(chunk).await.unwrap();
        }
    });

    // Time the sending process
    let start = Instant::now();

    // Send the stream to the server
    let response = client
        .send_data(Request::new(ReceiverStream::new(rx)))
        .await?;

    // Record the elapsed time
    let elapsed_time = start.elapsed();
    println!("Response: {:?}", response);
    println!("Time to send 256MB: {:.2?}", elapsed_time);

    Ok(())
}
