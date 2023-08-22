use super::*;

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewAudioHtml {
  pub(crate) inscription_id: InscriptionId,
}

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewImageHtml {
  pub(crate) inscription_id: InscriptionId,
}

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewPdfHtml {
  pub(crate) inscription_id: InscriptionId,
}

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewTextHtml<'a> {
  pub(crate) text: &'a str,
}

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewUnknownHtml;

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewVideoHtml {
  pub(crate) inscription_id: InscriptionId,
}
