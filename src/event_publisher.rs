use std::thread;

use anyhow::{anyhow, Result};
use lapin::{BasicProperties, Channel, Connection, ConnectionProperties, options::BasicPublishOptions};
use serde_json::to_vec;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;

use crate::index::event::Event;
use crate::settings::Settings;

pub struct EventPublisher {
  pub(crate) sender: mpsc::Sender<Event>,
}

impl EventPublisher {
  pub fn new(settings: &Settings) -> Result<Self, anyhow::Error> {
    let url = settings.rabbitmq_url()
      .ok_or_else(|| anyhow!("RabbitMQ URL is not set in settings"))?.clone();
    let username = settings.rabbitmq_username()
      .ok_or_else(|| anyhow!("RabbitMQ username is not set in settings"))?.clone();
    let password = settings.rabbitmq_password()
      .ok_or_else(|| anyhow!("RabbitMQ password is not set in settings"))?.clone();
    let exchange = settings.rabbitmq_exchange()
      .ok_or_else(|| anyhow!("RabbitMQ exchange is not set in settings"))?.clone();

    let addr = format!("amqp://{}:{}@{}", username, password, url);

    let (sender, receiver) = mpsc::channel::<Event>(128);

    thread::spawn(move || {
      let runtime = tokio::runtime::Runtime::new().unwrap();
      runtime.block_on(async {
        let conn = Connection::connect(&addr, ConnectionProperties::default()).await.unwrap();
        let channel = conn.create_channel().await.unwrap();
        Self::run_event_loop(receiver, channel, exchange).await;
      });
    });

    Ok(EventPublisher { sender })
  }

  async fn run_event_loop(mut receiver: Receiver<Event>, channel: Channel, exchange: String) {
    while let Some(event) = receiver.recv().await {
      let message = to_vec(&event).expect("Failed to serialize event");
      let _ = channel.basic_publish(
        &exchange,
        "",
        BasicPublishOptions::default(),
        &message,
        BasicProperties::default(),
      ).await.expect("Failed to publish message");
    }
  }
}
