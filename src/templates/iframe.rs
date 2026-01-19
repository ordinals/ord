use super::*;

pub(crate) struct Iframe {
  inscription_id: InscriptionId,
  kind: IframeKind,
}

enum IframeKind {
  Item { i: usize, id: InscriptionId },
  Main,
  Thumbnail,
}

impl Iframe {
  pub(crate) fn item(inscription_id: InscriptionId, i: usize, id: InscriptionId) -> Trusted<Self> {
    Trusted(Self {
      inscription_id,
      kind: IframeKind::Item { i, id },
    })
  }

  pub(crate) fn main(inscription_id: InscriptionId) -> Trusted<Self> {
    Trusted(Self {
      inscription_id,
      kind: IframeKind::Main,
    })
  }

  pub(crate) fn thumbnail(inscription_id: InscriptionId) -> Trusted<Self> {
    Trusted(Self {
      inscription_id,
      kind: IframeKind::Thumbnail,
    })
  }
}

impl Display for Iframe {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.kind {
      IframeKind::Item { i, id } => {
        write!(
          f,
          "<a href=/gallery/{}/{i}>\
            <iframe sandbox=allow-scripts scrolling=no loading=lazy src=/preview/{id}>\
            </iframe>\
          </a>",
          self.inscription_id,
        )
      }
      IframeKind::Main => {
        write!(
          f,
          "<iframe sandbox=allow-scripts loading=lazy src=/preview/{}></iframe>",
          self.inscription_id,
        )
      }
      IframeKind::Thumbnail => {
        write!(
          f,
          "<a href=/inscription/{}>\
            <iframe sandbox=allow-scripts scrolling=no loading=lazy src=/preview/{}>\
            </iframe>\
          </a>",
          self.inscription_id, self.inscription_id,
        )
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn gallery_item() {
    assert_regex_match!(
      Iframe::item(inscription_id(1), 2, inscription_id(3))
        .0
        .to_string(),
      "<a href=/gallery/1{64}i1/2><iframe sandbox=allow-scripts scrolling=no loading=lazy src=/preview/3{64}i3></iframe></a>",
    );
  }

  #[test]
  fn main() {
    assert_regex_match!(
      Iframe::main(inscription_id(1)).0.to_string(),
      "<iframe sandbox=allow-scripts loading=lazy src=/preview/1{64}i1></iframe>",
    );
  }

  #[test]
  fn thumbnail() {
    assert_regex_match!(
      Iframe::thumbnail(inscription_id(1)).0.to_string(),
      "<a href=/inscription/1{64}i1><iframe sandbox=allow-scripts scrolling=no loading=lazy src=/preview/1{64}i1></iframe></a>",
    );
  }
}
