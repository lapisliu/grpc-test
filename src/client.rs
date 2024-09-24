use tonic::Request;
use data_transfer::data_transfer_client::DataTransferClient;
use data_transfer::Chunk;
use tokio::time::Instant;
use tokio_stream::wrappers::ReceiverStream;

pub mod data_transfer {
    tonic::include_proto!("datatransfer");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    // Create the gRPC client
    let mut client = DataTransferClient::connect("http://[::1]:50051").await?;
    let elapsed_time = start.elapsed();
    println!("Time to connect: {:.2?}", elapsed_time);

    // Create a channel for streaming chunks
    let (mut tx, rx) = tokio::sync::mpsc::channel::<Chunk>(4);

    // Spawn a task to send 256MB of data in chunks of 64KB
    tokio::spawn(async move {
        let message_size = 1024 * 1024 * 1024; // 256MB
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
