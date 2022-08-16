use {super::*, boilerplate::Display};

pub(crate) mod ordinal;
pub(crate) mod root;

#[derive(Display)]
pub(crate) struct IndexHtml {
  content: Box<dyn Content>,
}

pub(crate) trait Content: Display {
  fn title(&self) -> String;

  fn index(self) -> IndexHtml;
}
