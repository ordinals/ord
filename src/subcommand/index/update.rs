use event_hasher::EventHasher;

use super::*;

pub(crate) fn run(settings: Settings) -> SubcommandResult {

  let (event_sender, mut event_receiver) = tokio::sync::mpsc::channel(1024);

  let hasher = EventHasher::create(&settings)?;

  let index = Index::open_with_event_sender(&settings, Some(event_sender))?;

  thread::spawn(move || {
    let _ = index.update();
  });

  let result = hasher.run(&mut event_receiver)?;

  log::info!(
    "update index ok! current_block_height {}, block_event_hash {}, cumulative_block_event_hash {}",
    result.0,
    result.1,
    result.2
  );

  Ok(None)
}
