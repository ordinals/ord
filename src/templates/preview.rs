use super::*;

#[derive(Boilerplate)]
pub(crate) struct PreviewAudioHtml {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) inscription_number: i32,
}

#[derive(Boilerplate)]
pub(crate) struct PreviewCodeHtml {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) language: media::Language,
  pub(crate) inscription_number: i32,
}

#[derive(Boilerplate)]
pub(crate) struct PreviewFontHtml {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) inscription_number: i32,
}

#[derive(Boilerplate)]
pub(crate) struct PreviewImageHtml {
  pub(crate) image_rendering: ImageRendering,
  pub(crate) inscription_id: InscriptionId,
  pub(crate) inscription_number: i32,
}

#[derive(Boilerplate)]
pub(crate) struct PreviewMarkdownHtml {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) inscription_number: i32,
}

#[derive(Boilerplate)]
pub(crate) struct PreviewModelHtml {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) inscription_number: i32,
}

#[derive(Boilerplate)]
pub(crate) struct PreviewPdfHtml {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) inscription_number: i32,
}

#[derive(Boilerplate)]
pub(crate) struct PreviewTextHtml {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) inscription_number: i32,
}

#[derive(Boilerplate)]
pub(crate) struct PreviewUnknownHtml;

#[derive(Boilerplate)]
pub(crate) struct PreviewVideoHtml {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) inscription_number: i32,
}
