use std::{fs, io, path::PathBuf, thread};

use thiserror::Error;
use tokio::sync::{
    mpsc::{UnboundedSender, unbounded_channel},
    oneshot,
};
use tokio_uring::fs::File;

#[derive(Debug, Error)]
pub enum Error {
    #[error("channel error, something bad happened")]
    Channel,
    #[error("IO error")]
    Io(#[from] io::Error),
}

enum Command {
    Write {
        path: PathBuf,
        data: Vec<u8>,
        respond_to: oneshot::Sender<Result<(), io::Error>>,
    },
    Read {
        path: PathBuf,
        respond_to: oneshot::Sender<Result<Vec<u8>, io::Error>>,
    },
}

/// `DiskInterface` is a simple wrapper around `io_uring` features to enable truly asynchronous
/// fs operations.
pub struct DiskInterface {
    channel: UnboundedSender<Command>,
}

impl DiskInterface {
    pub fn new() -> Self {
        let (sender, mut receiver) = unbounded_channel();

        thread::spawn(move || {
            tokio_uring::start(async move {
                while let Some(command) = receiver.recv().await {
                    match command {
                        Command::Write {
                            path,
                            data,
                            respond_to,
                        } => tokio_uring::spawn(async move {
                            respond_to
                                .send(try {
                                    let file = File::create(path).await?;
                                    file.write_all_at(data, 0).await.0?;
                                    file.sync_all().await?;
                                    file.close().await?;
                                })
                                .unwrap()
                        }),
                        Command::Read { path, respond_to } => tokio_uring::spawn(async move {
                            respond_to
                                .send(try {
                                    let file = File::open(&path).await?;
                                    let metadata = fs::metadata(&path)?;
                                    let size = metadata.len() as usize;
                                    let mut buffer = Vec::with_capacity(size);
                                    while buffer.len() < size {
                                        let (res, chunk) = file
                                            .read_at(
                                                vec![0; size - buffer.len()],
                                                buffer.len() as u64,
                                            )
                                            .await;
                                        let read_size = res?;
                                        if read_size == 0 {
                                            // We have unexpectedly reached the end of the file,
                                            // probably due to a data race of us reading the
                                            // file, and someone else writing on it.
                                            break;
                                        }
                                        buffer.extend_from_slice(&chunk[..read_size]);
                                    }
                                    file.close().await?;
                                    buffer
                                })
                                .unwrap()
                        }),
                    };
                }
            })
        });
        Self { channel: sender }
    }

    pub async fn read(&self, path: PathBuf) -> Result<Vec<u8>, Error> {
        let (sender, receiver) = oneshot::channel();
        self.channel
            .send(Command::Read {
                path,
                respond_to: sender,
            })
            .map_err(|_error| Error::Channel)?;
        Ok(receiver.await.map_err(|_error| Error::Channel)??)
    }

    pub async fn write(&self, path: PathBuf, data: Vec<u8>) -> Result<(), Error> {
        let (sender, receiver) = oneshot::channel();
        self.channel
            .send(Command::Write {
                path,
                data,
                respond_to: sender,
            })
            .map_err(|_error| Error::Channel);
        receiver.await.map_err(|_error| Error::Channel)??;
        Ok(())
    }
}
