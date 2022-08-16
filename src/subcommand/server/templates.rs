use {super::*, boilerplate::Display};

pub(crate) mod ordinal;
pub(crate) mod root;

#[derive(Display)]
pub(crate) struct BaseHtml {
  page: Box<dyn Page>,
}

impl BaseHtml {
  pub(crate) fn new<T: Page + 'static>(page: T) -> Self {
    Self {
      page: Box::new(page),
    }
  }
}

pub(crate) trait Page: Display {
  fn title(&self) -> String;
}
