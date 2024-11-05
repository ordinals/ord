use {
  self::{ImageRendering::*, Language::*, Media::*},
  super::*,
  brotli::enc::backward_references::BrotliEncoderMode::{
    self, BROTLI_MODE_FONT as FONT, BROTLI_MODE_GENERIC as GENERIC, BROTLI_MODE_TEXT as TEXT,
  },
  mp4::{MediaType, Mp4Reader, TrackType},
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Media {
  Audio,
  Code(Language),
  Font,
  Iframe,
  Image(ImageRendering),
  Markdown,
  Model,
  Pdf,
  Text,
  Unknown,
  Video,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Language {
  Css,
  JavaScript,
  Json,
  Python,
  Yaml,
}

impl Display for Language {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Css => "css",
        Self::JavaScript => "javascript",
        Self::Json => "json",
        Self::Python => "python",
        Self::Yaml => "yaml",
      }
    )
  }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ImageRendering {
  Auto,
  Pixelated,
}

impl Display for ImageRendering {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Auto => "auto",
        Self::Pixelated => "pixelated",
      }
    )
  }
}

impl Media {
  #[rustfmt::skip]
  const TABLE: &'static [(&'static str, BrotliEncoderMode, Media, &'static [&'static str])] = &[
    ("application/cbor",            GENERIC, Unknown,          &["cbor"]),
    ("application/json",            TEXT,    Code(Json),       &["json"]),
    ("application/octet-stream",    GENERIC, Unknown,          &["bin"]),
    ("application/pdf",             GENERIC, Pdf,              &["pdf"]),
    ("application/pgp-signature",   TEXT,    Text,             &["asc"]),
    ("application/protobuf",        GENERIC, Unknown,          &["binpb"]),
    ("application/x-javascript",    TEXT,    Code(JavaScript), &[]),
    ("application/yaml",            TEXT,    Code(Yaml),       &["yaml", "yml"]),
    ("audio/flac",                  GENERIC, Audio,            &["flac"]),
    ("audio/mpeg",                  GENERIC, Audio,            &["mp3"]),
    ("audio/wav",                   GENERIC, Audio,            &["wav"]),
    ("font/otf",                    GENERIC, Font,             &["otf"]),
    ("font/ttf",                    GENERIC, Font,             &["ttf"]),
    ("font/woff",                   GENERIC, Font,             &["woff"]),
    ("font/woff2",                  FONT,    Font,             &["woff2"]),
    ("image/apng",                  GENERIC, Image(Pixelated), &["apng"]),
    ("image/avif",                  GENERIC, Image(Auto),      &["avif"]),
    ("image/gif",                   GENERIC, Image(Pixelated), &["gif"]),
    ("image/jpeg",                  GENERIC, Image(Pixelated), &["jpg", "jpeg"]),
    ("image/jxl",                   GENERIC, Image(Auto),      &[]),
    ("image/png",                   GENERIC, Image(Pixelated), &["png"]),
    ("image/svg+xml",               TEXT,    Iframe,           &["svg"]),
    ("image/webp",                  GENERIC, Image(Pixelated), &["webp"]),
    ("model/gltf+json",             TEXT,    Model,            &["gltf"]),
    ("model/gltf-binary",           GENERIC, Model,            &["glb"]),
    ("model/stl",                   GENERIC, Unknown,          &["stl"]),
    ("text/css",                    TEXT,    Code(Css),        &["css"]),
    ("text/html",                   TEXT,    Iframe,           &[]),
    ("text/html;charset=utf-8",     TEXT,    Iframe,           &["html"]),
    ("text/javascript",             TEXT,    Code(JavaScript), &["js", "mjs"]),
    ("text/markdown",               TEXT,    Markdown,         &[]),
    ("text/markdown;charset=utf-8", TEXT,    Markdown,         &["md"]),
    ("text/plain",                  TEXT,    Text,             &[]),
    ("text/plain;charset=utf-8",    TEXT,    Text,             &["txt"]),
    ("text/x-python",               TEXT,    Code(Python),     &["py"]),
    ("video/mp4",                   GENERIC, Video,            &["mp4"]),
    ("video/webm",                  GENERIC, Video,            &["webm"]),
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

  #[test]
  fn no_duplicate_extensions() {
    let mut set = HashSet::new();
    for (_, _, _, extensions) in Media::TABLE {
      for extension in *extensions {
        assert!(set.insert(extension), "duplicate extension `{extension}`");
      }
    }
  }
}
