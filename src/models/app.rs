use crate::models::client::client::Client;
use crate::models::swarm::swarm_manager::SwarmManager;

pub struct App {
    client: Client,
    swarm_manager: SwarmManager,
}