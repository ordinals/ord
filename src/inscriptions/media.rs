use {
  self::{Language::*, Media::*},
  super::*,
  brotli::enc::backward_references::BrotliEncoderMode::{
    self, BROTLI_MODE_FONT as FONT, BROTLI_MODE_GENERIC as GENERIC, BROTLI_MODE_TEXT as TEXT,
  },
  mp4::{MediaType, Mp4Reader, TrackType},
  std::{fs::File, io::BufReader},
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Media {
  Audio,
  Code(Language),
  Font,
  Iframe,
  Image,
  Markdown,
  Model,
  Pdf,
  Text,
  Unknown,
  Video,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Language {
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

impl Media {
  #[rustfmt::skip]
  const TABLE: &'static [(&'static str, bool, BrotliEncoderMode, Media, &'static [&'static str])] = &[
    ("application/cbor",            true,  GENERIC, Unknown,          &["cbor"]),
    ("application/json",            true,  TEXT,    Code(Json),       &["json"]),
    ("application/octet-stream",    true,  GENERIC, Unknown,          &["bin"]),
    ("application/pdf",             true,  GENERIC, Pdf,              &["pdf"]),
    ("application/pgp-signature",   true,  TEXT,    Text,             &["asc"]),
    ("application/protobuf",        true,  GENERIC, Unknown,          &["binpb"]),
    ("application/x-javascript",    false, TEXT,    Code(JavaScript), &["js"]),
    ("application/yaml",            true,  TEXT,    Code(Yaml),       &["yaml", "yml"]),
    ("audio/flac",                  true,  GENERIC, Audio,            &["flac"]),
    ("audio/mpeg",                  true,  GENERIC, Audio,            &["mp3"]),
    ("audio/wav",                   true,  GENERIC, Audio,            &["wav"]),
    ("font/otf",                    true,  GENERIC, Font,             &["otf"]),
    ("font/ttf",                    true,  GENERIC, Font,             &["ttf"]),
    ("font/woff",                   true,  GENERIC, Font,             &["woff"]),
    ("font/woff2",                  true,  FONT,    Font,             &["woff2"]),
    ("image/apng",                  true,  GENERIC, Image,            &["apng"]),
    ("image/avif",                  true,  GENERIC, Image,            &["avif"]),
    ("image/gif",                   true,  GENERIC, Image,            &["gif"]),
    ("image/jpeg",                  true,  GENERIC, Image,            &["jpg", "jpeg"]),
    ("image/png",                   true,  GENERIC, Image,            &["png"]),
    ("image/svg+xml",               true,  TEXT,    Iframe,           &["svg"]),
    ("image/webp",                  true,  GENERIC, Image,            &["webp"]),
    ("model/gltf+json",             true,  TEXT,    Model,            &["gltf"]),
    ("model/gltf-binary",           true,  GENERIC, Model,            &["glb"]),
    ("model/stl",                   true,  GENERIC, Unknown,          &["stl"]),
    ("text/css",                    true,  TEXT,    Code(Css),        &["css"]),
    ("text/html",                   false, TEXT,    Iframe,           &["html"]),
    ("text/html;charset=utf-8",     true,  TEXT,    Iframe,           &["html"]),
    ("text/javascript",             true,  TEXT,    Code(JavaScript), &["js"]),
    ("text/markdown",               false, TEXT,    Markdown,         &["md"]),
    ("text/markdown;charset=utf-8", true,  TEXT,    Markdown,         &["md"]),
    ("text/plain",                  false, TEXT,    Text,             &["txt"]),
    ("text/plain;charset=utf-8",    true,  TEXT,    Text,             &["txt"]),
    ("text/x-python",               true,  TEXT,    Code(Python),     &["py"]),
    ("video/mp4",                   true,  GENERIC, Video,            &["mp4"]),
    ("video/webm",                  true,  GENERIC, Video,            &["webm"]),
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

    for (content_type, preferred, mode, _, extensions) in Self::TABLE {
      if *preferred && extensions.contains(&extension.as_str()) {
        return Ok((*content_type, *mode));
      }
    }

    let mut extensions = Self::TABLE
      .iter()
      .filter(|(_, preferred, _, _, _)| *preferred)
      .map(|(_, _, _, _, extensions)| extensions[0])
      .collect::<Vec<&str>>();

    extensions.sort();

    Err(anyhow!(
      "unsupported file extension `.{extension}`, supported extensions: {}",
      extensions.join(" "),
    ))
  }

  pub(crate) fn extension_for_content_type(content_type: &str) -> Option<&'static str> {
    lazy_static! {
      static ref EXTENSION_FOR_CONTENT_TYPE: BTreeMap<&'static str, &'static str> = {
        Media::TABLE
          .iter()
          .map(
            |(content_type, _preferred, _compression_mode, _media, extensions)| {
              (*content_type, extensions[0])
            },
          )
          .collect()
      };
    }

    EXTENSION_FOR_CONTENT_TYPE.get(content_type).cloned()
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
        return Ok(entry.3);
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
    for (_, preferred, _, _, extensions) in Media::TABLE {
      if *preferred {
        for extension in *extensions {
          assert!(set.insert(extension), "duplicate extension `{extension}`");
        }
      }
    }
  }
}
