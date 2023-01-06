use super::*;

pub(crate) const HTML: &str = "text/html;charset=utf-8";
pub(crate) const SVG: &str = "image/svg+xml";
pub(crate) const TEXT: &str = "text/plain;charset=utf-8";

const TABLE: &[(&str, bool, &[&str])] = &[
  ("image/apng", true, &["apng"]),
  ("image/gif", true, &["gif"]),
  ("image/jpeg", true, &["jpg", "jpeg"]),
  ("image/png", true, &["png"]),
  ("image/webp", true, &["webp"]),
  (HTML, false, &["html"]),
  (SVG, true, &["svg"]),
  (TEXT, false, &["txt"]),
];

lazy_static! {
  static ref IMAGE_CONTENT_TYPES: HashSet<&'static str> = TABLE
    .iter()
    .filter(|(_, image, _)| *image)
    .map(|(content_type, _, _)| *content_type)
    .collect();
}

pub(crate) fn is_image(content_type: &str) -> bool {
  IMAGE_CONTENT_TYPES.contains(content_type)
}

pub(crate) fn for_extension(extension: &str) -> Result<&'static str, Error> {
  for (content_type, _, extensions) in TABLE {
    if extensions.contains(&extension) {
      return Ok(content_type);
    }
  }

  Err(anyhow!(
    "unsupported file extension `.{extension}`, supported extensions: {}",
    TABLE
      .iter()
      .map(|(_, _, extensions)| extensions[0])
      .collect::<Vec<&str>>()
      .join(" "),
  ))
}

#[cfg(test)]
mod tests {
  #[test]
  fn is_image() {
    assert!(super::is_image("image/apng"));
    assert!(!super::is_image("foo"));
  }

  #[test]
  fn for_extension() {
    assert_eq!(super::for_extension("jpg").unwrap(), "image/jpeg");
    assert_eq!(super::for_extension("jpeg").unwrap(), "image/jpeg");
    assert_eq!(
      super::for_extension("foo").unwrap_err().to_string(),
      "unsupported file extension `.foo`, supported extensions: apng gif jpg png webp html svg txt"
    );
  }
}
