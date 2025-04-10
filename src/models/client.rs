use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, oneshot};
use crate::models::common::command::Command;
use crate::models::common::file::File;
use crate::models::common::message::Message;
use crate::models::common::short_peer_id::ShortPeerId;

pub struct Client {
    command_sender: mpsc::Sender<Command>,
    /// message and first 8 symbols of sender PeerId
    msg_receiver: mpsc::Receiver<(Message, ShortPeerId)>,
    download_path: String,
    current_file: Option<File>,

}

impl Client {
    pub fn new(msg_receiver: mpsc::Receiver<(Message, ShortPeerId)>, command_sender: mpsc::Sender<Command>) -> Self {

        let download_path = "./downloads".to_string();

        let current_file = None;

        let client = Self {
            command_sender,
            msg_receiver,
            download_path,
            current_file,
        };

        client.create_download_dir();

        client
    }

    pub async fn start_receiving(download_path: String, mut msg_receiver: mpsc::Receiver<(Message, ShortPeerId)>) {
        log::info!("Start receiving");

        tokio::spawn(async move {
            loop {
                if let Ok((msg, peer_id)) = msg_receiver.try_recv() {
                    println!("{}: {}", peer_id, msg);
                    msg.try_to_download_file(download_path.clone()).await;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });
    }

    pub async fn start_writing(mut current_file: Option<File>, command_sender: mpsc::Sender<Command>) {
        log::info!("Start writing");

        println!("----------------------------------");
        print_available_commands();
        println!("----------------------------------");

        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        while let Ok(Some(input)) = lines.next_line().await {
            let parts: Vec<&str> = input.splitn(2, char::is_whitespace).collect();
            match parts.as_slice() {
                ["subscribe", topic] => {
                    let (command, response_receiver) = Command::new_subscribe(topic.to_string());

                    if let Err(e) = command_sender.send(command).await {
                        log::error!("Error while sending command to swarm_manager: {e}");
                        println!("> Something went wrong, try again");
                        continue;
                    }

                    wait_for_response(response_receiver).await;
                }
                ["unsubscribe", topic] => {
                    let (command, response_receiver) = Command::new_unsubscribe(topic.to_string());

                    if let Err(e) = command_sender.send(command).await {
                        log::error!("Error while sending command to swarm_manager: {e}");
                        println!("> Something went wrong, try again");
                        continue;
                    }

                    wait_for_response(response_receiver).await;
                }
                ["msg", message] => {
                    let msg = Message::build(Some(message.to_string()), current_file.to_owned()).await;

                    let (command, response_receiver) = Command::new_send_message(msg);

                    if let Err(e) = command_sender.send(command).await {
                        log::error!("Error while sending message to swarm_manager: {e}");
                        println!("> Something went wrong, try again");
                        continue;
                    }

                    wait_for_response(response_receiver).await;

                    current_file = None;
                }
                ["file", path] => {
                    current_file = match File::from(path).await {
                        Ok(f) => {
                            log::info!("File read successfully");
                            println!("> File loaded successfully");
                            Some(f)
                        }
                        Err(e) => {
                            log::error!("Error while reading file: {e}");
                            println!("> Something went wrong while loading file");
                            None
                        }
                    }
                }
                _ => {
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
            Ok(_) => log::info!("Default dir created successfully"),
            Err(e) => log::error!("Couldn't create default dir: {e}"),
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
    println!("  subscribe <topic>");
    println!("  unsubscribe <topic> - unsubscribe topic ");
    println!("  msg <message> - send message");
    println!("  file <path> - adds file to next message if file exists");
}