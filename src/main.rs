use image::imageops::FilterType;
use image::{DynamicImage, ImageEncoder, ImageFormat, ImageReader};
use std::env;
use std::io::{self, Read, Write};

#[derive(Clone, PartialEq)]
enum FitMode {
  Cover,
  Contain,
  ScaleDown,
}

fn main() {
  let mut show_metadata: bool = false;
  let mut width: Option<u32> = None;
  let mut height: Option<u32> = None;
  let mut fit: Option<FitMode> = None;
  let mut quality: u8 = 85;
  let mut format: Option<String> = None;

  let mut args = env::args().skip(1);
  if args.len() == 0 {
    print_help_and_exit();
  }
  while let Some(arg) = args.next() {
    match arg.as_str() {
      "-i" | "--info" => {
        show_metadata = true;
      }
      "-w" | "--width" => {
        let Some(val) = args.next() else {
          print_error_and_exit("-w/--width requires a value");
        };
        match val.parse::<u32>() {
          Ok(v) if v > 0 => width = Some(v),
          _ => {
            print_error_and_exit("width must be a positive integer");
          }
        }
      }
      "-h" | "--height" => {
        let Some(val) = args.next() else {
          print_error_and_exit("-h/--height requires a value");
        };
        match val.parse::<u32>() {
          Ok(v) if v > 0 => height = Some(v),
          _ => {
            print_error_and_exit("height must be a positive integer");
          }
        }
      }
      "--fit" => {
        let Some(val) = args.next() else {
          print_error_and_exit("--fit requires a value");
        };
        match val.as_str() {
          "cover" => fit = Some(FitMode::Cover),
          "contain" => fit = Some(FitMode::Contain),
          "scale-down" => fit = Some(FitMode::ScaleDown),
          _ => {
            print_error_and_exit("invalid fit mode, possible values: cover, contain, scale-down");
          }
        };
      }
      "--cover" => {
        fit = Some(FitMode::Cover);
      }
      "--contain" => {
        fit = Some(FitMode::Contain);
      }
      "--scale-down" => {
        fit = Some(FitMode::ScaleDown);
      }
      "-q" | "--quality" => {
        let Some(val) = args.next() else {
          print_error_and_exit("-q/--quality requires a value");
        };
        match val.parse::<u8>() {
          Ok(q) if (1..=100).contains(&q) => {
            quality = q;
          }
          _ => {
            print_error_and_exit("quality must be an integer in 1..=100");
          }
        }
      }
      "-f" | "--format" => {
        let Some(val) = args.next() else {
          print_error_and_exit("-f/--format requires a value");
        };
        format = Some(val);
      }
      _ => {
        print_help_and_exit();
      }
    }
  }

  // Read all stdin into memory and decode by sniffing magic bytes
  let mut in_buf = Vec::new();
  io::stdin().read_to_end(&mut in_buf).expect("failed to read stdin");

  if show_metadata {
    let img_format = image::guess_format(&in_buf);
    if let Ok(f) = img_format {
      if let Some((w, h)) = ImageReader::with_format(io::Cursor::new(&in_buf), f)
        .into_dimensions()
        .ok()
      {
        println!("{:?} {}x{}", f, w, h);
      } else {
        let img = image::load_from_memory(&in_buf).expect("failed to decode image");
        println!("{:?} {}x{}", f, img.width(), img.height());
      }
    } else {
      print_error_and_exit("failed to guess image format");
    }
    return;
  }

  let mut img = image::load_from_memory(&in_buf).expect("failed to decode image");
  if width.is_some() && height.is_none() {
    let (w, h) = (img.width(), img.height());
    let aspect_ratio = w as f32 / h as f32;
    height = Some((width.unwrap() as f32 / aspect_ratio) as u32);
    if let Some(fit) = fit.clone() {
      if fit == FitMode::Cover {
        print_error_and_exit("--cover requires both --width and --height");
      }
    }
  }
  if height.is_some() && width.is_none() {
    let (w, h) = (img.width(), img.height());
    let aspect_ratio = w as f32 / h as f32;
    width = Some((height.unwrap() as f32 * aspect_ratio) as u32);
    if let Some(fit) = fit.clone() {
      if fit == FitMode::Cover {
        print_error_and_exit("--cover requires both --width and --height");
      }
    }
  }

  if let (Some(w), Some(h)) = (width, height) {
    let (ow, oh) = (img.width(), img.height());
    if let Some(fit) = fit {
      match fit {
        FitMode::ScaleDown => {
          if w < ow || h < oh {
            img = img.thumbnail(w, h);
          }
        }
        FitMode::Cover => {
          img = img.resize_to_fill(w, h, FilterType::Lanczos3);
        }
        FitMode::Contain => {
          img = img.resize(w, h, FilterType::Lanczos3);
        }
      }
    } else {
      img = img.resize_exact(w, h, FilterType::Lanczos3);
    }
  }

  let format = match format {
    Some(f) => match f.as_str() {
      "jpg" | "jpeg" => ImageFormat::Jpeg,
      "png" => ImageFormat::Png,
      "webp" => ImageFormat::WebP,
      "avif" => ImageFormat::Avif,
      "gif" => ImageFormat::Gif,
      "ico" => ImageFormat::Ico,
      other => {
        print_error_and_exit(&format!("unsupported format: {other}"));
      }
    },
    None => {
      let Some(format) = image::guess_format(&in_buf).ok() else {
        print_error_and_exit("failed to guess image format");
      };
      format
    }
  };

  match format {
    ImageFormat::Jpeg => encode_jpeg(&img, quality),
    ImageFormat::Png => encode_png(&img),
    ImageFormat::WebP => encode_webp(&img, quality),
    ImageFormat::Avif => encode_avif(&img, quality),
    ImageFormat::Ico => encode_ico(&img),
    _ => {
      print_error_and_exit(&format!("unsupported format: {:?}", format));
    }
  }
}

fn encode_jpeg(img: &DynamicImage, quality: u8) {
  let mut stdout = io::stdout().lock();
  let rgb = img.to_rgb8();
  let (w, h) = (rgb.width(), rgb.height());
  let buf = rgb.as_raw();
  let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut stdout, quality);
  enc
    .write_image(buf, w, h, image::ExtendedColorType::Rgb8)
    .expect("failed to encode jpeg");
}

fn encode_png(img: &DynamicImage) {
  let mut stdout = io::stdout().lock();
  let rgba = img.to_rgba8();
  let (w, h) = (rgba.width(), rgba.height());
  let buf = rgba.as_raw();
  let enc = image::codecs::png::PngEncoder::new(&mut stdout);
  enc
    .write_image(buf, w, h, image::ExtendedColorType::Rgba8)
    .expect("error: failed to encode png");
}

fn encode_webp(img: &DynamicImage, quality: u8) {
  let mut stdout = io::stdout().lock();
  let rgba = img.to_rgba8();
  let (w, h) = (rgba.width(), rgba.height());
  let stride = w as i32 * 4;
  unsafe {
    let mut out_buf = std::ptr::null_mut();
    let size = libwebp_sys::WebPEncodeRGBA(rgba.as_ptr(), w as i32, h as i32, stride, quality as f32, &mut out_buf);
    stdout
      .write_all(std::slice::from_raw_parts(out_buf, size).into())
      .expect("error: failed to encode webp");
  }
}

fn encode_avif(img: &DynamicImage, quality: u8) {
  let mut stdout = io::stdout().lock();
  let rgba = img.to_rgba8();
  let (w, h) = (rgba.width(), rgba.height());
  let buf = rgba.as_raw();
  let enc = image::codecs::avif::AvifEncoder::new_with_speed_quality(&mut stdout, 5, quality);
  enc
    .write_image(buf, w, h, image::ExtendedColorType::Rgba8)
    .expect("error: failed to encode avif");
  return;
}

fn encode_ico(img: &DynamicImage) {
  let mut stdout = io::stdout().lock();
  let rgba = img.to_rgba8();
  let (w, h) = (rgba.width(), rgba.height());
  let buf = rgba.as_raw();
  let enc = image::codecs::ico::IcoEncoder::new(&mut stdout);
  enc
    .write_image(buf, w, h, image::ExtendedColorType::Rgba8)
    .expect("error: failed to encode icon");
}

fn print_help_and_exit() {
  print!(
    r#"Usage: reimg [OPTIONS] < input_image_file > output_image_file

Options:
  -w, --width <width>     Set the width of the output image
  -h, --height <height>   Set the height of the output image
  --fit <fit>             Set the fit mode for the resize operation [possible values: cover, contain, scale-down]
    --cover               Resize the image to fill the given dimensions, cropping if necessary
    --contain             Resize the image to fit the given dimensions
    --scale-down          Resize the image to fit the given dimensions, but not larger than the original
  -q, --quality <quality> Set the quality of the output image [default: 85]
  -f, --format <format>   Set the format of the output image [possible values: jpeg, png, webp, avif, ico ]
  -i, --info              Show image metadata
"#
  );
  std::process::exit(0);
}

fn print_error_and_exit(message: &str) -> ! {
  let _ = writeln!(io::stderr(), "error: {message}");
  std::process::exit(1);
}
