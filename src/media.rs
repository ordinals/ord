use {
  super::*,
  mp4::{MediaType, Mp4Reader, TrackType},
  std::{fs::File, io::BufReader},
};

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
  pub(crate) fn content_type_for_path(path: &Path) -> Result<&'static str, Error> {
    let extension = path
      .extension()
      .ok_or_else(|| anyhow!("file must have extension"))?
      .to_str()
      .ok_or_else(|| anyhow!("unrecognized extension"))?;

    let extension = extension.to_lowercase();

    if extension == "mp4" {
      Media::check_mp4_codec(path)?;
    }

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

  pub(crate) fn check_mp4_codec(path: &Path) -> Result<(), Error> {
    let f = File::open(path)?;
    let size = f.metadata()?.len();
    let reader = BufReader::new(f);

    let mp4 = Mp4Reader::read_header(reader, size)?;

    for track in mp4.tracks().values() {
      if let TrackType::Video = track.track_type()? {
        let media_type = track.media_type()?;
        if media_type != MediaType::H264 {
          return Err(anyhow!(
            "Unsupported video codec, only H.264 is supported in MP4: {media_type}"
          ));
        }
      }
    }

    Ok(())
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
  ("application/pdf", Media::Pdf, &["pdf"]),
  ("application/pgp-signature", Media::Text, &["asc"]),
  ("application/yaml", Media::Text, &["yaml", "yml"]),
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
  ("video/mp4", Media::Video, &["mp4"]),
  ("video/webm", Media::Video, &["webm"]),
];

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn for_extension() {
    assert_eq!(
      Media::content_type_for_path(Path::new("pepe.jpg")).unwrap(),
      "image/jpeg"
    );
    assert_eq!(
      Media::content_type_for_path(Path::new("pepe.jpeg")).unwrap(),
      "image/jpeg"
    );
    assert_eq!(
      Media::content_type_for_path(Path::new("pepe.JPG")).unwrap(),
      "image/jpeg"
    );

    assert_regex_match!(
      Media::content_type_for_path(Path::new("pepe.foo")).unwrap_err(),
      r"unsupported file extension `\.foo`, supported extensions: apng .*"
    );
  }

  #[test]
  fn h264_in_mp4_is_allowed() {
    assert!(Media::check_mp4_codec(Path::new("examples/h264.mp4")).is_ok(),);
  }

  #[test]
  fn av1_in_mp4_is_rejected() {
    assert!(Media::check_mp4_codec(Path::new("examples/av1.mp4")).is_err(),);
  }
}
