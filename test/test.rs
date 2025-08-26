use std::{
  fs::File,
  io::Write,
  process::{Command, Stdio},
};

#[test]
fn test_jpeg_to_jpeg() {
  Command::new("cargo").arg("build").output().unwrap();

  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = Command::new("target/debug/reimg")
    .arg("-w")
    .arg("100")
    .arg("-h")
    .arg("100")
    .arg("--cover")
    .stdin(input)
    .output()
    .unwrap();

  let stderr = output.stderr;
  if !stderr.is_empty() {
    println!("stderr: {}", String::from_utf8_lossy(&stderr));
    assert!(false);
  }

  let mut child = Command::new("target/debug/reimg")
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
  Command::new("cargo").arg("build").output().unwrap();

  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = Command::new("target/debug/reimg")
    .arg("-w")
    .arg("100")
    .arg("-h")
    .arg("100")
    .arg("--cover")
    .arg("-f")
    .arg("png")
    .stdin(input)
    .output()
    .unwrap();
  let stderr = output.stderr;
  if !stderr.is_empty() {
    println!("stderr: {}", String::from_utf8_lossy(&stderr));
    assert!(false);
  }

  let mut child = Command::new("target/debug/reimg")
    .arg("-i")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();

  // Write the previous output's stdout to the child's stdin
  child.stdin.as_mut().unwrap().write_all(&output.stdout).unwrap();

  let info = child.wait_with_output().unwrap();
  assert_eq!(info.stdout, b"Png 100x100\n");
}

#[test]
fn test_jpeg_to_webp() {
  Command::new("cargo").arg("build").output().unwrap();

  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = Command::new("target/debug/reimg")
    .arg("-w")
    .arg("100")
    .arg("-h")
    .arg("100")
    .arg("--cover")
    .arg("-f")
    .arg("webp")
    .stdin(input)
    .output()
    .unwrap();

  let stderr = output.stderr;
  if !stderr.is_empty() {
    println!("stderr: {}", String::from_utf8_lossy(&stderr));
    assert!(false);
  }

  let mut child = Command::new("target/debug/reimg")
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
  Command::new("cargo").arg("build").output().unwrap();

  let input = File::open("test/testdata/photo.jpeg").unwrap();
  let output = Command::new("target/debug/reimg")
    .arg("-w")
    .arg("100")
    .arg("-h")
    .arg("100")
    .arg("--cover")
    .arg("-f")
    .arg("avif")
    .stdin(input)
    .output()
    .unwrap();

  let stderr = output.stderr;
  if !stderr.is_empty() {
    println!("stderr: {}", String::from_utf8_lossy(&stderr));
    assert!(false);
  }
}
