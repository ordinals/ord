use super::*;

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Content {
  Audio,
  Iframe,
  Image,
  Text,
}

impl Content {
  pub(crate) fn content_type_for_extension(extension: &str) -> Result<&'static str, Error> {
    // todo: test this
    let extension = extension.to_lowercase();
    for (content_type, _, extensions) in TABLE {
      if extensions.contains(&extension.as_str()) {
        return Ok(content_type);
      }
    }

    let mut extensions = TABLE
      .iter()
      .map(|(_, _, extensions)| extensions[0])
      .collect::<Vec<&str>>();
    extensions.sort();

    Err(anyhow!(
      "unsupported file extension `.{extension}`, supported extensions: {}",
      extensions.join(" "),
    ))
  }
}

impl FromStr for Content {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    for entry in TABLE {
      if entry.0 == s {
        return Ok(entry.1);
      }
    }

    Err(anyhow!("unknown content type: {s}"))
  }
}

const TABLE: &[(&str, Content, &[&str])] = &[
  ("audio/flac", Content::Audio, &["flac"]),
  ("audio/mpeg", Content::Audio, &["mp3"]),
  ("audio/wav", Content::Audio, &["wav"]),
  ("image/apng", Content::Image, &["apng"]),
  ("image/gif", Content::Image, &["gif"]),
  ("image/jpeg", Content::Image, &["jpg", "jpeg"]),
  ("image/png", Content::Image, &["png"]),
  ("image/svg+xml", Content::Iframe, &["svg"]),
  ("image/webp", Content::Image, &["webp"]),
  ("text/html;charset=utf-8", Content::Iframe, &["html"]),
  ("text/plain;charset=utf-8", Content::Text, &["txt"]),
];

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn for_extension() {
    assert_eq!(
      Content::content_type_for_extension("jpg").unwrap(),
      "image/jpeg"
    );
    assert_eq!(
      Content::content_type_for_extension("jpeg").unwrap(),
      "image/jpeg"
    );
    assert_eq!(
      Content::content_type_for_extension("foo").unwrap_err().to_string(),
      "unsupported file extension `.foo`, supported extensions: apng gif html jpg mp3 png svg txt webp"
    );
  }
}
