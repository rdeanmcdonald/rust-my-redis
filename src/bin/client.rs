use bytes::Bytes;
use futures::future::join_all;
use mini_redis::client;
use std::{error::Error, time::Duration};
use tokio::{
    spawn,
    sync::{
        mpsc::{channel, Receiver, Sender},
        oneshot,
    },
    task::JoinHandle,
};

type Response<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        sender: oneshot::Sender<Response<Option<Bytes>>>,
    },
    Set {
        key: String,
        val: Bytes,
        sender: oneshot::Sender<Response<()>>,
    },
}

async fn client(mut receiver: Receiver<Command>) {
    // Open a conn to the redis server
    let mut client = client::connect("127.0.0.1:6379").await.unwrap();

    loop {
        let a = receiver.recv().await.unwrap();

        match a {
            Command::Get { key, sender } => {
                let result = client.get(&key).await;
                sender.send(result);
            }
            Command::Set {key, val, sender} => {
                let result = client.set(&key, val).await;
                sender.send(result);
            }
            // Publish(Publish) => {},
            // Subscribe(Subscribe) => {},
            // Unsubscribe(Unsubscribe) => {},
            // Unknown(Unknown) => {},
        }
    }
}

async fn set(sender: Sender<Command>, key: &str, val: String) -> Response<()> {
    let (oneshot_sender, oneshot_receiver) = oneshot::channel();

    sender
        .send(Command::Set {
            key: key.into(),
            val: Bytes::from(val),
            sender: oneshot_sender,
        })
        .await
        .unwrap();

    oneshot_receiver.await?
    // match result {
    //     Ok(data) => println!("SUCCESSFULL SET"),
    //     Err(e) => println!("ERROR ON SET: {:?}", e),
    // }
}

async fn get(sender: Sender<Command>, key: &str) -> Response<Option<Bytes>> {
    let (oneshot_sender, oneshot_receiver) = oneshot::channel();

    sender
        .send(Command::Get {
            key: key.into(),
            sender: oneshot_sender,
        })
        .await
        .unwrap();

    oneshot_receiver.await?
    // match result {
    //     Ok(data) => {
    //         match data {
    //             Ok(val) => println!("GOT VALUE FROM REDIS: {:?}", val),
    //             Err(e) => println!("ERROR ON GET: {:?}", e),
    //         }
    //     }
    //     Err(e) => println!("ERROR ON GET: {:?}", e),
    // }
}

async fn setter(tx: Sender<Command>) {
    for i in 0..10 {
        let key = format!("key_{}", i);
        let val = format!("val_{}", i);
        tokio::time::sleep(Duration::from_millis(1000)).await;
        let _result = set(tx.clone(), &key, val.into()).await;
    }
}

async fn loop_till_key_returned(tx: Sender<Command>, key: String) {
    loop {
        match get(tx.clone(), &key).await {
            Ok(maybe_val) => match maybe_val {
                Some(val) => {
                    println!("FOUND VAL {:?} FOR KEY {}", val, key);
                    break;
                }
                None => {
                    println!("KEY {} NOT YET FOUND", key);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            },
            Err(e) => {
                println!("ERROR ON REQUEST FOR KEY {}: {}", key, e);
                break;
            }
        }
    }
}

async fn getter(tx: Sender<Command>) {
    let getter_handles: Vec<JoinHandle<()>> = (0..10)
        .into_iter()
        .map(|x| spawn(loop_till_key_returned(tx.clone(), format!("key_{}", x))))
        .collect();

    let _results = join_all(getter_handles).await;
}

#[tokio::main]
async fn main() {
    let (tx, rx) = channel(100);

    let work = vec![
        spawn(client(rx)),
        spawn(setter(tx.clone())),
        spawn(getter(tx.clone())),
    ];

    let _results = join_all(work).await;
}
