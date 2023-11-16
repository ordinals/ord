use {
  super::*,
  brotli::enc::backward_references::BrotliEncoderMode,
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
  const TABLE: &'static [(&'static str, BrotliEncoderMode, Media, &'static [&'static str])] = &[
    ("application/cbor", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Unknown, &["cbor"]),
    ("application/json", BrotliEncoderMode::BROTLI_MODE_TEXT, Media::Code, &["json"]),
    ("application/pdf", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Pdf, &["pdf"]),
    ("application/pgp-signature", BrotliEncoderMode::BROTLI_MODE_TEXT, Media::Text, &["asc"]),
    ("application/protobuf", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Unknown, &["binpb"]),
    ("application/yaml", BrotliEncoderMode::BROTLI_MODE_TEXT, Media::Code, &["yaml", "yml"]),
    ("audio/flac", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Audio, &["flac"]),
    ("audio/mpeg", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Audio, &["mp3"]),
    ("audio/wav", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Audio, &["wav"]),
    ("font/otf", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Unknown, &["otf"]),
    ("font/ttf", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Unknown, &["ttf"]),
    ("font/woff", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Unknown, &["woff"]),
    ("font/woff2", BrotliEncoderMode::BROTLI_MODE_FONT, Media::Unknown, &["woff2"]),
    ("image/apng", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Image, &["apng"]),
    ("image/avif", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Image, &[]),
    ("image/gif", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Image, &["gif"]),
    ("image/jpeg", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Image, &["jpg", "jpeg"]),
    ("image/png", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Image, &["png"]),
    ("image/svg+xml", BrotliEncoderMode::BROTLI_MODE_TEXT, Media::Iframe, &["svg"]),
    ("image/webp", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Image, &["webp"]),
    ("model/gltf+json", BrotliEncoderMode::BROTLI_MODE_TEXT, Media::Model, &["gltf"]),
    ("model/gltf-binary", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Model, &["glb"]),
    ("model/stl", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Unknown, &["stl"]),
    ("text/css", BrotliEncoderMode::BROTLI_MODE_TEXT, Media::Code, &["css"]),
    ("text/html", BrotliEncoderMode::BROTLI_MODE_TEXT, Media::Iframe, &[]),
    ("text/html;charset=utf-8", BrotliEncoderMode::BROTLI_MODE_TEXT, Media::Iframe, &["html"]),
    ("text/javascript", BrotliEncoderMode::BROTLI_MODE_TEXT, Media::Code, &["js"]),
    ("text/markdown", BrotliEncoderMode::BROTLI_MODE_TEXT, Media::Markdown, &[]),
    ("text/markdown;charset=utf-8", BrotliEncoderMode::BROTLI_MODE_TEXT, Media::Markdown, &["md"]),
    ("text/plain", BrotliEncoderMode::BROTLI_MODE_TEXT, Media::Text, &[]),
    ("text/plain;charset=utf-8", BrotliEncoderMode::BROTLI_MODE_TEXT, Media::Text, &["txt"]),
    ("text/x-python", BrotliEncoderMode::BROTLI_MODE_TEXT, Media::Code, &["py"]),
    ("video/mp4", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Video, &["mp4"]),
    ("video/webm", BrotliEncoderMode::BROTLI_MODE_GENERIC, Media::Video, &["webm"]),
  ];

  pub(crate) fn content_type_for_path(
    path: &Path,
  ) -> Result<(&'static str, BrotliEncoderMode), Error> {
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
      ("image/jpeg", BrotliEncoderMode::BROTLI_MODE_GENERIC)
    );
    assert_eq!(
      Media::content_type_for_path(Path::new("pepe.jpeg")).unwrap(),
      ("image/jpeg", BrotliEncoderMode::BROTLI_MODE_GENERIC)
    );
    assert_eq!(
      Media::content_type_for_path(Path::new("pepe.JPG")).unwrap(),
      ("image/jpeg", BrotliEncoderMode::BROTLI_MODE_GENERIC)
    );
    assert_eq!(
      Media::content_type_for_path(Path::new("pepe.txt")).unwrap(),
      (
        "text/plain;charset=utf-8",
        BrotliEncoderMode::BROTLI_MODE_TEXT
      )
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
