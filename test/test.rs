use base64::Engine;
use std::{
  env, fs,
  fs::File,
  io::Write,
  path::PathBuf,
  process::{Command, Stdio},
  time::{SystemTime, UNIX_EPOCH},
};

fn reimg_command() -> Command {
  Command::new(env!("CARGO_BIN_EXE_reimg"))
}

fn unique_temp_path(name: &str) -> PathBuf {
  let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
  env::temp_dir().join(format!("reimg-{}-{nanos}-{name}", std::process::id()))
}

fn inspect_image_info(image: &[u8]) -> Vec<u8> {
  let mut child = reimg_command()
    .arg("-i")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();

  child.stdin.as_mut().unwrap().write_all(image).unwrap();
  child.wait_with_output().unwrap().stdout
}

fn assert_no_stderr(output: &std::process::Output) {
  let stderr = &output.stderr;
  if !stderr.is_empty() {
    println!("stderr: {}", String::from_utf8_lossy(stderr));
    assert!(false);
  }
}

#[test]
fn test_jpeg_to_jpeg() {
  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = reimg_command()
    .arg("-w")
    .arg("100")
    .arg("-h")
    .arg("100")
    .arg("--cover")
    .stdin(input)
    .output()
    .unwrap();

  assert_no_stderr(&output);

  let mut child = reimg_command()
    .arg("-i")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();

  // Write the previous output's stdout to the child's stdin
  child.stdin.as_mut().unwrap().write_all(&output.stdout).unwrap();

  let info = child.wait_with_output().unwrap();
  assert_eq!(info.stdout, b"Jpeg 100x100\n");
}

#[test]
fn test_jpeg_to_png() {
  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = reimg_command()
    .arg("-w")
    .arg("100")
    .arg("-h")
    .arg("100")
    .arg("--cover")
    .arg("--format")
    .arg("png")
    .stdin(input)
    .output()
    .unwrap();
  assert_no_stderr(&output);

  assert_eq!(inspect_image_info(&output.stdout), b"Png 100x100\n");
}

#[test]
fn test_file_input_output() {
  let output_path = unique_temp_path("output.webp");

  let output = reimg_command()
    .arg("-w")
    .arg("100")
    .arg("-h")
    .arg("100")
    .arg("--cover")
    .arg("test/testdata/photo.jpeg")
    .arg(&output_path)
    .output()
    .unwrap();

  assert_no_stderr(&output);

  let info = reimg_command().arg("-i").arg(&output_path).output().unwrap();
  assert_eq!(info.stdout, b"WebP 100x100\n");

  fs::remove_file(output_path).unwrap();
}

#[test]
fn test_data_url() {
  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = reimg_command()
    .arg("-w")
    .arg("100")
    .arg("-h")
    .arg("100")
    .arg("--cover")
    .arg("--format")
    .arg("webp")
    .arg("--data-url")
    .stdin(input)
    .output()
    .unwrap();
  assert_no_stderr(&output);

  assert!(!output.stdout.is_empty());
  let data_url = String::from_utf8_lossy(&output.stdout);
  assert!(!data_url.is_empty());
  assert!(data_url.starts_with("data:image/webp;base64,"));

  let encoded = data_url.strip_prefix("data:image/webp;base64,").unwrap().trim();
  let decoded = base64::engine::general_purpose::STANDARD.decode(encoded).unwrap();

  assert_eq!(inspect_image_info(&decoded), b"WebP 100x100\n");
}

#[test]
fn test_size_option() {
  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = reimg_command()
    .arg("-s")
    .arg("120x80")
    .arg("--format")
    .arg("png")
    .stdin(input)
    .output()
    .unwrap();

  assert_no_stderr(&output);
  assert_eq!(inspect_image_info(&output.stdout), b"Png 120x80\n");
}

#[test]
fn test_scale_option() {
  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = reimg_command()
    .arg("--scale")
    .arg("0.5")
    .arg("--format")
    .arg("png")
    .stdin(input)
    .output()
    .unwrap();

  assert_no_stderr(&output);
  assert_eq!(inspect_image_info(&output.stdout), b"Png 660x807\n");
}

#[test]
fn test_2x_scale_shortcut() {
  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = reimg_command()
    .arg("--2x")
    .arg("--format")
    .arg("png")
    .stdin(input)
    .output()
    .unwrap();

  assert_no_stderr(&output);
  assert_eq!(inspect_image_info(&output.stdout), b"Png 2640x3226\n");
}

#[test]
fn test_width_only_with_contain() {
  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = reimg_command()
    .arg("-w")
    .arg("100")
    .arg("--contain")
    .arg("--format")
    .arg("png")
    .stdin(input)
    .output()
    .unwrap();

  assert_no_stderr(&output);
  assert_eq!(inspect_image_info(&output.stdout), b"Png 100x122\n");
}

#[test]
fn test_width_only_with_scale_down() {
  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = reimg_command()
    .arg("-w")
    .arg("100")
    .arg("--scale-down")
    .arg("--format")
    .arg("png")
    .stdin(input)
    .output()
    .unwrap();

  assert_no_stderr(&output);
  assert_eq!(inspect_image_info(&output.stdout), b"Png 100x122\n");
}

#[test]
fn test_width_only_with_cover_requires_height() {
  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = reimg_command()
    .arg("-w")
    .arg("100")
    .arg("--cover")
    .stdin(input)
    .output()
    .unwrap();

  assert!(!output.status.success());
  assert_eq!(output.stderr, b"error: --cover requires both --width and --height\n");
}

#[test]
fn test_jpeg_to_webp() {
  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = reimg_command()
    .arg("-w")
    .arg("100")
    .arg("-h")
    .arg("100")
    .arg("--cover")
    .arg("--format")
    .arg("webp")
    .stdin(input)
    .output()
    .unwrap();

  assert_no_stderr(&output);

  let mut child = reimg_command()
    .arg("-i")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();

  // Write the previous output's stdout to the child's stdin
  child.stdin.as_mut().unwrap().write_all(&output.stdout).unwrap();

  let info = child.wait_with_output().unwrap();
  assert_eq!(info.stdout, b"WebP 100x100\n");
}

#[test]
fn test_jpeg_to_avif() {
  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = reimg_command()
    .arg("-w")
    .arg("100")
    .arg("-h")
    .arg("100")
    .arg("--cover")
    .arg("--format")
    .arg("avif")
    .stdin(input)
    .output()
    .unwrap();

  assert_no_stderr(&output);
}
