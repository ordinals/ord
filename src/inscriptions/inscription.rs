use super::*;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct TransactionInscription {
  pub(crate) parsed_inscription: ParsedInscription,
  pub(crate) tx_input_index: u32,
  pub(crate) tx_input_offset: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Inscription {
  pub(crate) body: Option<Vec<u8>>,
  pub(crate) content_type: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ParsedInscription {
  pub(crate) inscription: Inscription,
  pub(crate) cursed: bool,
}

impl Inscription {
  #[cfg(test)]
  pub(crate) fn new(content_type: Option<Vec<u8>>, body: Option<Vec<u8>>) -> Self {
    Self { content_type, body }
  }

  pub(crate) fn from_transaction(tx: &Transaction) -> Vec<TransactionInscription> {
    let mut result = Vec::new();
    for (index, tx_in) in tx.input.iter().enumerate() {
      // if index != 0 { break }; // TODO: If before activation block height

      let Ok(inscriptions) = InscriptionParser::parse(&tx_in.witness) else { continue };

      result.extend(
        inscriptions
          .into_iter()
          .enumerate()
          .map(|(offset, parsed_inscription)| TransactionInscription {
            parsed_inscription: ParsedInscription {
              cursed: parsed_inscription.cursed || index != 0 || offset != 0,
              inscription: parsed_inscription.inscription,
            },
            tx_input_index: index as u32,
            tx_input_offset: offset as u32,
          })
          .collect::<Vec<TransactionInscription>>(),
      )
    }

    result
  }

  pub(crate) fn from_file(chain: Chain, path: impl AsRef<Path>) -> Result<Self, Error> {
    let path = path.as_ref();

    let body = fs::read(path).with_context(|| format!("io error reading {}", path.display()))?;

    if let Some(limit) = chain.inscription_content_size_limit() {
      let len = body.len();
      if len > limit {
        bail!("content size of {len} bytes exceeds {limit} byte limit for {chain} inscriptions");
      }
    }

    let content_type = Media::content_type_for_path(path)?;

    Ok(Self {
      body: Some(body),
      content_type: Some(content_type.into()),
    })
  }

  pub(crate) fn append_reveal_script_to_builder(
    &self,
    mut builder: script::Builder,
  ) -> script::Builder {
    builder = builder
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(PROTOCOL_ID);

    if let Some(content_type) = &self.content_type {
      builder = builder
        .push_slice(CONTENT_TYPE_TAG)
        .push_slice(content_type);
    }

    if let Some(body) = &self.body {
      builder = builder.push_slice(BODY_TAG);
      for chunk in body.chunks(520) {
        builder = builder.push_slice(chunk);
      }
    }

    builder.push_opcode(opcodes::all::OP_ENDIF)
  }

  pub(crate) fn append_reveal_script(&self, builder: script::Builder) -> Script {
    self.append_reveal_script_to_builder(builder).into_script()
  }

  pub(crate) fn media(&self) -> Media {
    if self.body.is_none() {
      return Media::Unknown;
    }

    let Some(content_type) = self.content_type() else {
      return Media::Unknown;
    };

    content_type.parse().unwrap_or(Media::Unknown)
  }

  pub(crate) fn body(&self) -> Option<&[u8]> {
    Some(self.body.as_ref()?)
  }

  pub(crate) fn into_body(self) -> Option<Vec<u8>> {
    self.body
  }

  pub(crate) fn content_length(&self) -> Option<usize> {
    Some(self.body()?.len())
  }

  pub(crate) fn content_type(&self) -> Option<&str> {
    str::from_utf8(self.content_type.as_ref()?).ok()
  }

  #[cfg(test)]
  pub(crate) fn to_witness(&self) -> Witness {
    let builder = script::Builder::new();

    let script = self.append_reveal_script(builder);

    let mut witness = Witness::new();

    witness.push(script);
    witness.push([]);

    witness
  }
}
