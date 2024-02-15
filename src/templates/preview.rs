use super::*;

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewAudioHtml {
  pub(crate) inscription_id: InscriptionId,
}

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewCodeHtml {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) language: media::Language,
}

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewFontHtml {
  pub(crate) inscription_id: InscriptionId,
}

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewImageHtml {
  pub(crate) image_rendering: ImageRendering,
  pub(crate) inscription_id: InscriptionId,
}

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewMarkdownHtml {
  pub(crate) inscription_id: InscriptionId,
}

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewModelHtml {
  pub(crate) inscription_id: InscriptionId,
}

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewPdfHtml {
  pub(crate) inscription_id: InscriptionId,
}

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewTextHtml {
  pub(crate) inscription_id: InscriptionId,
}

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewUnknownHtml;

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewVideoHtml {
  pub(crate) inscription_id: InscriptionId,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn image_rendering() {
    assert!(PreviewImageHtml {
      inscription_id: "0000000000000000000000000000000000000000000000000000000000000000i0"
        .parse()
        .unwrap(),
      image_rendering: ImageRendering::Auto,
    }
    .to_string()
    .contains("image-rendering: auto;"));

    assert!(PreviewImageHtml {
      inscription_id: "0000000000000000000000000000000000000000000000000000000000000000i0"
        .parse()
        .unwrap(),
      image_rendering: ImageRendering::Pixelated,
    }
    .to_string()
    .contains("image-rendering: pixelated;"));
  }
}
