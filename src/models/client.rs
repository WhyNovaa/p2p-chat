use crate::models::args::Args;
use crate::models::common::command::Command;
use crate::models::common::file::File;
use crate::models::common::message::Message;
use crate::models::common::short_peer_id::ShortPeerId;
use clap::Parser;
use std::path::PathBuf;
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, oneshot};

pub struct Client {
    command_sender: mpsc::Sender<Command>,
    /// message and first 8 symbols of sender PeerId
    msg_receiver: mpsc::Receiver<(Message, ShortPeerId)>,
    download_path: PathBuf,
    current_file: Option<File>,
}

impl Client {
    pub fn new(
        msg_receiver: mpsc::Receiver<(Message, ShortPeerId)>,
        command_sender: mpsc::Sender<Command>,
    ) -> Self {
        let args = Args::parse();

        let download_path = args.dir.filter(|path| path.is_dir()).unwrap_or_else(|| {
            log::warn!("Invalid dir or dir wasn't entered, using default download path");
            PathBuf::from("./downloads")
        });

        let client = Self {
            command_sender,
            msg_receiver,
            download_path,
            current_file: None,
        };

        client.create_download_dir();

        client
    }

    pub async fn start_receiving(
        download_path: PathBuf,
        mut msg_receiver: mpsc::Receiver<(Message, ShortPeerId)>,
    ) {
        log::info!("Start receiving");

        tokio::spawn(async move {
            loop {
                if let Some((msg, peer_id)) = msg_receiver.recv().await {
                    println!("{}: {}", peer_id, msg);
                    msg.try_to_download_file(&download_path).await;
                }
            }
        });
    }

    pub async fn start_writing(
        mut current_file: Option<File>,
        command_sender: mpsc::Sender<Command>,
    ) {
        log::info!("Start writing");

        println!("----------------------------------");
        print_available_commands();
        println!("----------------------------------");

        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        while let Ok(Some(input)) = lines.next_line().await {
            match parse::parse_command(&input) {
                Ok((_, cmd)) => match cmd {
                    parse::UserCommand::Subscribe(topic) => {
                        let (command, response_receiver) =
                            Command::new_subscribe(topic.to_string());

                        if let Err(e) = command_sender.send(command).await {
                            log::error!("Error while sending command to swarm_manager: {e}");
                            println!("> Something went wrong, try again");
                            continue;
                        }

                        wait_for_response(response_receiver).await;
                    }
                    parse::UserCommand::Unsubscribe(topic) => {
                        let (command, response_receiver) =
                            Command::new_unsubscribe(topic.to_string());

                        if let Err(e) = command_sender.send(command).await {
                            log::error!("Error while sending command to swarm_manager: {e}");
                            println!("> Something went wrong, try again");
                            continue;
                        }

                        wait_for_response(response_receiver).await;
                    }
                    parse::UserCommand::Msg(message) => {
                        let msg = Message::build(
                            Some(message.to_string()),
                            Option::take(&mut current_file),
                        )
                        .await;

                        let (command, response_receiver) = Command::new_send_message(msg);

                        if let Err(e) = command_sender.send(command).await {
                            log::error!("Error while sending message to swarm_manager: {e}");
                            println!(
                                "> Something went wrong while sending message: {e}, try again"
                            );
                            continue;
                        }

                        wait_for_response(response_receiver).await;
                    }
                    parse::UserCommand::File(path) => {
                        current_file = match File::from(path).await {
                            Ok(f) => {
                                log::info!("File read successfully");
                                println!("> File loaded successfully");
                                Some(f)
                            }
                            Err(e) => {
                                log::error!("Error while reading file: {e}");
                                println!("> Something went wrong while loading file: {e}");
                                None
                            }
                        }
                    }
                },
                Err(_) => {
                    println!("----------------------------------");
                    println!("Wrong command!");
                    print_available_commands();
                    println!("----------------------------------");
                }
            }
        }
    }

    pub async fn run(self) {
        log::info!("Running client...");

        Self::start_receiving(self.download_path, self.msg_receiver).await;
        Self::start_writing(self.current_file, self.command_sender).await;
    }

    pub fn create_download_dir(&self) {
        match std::fs::create_dir(&self.download_path) {
            Ok(_) => log::info!("Download dir created successfully"),
            Err(e) => match e.kind() {
                io::ErrorKind::AlreadyExists => log::warn!("Download directory already exists"),
                _ => log::error!("Couldn't create default dir: {e}"),
            },
        }
    }
}

pub async fn wait_for_response(response_receiver: oneshot::Receiver<String>) {
    tokio::select! {
        response = response_receiver => {
            match response {
                Ok(response) => println!("> {}", response),
                Err(_) => log::error!("Failed to receive response."),
            }
        }
        _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
            log::info!("Timed out waiting for response.");
        }
    }
}

fn print_available_commands() {
    println!("Available commands:");
    println!("  subscribe <topic> - subscribe topic");
    println!("  unsubscribe <topic> - unsubscribe topic ");
    println!("  msg <message> - send message");
    println!("  file <path> - add file to next message if file exists");
}

mod parse {
    use nom::IResult;
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::combinator::map;
    use nom::sequence::preceded;

    #[derive(Debug)]
    pub enum UserCommand {
        Subscribe(String),
        Unsubscribe(String),
        Msg(String),
        File(String),
    }

    pub fn parse_command(input: &str) -> IResult<&str, UserCommand> {
        alt((
            map(preceded(tag("subscribe "), rest_str), |topic: &str| {
                UserCommand::Subscribe(topic.to_string())
            }),
            map(preceded(tag("unsubscribe "), rest_str), |topic: &str| {
                UserCommand::Unsubscribe(topic.to_string())
            }),
            map(preceded(tag("msg "), rest_str), |msg: &str| {
                UserCommand::Msg(msg.to_string())
            }),
            map(preceded(tag("file "), rest_str), |path: &str| {
                UserCommand::File(path.to_string())
            }),
        ))(input)
    }

    pub fn rest_str(input: &str) -> IResult<&str, &str> {
        // просто возвращает оставшуюся строку
        Ok(("", input.trim()))
    }
}
