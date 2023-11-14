use {
  super::*,
  brotlic::CompressionMode,
  mp4::{MediaType, Mp4Reader, TrackType},
  std::{fs::File, io::BufReader},
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Media {
  Audio,
  Code,
  Iframe,
  Image,
  Markdown,
  Model,
  Pdf,
  Text,
  Unknown,
  Video,
}

impl Media {
  #[rustfmt::skip]
  const TABLE: &'static [(&'static str, CompressionMode, Media, &'static [&'static str])] = &[
    ("application/cbor", CompressionMode::Generic, Media::Unknown, &["cbor"]),
    ("application/json", CompressionMode::Text, Media::Code, &["json"]),
    ("application/pdf", CompressionMode::Generic, Media::Pdf, &["pdf"]),
    ("application/pgp-signature", CompressionMode::Text, Media::Text, &["asc"]),
    ("application/protobuf", CompressionMode::Generic, Media::Unknown, &["binpb"]),
    ("application/yaml", CompressionMode::Text, Media::Code, &["yaml", "yml"]),
    ("audio/flac", CompressionMode::Generic, Media::Audio, &["flac"]),
    ("audio/mpeg", CompressionMode::Generic, Media::Audio, &["mp3"]),
    ("audio/wav", CompressionMode::Generic, Media::Audio, &["wav"]),
    ("font/otf", CompressionMode::Generic, Media::Unknown, &["otf"]),
    ("font/ttf", CompressionMode::Generic, Media::Unknown, &["ttf"]),
    ("font/woff", CompressionMode::Generic, Media::Unknown, &["woff"]),
    ("font/woff2", CompressionMode::Font, Media::Unknown, &["woff2"]),
    ("image/apng", CompressionMode::Generic, Media::Image, &["apng"]),
    ("image/avif", CompressionMode::Generic, Media::Image, &[]),
    ("image/gif", CompressionMode::Generic, Media::Image, &["gif"]),
    ("image/jpeg", CompressionMode::Generic, Media::Image, &["jpg", "jpeg"]),
    ("image/png", CompressionMode::Generic, Media::Image, &["png"]),
    ("image/svg+xml", CompressionMode::Text, Media::Iframe, &["svg"]),
    ("image/webp", CompressionMode::Generic, Media::Image, &["webp"]),
    ("model/gltf+json", CompressionMode::Text, Media::Model, &["gltf"]),
    ("model/gltf-binary", CompressionMode::Generic, Media::Model, &["glb"]),
    ("model/stl", CompressionMode::Generic, Media::Unknown, &["stl"]),
    ("text/css", CompressionMode::Text, Media::Code, &["css"]),
    ("text/html", CompressionMode::Text, Media::Iframe, &[]),
    ("text/html;charset=utf-8", CompressionMode::Text, Media::Iframe, &["html"]),
    ("text/javascript", CompressionMode::Text, Media::Code, &["js"]),
    ("text/markdown", CompressionMode::Text, Media::Markdown, &[]),
    ("text/markdown;charset=utf-8", CompressionMode::Text, Media::Markdown, &["md"]),
    ("text/plain", CompressionMode::Text, Media::Text, &[]),
    ("text/plain;charset=utf-8", CompressionMode::Text, Media::Text, &["txt"]),
    ("text/x-python", CompressionMode::Text, Media::Code, &["py"]),
    ("video/mp4", CompressionMode::Generic, Media::Video, &["mp4"]),
    ("video/webm", CompressionMode::Generic, Media::Video, &["webm"]),
  ];

  pub(crate) fn content_type_for_path(path: &Path) -> Result<(&'static str, CompressionMode), Error> {
    let extension = path
      .extension()
      .ok_or_else(|| anyhow!("file must have extension"))?
      .to_str()
      .ok_or_else(|| anyhow!("unrecognized extension"))?;

    let extension = extension.to_lowercase();

    if extension == "mp4" {
      Media::check_mp4_codec(path)?;
    }

    for (content_type, mode, _, extensions) in Self::TABLE {
      if extensions.contains(&extension.as_str()) {
        return Ok((*content_type, *mode));
      }
    }

    let mut extensions = Self::TABLE
      .iter()
      .flat_map(|(_, _, _, extensions)| extensions.first().cloned())
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
    for entry in Self::TABLE {
      if entry.0 == s {
        return Ok(entry.2);
      }
    }

    Err(anyhow!("unknown content type: {s}"))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn for_extension() {
    assert_eq!(
      Media::content_type_for_path(Path::new("pepe.jpg")).unwrap(),
      ("image/jpeg", CompressionMode::Generic)
    );
    assert_eq!(
      Media::content_type_for_path(Path::new("pepe.jpeg")).unwrap(),
      ("image/jpeg", CompressionMode::Generic)
    );
    assert_eq!(
      Media::content_type_for_path(Path::new("pepe.JPG")).unwrap(),
      ("image/jpeg", CompressionMode::Generic)
    );
    assert_eq!(
      Media::content_type_for_path(Path::new("pepe.txt")).unwrap(),
      ("text/plain;charset=utf-8", CompressionMode::Text)
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
