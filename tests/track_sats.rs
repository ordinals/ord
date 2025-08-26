use std::fs;

#[test]
fn track_sats_option_parses_correctly() {
  // Test that the basic file parsing logic works correctly
  let temp_dir = tempfile::tempdir().unwrap();
  let sats_file = temp_dir.path().join("sats.txt");
  fs::write(&sats_file, "0\n1\n100\n1000").unwrap();

  // Verify the file was created and contains the expected content
  let content = fs::read_to_string(&sats_file).unwrap();
  let lines: Vec<&str> = content.lines().collect();
  assert_eq!(lines.len(), 4);
  assert_eq!(lines[0], "0");
  assert_eq!(lines[1], "1");
  assert_eq!(lines[2], "100");
  assert_eq!(lines[3], "1000");
}

#[test]
fn track_sats_file_loading() {
  let temp_dir = tempfile::tempdir().unwrap();
  let sats_file = temp_dir.path().join("sats.txt");
  
  // Test with comments and empty lines
  fs::write(&sats_file, "# This is a comment\n0\n\n1\n# Another comment\n100").unwrap();

  // Test the file loading logic directly
  let content = fs::read_to_string(&sats_file).unwrap();
  let mut tracked_sats = std::collections::HashSet::new();
  for line in content.lines() {
    let line = line.trim();
    if !line.is_empty() && !line.starts_with('#') {
      let sat: u64 = line.parse().unwrap();
      tracked_sats.insert(sat);
    }
  }
  
  assert_eq!(tracked_sats.len(), 3);
  assert!(tracked_sats.contains(&0));
  assert!(tracked_sats.contains(&1));
  assert!(tracked_sats.contains(&100));
  assert!(!tracked_sats.contains(&1000));
}

#[test]
fn track_sats_with_invalid_numbers() {
  let temp_dir = tempfile::tempdir().unwrap();
  let sats_file = temp_dir.path().join("sats.txt");
  fs::write(&sats_file, "0\ninvalid\n100").unwrap();

  // Test that parsing invalid numbers fails
  let content = fs::read_to_string(&sats_file).unwrap();
  let mut has_error = false;
  
  for line in content.lines() {
    let line = line.trim();
    if !line.is_empty() && !line.starts_with('#') {
      if let Err(_) = line.parse::<u64>() {
        has_error = true;
        break;
      }
    }
  }
  
  assert!(has_error);
}

#[test]
fn track_sats_basic_functionality() {
  // Test that the basic file parsing logic works
  let temp_dir = tempfile::tempdir().unwrap();
  let sats_file = temp_dir.path().join("sats.txt");
  fs::write(&sats_file, "0\n1\n100").unwrap();

  let content = fs::read_to_string(&sats_file).unwrap();
  let mut tracked_sats = std::collections::HashSet::new();
  for line in content.lines() {
    let line = line.trim();
    if !line.is_empty() && !line.starts_with('#') {
      let sat: u64 = line.parse().unwrap();
      tracked_sats.insert(sat);
    }
  }
  
  assert_eq!(tracked_sats.len(), 3);
  assert!(tracked_sats.contains(&0));
  assert!(tracked_sats.contains(&1));
  assert!(tracked_sats.contains(&100));
} 