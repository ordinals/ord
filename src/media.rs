use super::*;

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Media {
  Audio,
  Iframe,
  Image,
  Pdf,
  Text,
  Unknown,
  Video,
}

impl Media {
  pub(crate) fn content_type_for_extension(extension: &str) -> Result<&'static str, Error> {
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

impl FromStr for Media {
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

const TABLE: &[(&str, Media, &[&str])] = &[
  ("application/json", Media::Text, &["json"]),
  ("application/yaml", Media::Text, &["yaml", "yml"]),
  ("application/pdf", Media::Pdf, &["pdf"]),
  ("application/pgp-signature", Media::Text, &["asc"]),
  ("audio/flac", Media::Audio, &["flac"]),
  ("audio/mpeg", Media::Audio, &["mp3"]),
  ("audio/wav", Media::Audio, &["wav"]),
  ("image/apng", Media::Image, &["apng"]),
  ("image/gif", Media::Image, &["gif"]),
  ("image/jpeg", Media::Image, &["jpg", "jpeg"]),
  ("image/png", Media::Image, &["png"]),
  ("image/svg+xml", Media::Iframe, &["svg"]),
  ("image/webp", Media::Image, &["webp"]),
  ("text/html;charset=utf-8", Media::Iframe, &["html"]),
  ("text/plain;charset=utf-8", Media::Text, &["txt"]),
  ("video/webm", Media::Video, &["webm"]),
];

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn for_extension() {
    assert_eq!(
      Media::content_type_for_extension("jpg").unwrap(),
      "image/jpeg"
    );
    assert_eq!(
      Media::content_type_for_extension("jpeg").unwrap(),
      "image/jpeg"
    );
    assert_eq!(
      Media::content_type_for_extension("JPG").unwrap(),
      "image/jpeg"
    );

    assert_regex_match!(
      Media::content_type_for_extension("foo").unwrap_err(),
      r"unsupported file extension `\.foo`, supported extensions: apng .*"
    );
  }
}
