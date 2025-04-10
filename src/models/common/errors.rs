use libp2p::gossipsub::PublishError;

#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("Couldn't read file")]
    CouldntReadFile,
    #[error("Filename is not in UTF-8")]
    WrongEncoding,
}

#[derive(Debug, thiserror::Error)]
pub enum SendingError {
    #[error("Swarm is not subscribed to the topic")]
    NoSubscribedTopic,
    #[error("Message encoding error")]
    CantEncodeMessage,
    #[error("Other error {0}")]
    Other(#[from] PublishError),
}
