use tokio::time::{sleep, Duration};
use tokio::sync::mpsc;

enum LogMessage {
    Info(String),
    Error(String),
    Shutdown,
}

#[tokio::main]
async fn main() {
    let (sender, mut receiver) = mpsc::channel::<LogMessage>(32);

    let sender1 = sender.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            sender1.send(LogMessage::Info("[TASK1] Some msg from task1".to_string())).await.unwrap();
        }

    });

    let sender2 = sender.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(2)).await;
            sender2.send(LogMessage::Error("[TASK2] Some msg from task2".to_string())).await.unwrap();
        }

    });

    let sender3 = sender.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(7)).await;
            sender3.send(LogMessage::Shutdown).await.unwrap();
        }

    });

    let logger_task = tokio::spawn(async move {
        println!("Hello from logger");
        while let Some(message) = receiver.recv().await {
            match message {
                LogMessage::Info(msg) => {
                    println!("{}", msg);
                },
                LogMessage::Error(msg) => {
                    eprintln!("{}", msg);
                }
                LogMessage::Shutdown => {
                    println!("Logger shutdown");
                    break;
                }
            }
        }
    });
    logger_task.await.unwrap();
}