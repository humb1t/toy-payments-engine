#![allow(clippy::unwrap_used)]

use std::{
    thread::{self},
    time::Duration,
};

use csv::ReaderBuilder;
use redb::{Database, backends::InMemoryBackend};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    runtime::Builder,
    sync::mpsc::{self, Receiver, Sender},
    time,
};
use tokio_stream::{
    StreamExt,
    wrappers::{LinesStream, ReceiverStream},
};
use toy_payments_engine::State;

const TCP_ADDRESS: &str = "127.0.0.1:8000";
const TEST_DELAY_MS: u64 = 100;
const TEST_REQUESTS_COUNT: usize = 1000;
const STREAM_TIMEOUT_S: u64 = 1;
const LITSENER_TIMEOUT_MS: u64 = 100;
type ChannelMessage = String;

#[test]
#[ignore = "do not run by default as it is slow"]
fn parallel_tcp_streams() {
    let (sender, receiver) = mpsc::channel::<ChannelMessage>(100);
    let parallelism = thread::available_parallelism().unwrap().get();
    let _simulation_handle = thread::spawn(move || {
        let simulation_rt = Builder::new_multi_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        simulation_rt.block_on(async {
            crate::simulation(parallelism, sender).await;
        });
    });
    let engine_handle = thread::spawn(move || {
        let engine_rt = Builder::new_current_thread().build().unwrap();
        engine_rt.block_on(async {
            crate::engine(receiver).await;
        });
    });
    let test_rt = Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();
    test_rt.block_on(async move {
        time::sleep(Duration::from_millis(TEST_DELAY_MS)).await;
        let payload_example = include_bytes!("./resourses/test_multiple_streams.csv");
        let mut payload_senders_handlers = Vec::new();
        for _ in 2..parallelism / 2 {
            let handle = tokio::spawn(async {
                let mut stream = TcpStream::connect(TCP_ADDRESS)
                    .await
                    .inspect_err(|error| eprintln!("{error:?}"))
                    .unwrap();
                for _ in 0..TEST_REQUESTS_COUNT {
                    stream.write_all(payload_example).await.unwrap();
                }
            });
            payload_senders_handlers.push(handle);
        }
        futures::future::join_all(payload_senders_handlers).await;
    });
    engine_handle.join().unwrap();
}

async fn simulation(parallelism: usize, sender: Sender<ChannelMessage>) {
    let mut tcp_listener_handlers = Vec::new();
    let listener = TcpListener::bind(TCP_ADDRESS).await.unwrap();
    for _ in 2..=parallelism / 2 {
        if time::timeout(Duration::from_millis(LITSENER_TIMEOUT_MS), async {
            let mut stream = listener
                .accept()
                .await
                .inspect_err(|error| eprintln!("{error:?}"))
                .unwrap();
            let sender_clone = sender.clone();
            let tcp_listener_handler = tokio::spawn(async move {
                // NOT OPTIMAL, BUT OKAY FOR TESTS
                let mut buf = Vec::new();
                stream.0.read_to_end(&mut buf).await.unwrap();
                let line_stream =
                    LinesStream::new(buf.lines()).timeout(Duration::from_secs(STREAM_TIMEOUT_S));
                tokio::pin!(line_stream);
                while let Ok(Some(line)) = line_stream.try_next().await {
                    sender_clone.send(line.unwrap()).await.unwrap();
                }
                // For debug purposes.
                // println!(
                //     "la:{},pa:{},pl:{}",
                //     stream.0.local_addr().unwrap(),
                //     stream.0.peer_addr().unwrap(),
                //     String::from_utf8(buf).unwrap_or("ERROR".to_string())
                // );
            });
            tcp_listener_handlers.push(tcp_listener_handler);
        })
        .await
        .is_err()
        {
            eprintln!("did not receive requests to connect within {LITSENER_TIMEOUT_MS} ms");
        }
    }
    // drop(sender);
    futures::future::join_all(tcp_listener_handlers).await;
}

async fn engine(receiver: Receiver<ChannelMessage>) {
    let database = Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .map_err(redb::Error::from)
        .unwrap();
    let mut state = State::new(database);
    let mut reader = ReceiverStream::new(receiver);
    let mut counter = 0;
    while let Some(new_transaction) = reader.next().await {
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(false)
            .from_reader(new_transaction.as_bytes());
        toy_payments_engine::process_transactions(csv_reader.deserialize(), &mut state)
            .inspect_err(|error| eprintln!("{error:?}"))
            .unwrap();
        counter += 1;
    }
    println!("processed {counter} transactions");
}
