use std::sync::Arc;

use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use crate::index::event::Event;

pub struct EventPublisher {
  pub event_sender: mpsc::Sender<Event>,
  #[allow(dead_code)]
  runtime: Arc<Runtime>,
}

impl EventPublisher {
  pub fn new() -> Self {
    let (event_sender, mut event_receiver) = mpsc::channel::<Event>(128);
    let runtime = Arc::new(Runtime::new().unwrap());

    let rt_clone = runtime.clone();
    rt_clone.spawn(async move {
      while let Some(event) = event_receiver.recv().await {
        EventPublisher::handle_event(event);
      }
    });

    EventPublisher {
      event_sender,
      runtime,
    }
  }

  fn handle_event(event: Event) {
    println!("Received event: {:?}", event);
  }
}
