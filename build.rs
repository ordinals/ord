use {
  pulldown_cmark::{html, Parser},
  std::{env, ffi::OsStr, fs, path::Path},
};

fn main() {
  println!("cargo:rerun-if-changed=markdown");

  for result in fs::read_dir("markdown").unwrap() {
    let entry = result.unwrap();
    let path = entry.path();

    if path.extension() != Some(OsStr::new("md")) {
      continue;
    }

    let input = fs::read_to_string(&path).unwrap();

    let parser = Parser::new(&input);

    let mut output = String::new();
    html::push_html(&mut output, parser);

    fs::write(
      Path::new(&env::var("OUT_DIR").unwrap())
        .join(path.file_stem().unwrap())
        .with_extension("html"),
      output,
    )
    .unwrap();
  }
}
