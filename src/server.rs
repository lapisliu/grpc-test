use tonic::{transport::Server, Request, Response, Status};
use tokio_stream::StreamExt;
use std::time::Instant;

pub mod data_transfer {
    tonic::include_proto!("datatransfer");
}

use data_transfer::data_transfer_server::{DataTransfer, DataTransferServer};
use data_transfer::{Chunk, Ack};

#[derive(Default)]
pub struct MyDataTransfer {}

#[tonic::async_trait]
impl DataTransfer for MyDataTransfer {
    async fn send_data(
        &self,
        request: Request<tonic::Streaming<Chunk>>, // Accept a stream of data chunks
    ) -> Result<Response<Ack>, Status> {
        let mut stream = request.into_inner();
        let mut received_bytes: usize = 0;
        let start = Instant::now();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    received_bytes += chunk.data.len();
                }
                Err(e) => {
                    return Err(Status::unknown(format!("Error receiving data: {}", e)));
                }
            }
        }

        let elapsed_time = start.elapsed();
        println!("Received {} bytes in {:.2?}", received_bytes, elapsed_time);

        let reply = Ack {
            message: format!("Received {} bytes", received_bytes),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let data_transfer = MyDataTransfer::default();

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(DataTransferServer::new(data_transfer))
        .serve(addr)
        .await?;

    Ok(())
}
