# Command
```
docker-compose --env-file .env up -d
```

# Runebeta code modified files
1. Cargo.toml
2. src/lib.rs
3. src/index.rs
4. src/index/updater.rs
5. src/index/updater/rune_updater.rs
6. diesel.toml
7. run.sh
8. docker-compose.xml
9. runebeta folder
10. migrations folder

## Cargo toml
```
# runebeta
diesel = { version = "2.1", features = ["postgres", "uuid", "serde_json", "numeric"] }
bigdecimal =  { version = "0.4.3" }
deadpool-diesel = { version = "0.4", features = ["postgres"] }
dotenvy = "0.15"
# End runebeta

```
## src/lib.rs
Add runebeta module
mod runebeta;
pub use runebeta::*,

## src/index.rs
Add extension instance from the line 624
```
  pub fn update(&self) -> Result {
    loop {
      let wtx = self.begin_write()?;
      let extension = Arc::new(Mutex::new(IndexExtension::new(self.settings.chain())));
      ....

      match updater.update_index(wtx, extension) {
        ...
      }

```
## src/index/updater.rs

```
impl<'index> Updater<'index> {
  pub(crate) fn update_index(
    &mut self,
    mut wtx: WriteTransaction,
    extension: Arc<Mutex<IndexExtension>>,
  ) -> Result {
    ...

    while let Ok(block) = rx.recv() {
      //
      let index_inscriptions = self.height >= self.index.first_inscription_height
        && self.index.settings.index_inscriptions();
      if index_inscriptions && block.txdata.len() > 0 {
        //Index block with data only
        if let Ok(mut extension) = extension.try_lock() {
          let _res = extension.index_block(self.height as i64, &block.header, &block.txdata);
        }
      }
      self.index_block(
        &mut outpoint_sender,
        &mut value_receiver,
        &mut wtx,
        extension.clone(),
        block,
        &mut value_cache,
      )?;
      ...
      if uncommitted > 0 {
        self.commit(wtx, extension.clone(), value_cache)?;
      }

    }
```

```
    let mut rune_updater = RuneUpdater {
        ...
        extension, // Add externsion here
      };
```

## src/index/updater/rune_updater.rs

```
  // Sort balances by id so tests can assert balances in a fixed order
  balances.sort();
  if let Ok(mut extension) = self.extension.try_lock() {
    let _res = extension.index_outpoint_balances(
      &txid,
      vout as i32,
      &balances
        .iter()
        .map(|(rune_id, balance)| (rune_id.clone(), BigDecimal::from(balance.0)))
        .collect(),
    );
  }
```

Line 286 in the function create_rune_entry
```
    /*
     * Taivv April 03, index data to postgres
     */
    if let Ok(mut extension) = self.extension.try_lock() {
      let _ = extension.index_transaction_rune_entry(&txid, &id, &entry);
    }

    self.id_to_entry.insert(id.store(), entry.store())?;
```
