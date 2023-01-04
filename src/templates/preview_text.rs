use super::*;

#[derive(boilerplate::Boilerplate)]
pub(crate) struct PreviewTextHtml<'a> {
  pub(crate) text: &'a str,
}
