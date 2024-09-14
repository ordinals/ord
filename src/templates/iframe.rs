use super::*;

pub(crate) struct Iframe {
  inscription_id: InscriptionId,
  thumbnail: bool,
}

impl Iframe {
  pub(crate) fn thumbnail(inscription_id: InscriptionId) -> Trusted<Self> {
    Trusted(Self {
      inscription_id,
      thumbnail: true,
    })
  }

  pub(crate) fn main(inscription_id: InscriptionId) -> Trusted<Self> {
    Trusted(Self {
      inscription_id,
      thumbnail: false,
    })
  }
}

impl Display for Iframe {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    if self.thumbnail {
      write!(f, "<a href=/inscription/{}>", self.inscription_id)?;
    }

    write!(
      f,
      "<iframe sandbox=allow-scripts loading=lazy src=/preview/{}></iframe>",
      self.inscription_id
    )?;

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
  fn thumbnail() {
    assert_regex_match!(
      Iframe::thumbnail(inscription_id(1))
      .0.to_string(),
      "<a href=/inscription/1{64}i1><iframe sandbox=allow-scripts loading=lazy src=/preview/1{64}i1></iframe></a>",
    );
  }

  #[test]
  fn main() {
    assert_regex_match!(
      Iframe::main(inscription_id(1)).0.to_string(),
      "<iframe sandbox=allow-scripts loading=lazy src=/preview/1{64}i1></iframe>",
    );
  }
}
