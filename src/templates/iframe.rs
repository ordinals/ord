use super::*;

pub(crate) struct Iframe {
  inscription_id: InscriptionId,
  thumbnail: bool,
  lazy_load: bool,
}

impl Iframe {
  pub(crate) fn lazy_load(inscription_id: InscriptionId) -> Trusted<Self> {
    Trusted(Self {
      inscription_id,
      thumbnail: true,
      lazy_load: true,
    })
  }

  pub(crate) fn thumbnail(inscription_id: InscriptionId) -> Trusted<Self> {
    Trusted(Self {
      inscription_id,
      thumbnail: true,
      lazy_load: false,
    })
  }

  pub(crate) fn main(inscription_id: InscriptionId) -> Trusted<Self> {
    Trusted(Self {
      inscription_id,
      thumbnail: false,
      lazy_load: false,
    })
  }
}

impl Display for Iframe {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.thumbnail {
      write!(f, "<a href=/inscription/{}>", self.inscription_id)?;
    }

    if self.lazy_load {
      write!(
        f,
        "<iframe class=lazyload-iframe sandbox=allow-scripts scrolling=no src=\"about:blank\" data-src=/preview/{}></iframe>",
        self.inscription_id
      )?;
    } else {
      write!(
        f,
        "<iframe sandbox=allow-scripts scrolling=no loading=lazy src=/preview/{}></iframe>",
        self.inscription_id
      )?;
    }

    if self.thumbnail {
      write!(f, "</a>",)?
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn lazy_load() {
    assert_regex_match!(
      Iframe::lazy_load(inscription_id(1))
      .0.to_string(),
      "<a href=/inscription/1{64}i1><iframe class=lazyload-iframe sandbox=allow-scripts scrolling=no src=\"about:blank\" data-src=/preview/1{64}i1></iframe></a>",
    );
  }

  #[test]
  fn thumbnail() {
    assert_regex_match!(
      Iframe::thumbnail(inscription_id(1))
      .0.to_string(),
      "<a href=/inscription/1{64}i1><iframe sandbox=allow-scripts scrolling=no loading=lazy src=/preview/1{64}i1></iframe></a>",
    );
  }

  #[test]
  fn main() {
    assert_regex_match!(
      Iframe::main(inscription_id(1)).0.to_string(),
      "<iframe sandbox=allow-scripts scrolling=no loading=lazy src=/preview/1{64}i1></iframe>",
    );
  }
}
