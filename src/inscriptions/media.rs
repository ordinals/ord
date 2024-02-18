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
  const TABLE: &'static [(&'static str, BrotliEncoderMode, Media, &'static [&'static str], &'static str)] = &[
    ("application/cbor",            GENERIC, Unknown,          &["cbor"],        "cbor"),
    ("application/json",            TEXT,    Code(Json),       &["json"],        "json"),
    ("application/octet-stream",    GENERIC, Unknown,          &["bin"],         "bin"),
    ("application/pdf",             GENERIC, Pdf,              &["pdf"],         "pdf"),
    ("application/pgp-signature",   TEXT,    Text,             &["asc"],         "asc"),
    ("application/protobuf",        GENERIC, Unknown,          &["binpb"],       "binp"),
    ("application/x-javascript",    TEXT,    Code(JavaScript), &[],              "js"),
    ("application/yaml",            TEXT,    Code(Yaml),       &["yaml", "yml"], "yaml"),
    ("audio/flac",                  GENERIC, Audio,            &["flac"],        "flac"),
    ("audio/mpeg",                  GENERIC, Audio,            &["mp3"],         "mp3"),
    ("audio/wav",                   GENERIC, Audio,            &["wav"],         "wav"),
    ("font/otf",                    GENERIC, Font,             &["otf"],         "otf"),
    ("font/ttf",                    GENERIC, Font,             &["ttf"],         "ttf"),
    ("font/woff",                   GENERIC, Font,             &["woff"],        "woff"),
    ("font/woff2",                  FONT,    Font,             &["woff2"],       "woff2"),
    ("image/apng",                  GENERIC, Image,            &["apng"],        "apng"),
    ("image/avif",                  GENERIC, Image,            &["avif"],        "avif"),
    ("image/gif",                   GENERIC, Image,            &["gif"],         "gif"),
    ("image/jpeg",                  GENERIC, Image,            &["jpg", "jpeg"], "jpg"),
    ("image/png",                   GENERIC, Image,            &["png"],         "png"),
    ("image/svg+xml",               TEXT,    Iframe,           &["svg"],         "svg"),
    ("image/webp",                  GENERIC, Image,            &["webp"],        "webp"),
    ("model/gltf+json",             TEXT,    Model,            &["gltf"],        "gltf"),
    ("model/gltf-binary",           GENERIC, Model,            &["glb"],         "glb"),
    ("model/stl",                   GENERIC, Unknown,          &["stl"],         "stl"),
    ("text/css",                    TEXT,    Code(Css),        &["css"],         "css"),
    ("text/html",                   TEXT,    Iframe,           &[],              "html"),
    ("text/html;charset=utf-8",     TEXT,    Iframe,           &["html"],        "html"),
    ("text/javascript",             TEXT,    Code(JavaScript), &["js"],          "js"),
    ("text/markdown",               TEXT,    Markdown,         &[],              "md"),
    ("text/markdown;charset=utf-8", TEXT,    Markdown,         &["md"],          "md"),
    ("text/plain",                  TEXT,    Text,             &[],              "txt"),
    ("text/plain;charset=utf-8",    TEXT,    Text,             &["txt"],         "txt"),
    ("text/x-python",               TEXT,    Code(Python),     &["py"],          "py"),
    ("video/mp4",                   GENERIC, Video,            &["mp4"],         "mp4"),
    ("video/webm",                  GENERIC, Video,            &["webm"],        "webm"),
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

    for (content_type, mode, _, extensions, _) in Self::TABLE {
      if extensions.contains(&extension.as_str()) {
        return Ok((*content_type, *mode));
      }
    }

    let mut extensions = Self::TABLE
      .iter()
      .flat_map(|(_, _, _, extensions, _)| extensions.first().cloned())
      .collect::<Vec<&str>>();

    extensions.sort();

    Err(anyhow!(
      "unsupported file extension `.{extension}`, supported extensions: {}",
      extensions.join(" "),
    ))
  }

  pub(crate) fn extension_for_content_type(content_type: &str) -> Option<&'static str> {
    lazy_static! {
      static ref CONTENT_TYPE_TO_EXTENSION: BTreeMap<&'static str, &'static str> = {
        Media::TABLE
          .iter()
          .map(
            |(content_type, _compression_mode, _media, _extensions, extension)| {
              (*content_type, *extension)
            },
          )
          .collect()
      };
    }

    CONTENT_TYPE_TO_EXTENSION.get(content_type).cloned()
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
