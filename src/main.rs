use base64::Engine;
use image::imageops::FilterType;
use image::{DynamicImage, ImageEncoder, ImageFormat, ImageReader};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

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
  let mut scale: Option<f32> = None;
  let mut fit: Option<FitMode> = None;
  let mut quality: u8 = 85;
  let mut format: Option<String> = None;
  let mut data_url: bool = false;
  let mut positionals = Vec::new();

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
      "-s" | "--size" => {
        let Some(val) = args.next() else {
          print_error_and_exit("-s/--size requires a value");
        };
        let Some((w, h)) = parse_size(&val) else {
          print_error_and_exit("size must be in the format <width>x<height>");
        };
        width = Some(w);
        height = Some(h);
      }
      "--scale" => {
        let Some(val) = args.next() else {
          print_error_and_exit("--scale requires a value");
        };
        match val.parse::<f32>() {
          Ok(v) if v.is_finite() && v > 0.0 => {
            scale = Some(v);
          }
          _ => {
            print_error_and_exit("scale must be a positive number");
          }
        }
      }
      "--2x" => {
        scale = Some(2.0);
      }
      "--3x" => {
        scale = Some(3.0);
      }
      "--4x" => {
        scale = Some(4.0);
      }
      "--5x" => {
        scale = Some(5.0);
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
      "--format" => {
        let Some(val) = args.next() else {
          print_error_and_exit("--format requires a value");
        };
        format = Some(val);
      }
      "--data-url" => {
        data_url = true;
      }
      _ if arg.starts_with('-') => {
        print_help_and_exit();
      }
      _ => {
        positionals.push(arg);
      }
    }
  }

  if positionals.len() > 2 {
    print_error_and_exit("too many positional arguments");
  }
  let input_file = positionals.first().cloned();
  let output_file = positionals.get(1).cloned();

  let in_buf = read_input(input_file.as_deref());

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

  if let Some(scale) = scale {
    if width.is_none() && height.is_none() {
      width = Some(scale_dimension(img.width(), scale));
      height = Some(scale_dimension(img.height(), scale));
    } else {
      width = width.map(|w| scale_dimension(w, scale));
      height = height.map(|h| scale_dimension(h, scale));
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
    Some(f) => parse_image_format(&f),
    None => output_file
      .as_deref()
      .and_then(image_format_from_path)
      .or_else(|| image::guess_format(&in_buf).ok())
      .unwrap_or_else(|| {
        print_error_and_exit("failed to guess output image format");
      }),
  };

  let mut out_buf = match format {
    ImageFormat::Jpeg => encode_jpeg(&img, quality),
    ImageFormat::Png => encode_png(&img),
    ImageFormat::WebP => encode_webp(&img, quality),
    ImageFormat::Avif => encode_avif(&img, quality),
    ImageFormat::Ico => encode_ico(&img),
    _ => {
      print_error_and_exit(&format!("unsupported output image format: {:?}", format));
    }
  };

  if data_url && output_file.is_none() {
    let encoded = base64::engine::general_purpose::STANDARD.encode(out_buf);
    out_buf = format!("data:{};base64,{}", mime_type(format), encoded).into_bytes();
  }

  write_output(output_file.as_deref(), &out_buf);
}

fn parse_size(size: &str) -> Option<(u32, u32)> {
  let (width, height) = size.split_once('x')?;
  let width = width.parse::<u32>().ok()?;
  let height = height.parse::<u32>().ok()?;
  if width == 0 || height == 0 {
    return None;
  }
  Some((width, height))
}

fn scale_dimension(dimension: u32, scale: f32) -> u32 {
  ((dimension as f32 * scale).round() as u32).max(1)
}

fn read_input(input_file: Option<&str>) -> Vec<u8> {
  match input_file {
    Some(path) => fs::read(path).unwrap_or_else(|err| {
      print_error_and_exit(&format!("failed to read input file `{path}`: {err}"));
    }),
    None => {
      let mut in_buf = Vec::new();
      io::stdin().read_to_end(&mut in_buf).expect("failed to read stdin");
      in_buf
    }
  }
}

fn write_output(output_file: Option<&str>, out_buf: &[u8]) {
  match output_file {
    Some(path) => fs::write(path, out_buf).unwrap_or_else(|err| {
      print_error_and_exit(&format!("failed to write output file `{path}`: {err}"));
    }),
    None => {
      let mut stdout = io::stdout().lock();
      stdout.write_all(out_buf).expect("failed to write stdout");
    }
  }
}

fn parse_image_format(format: &str) -> ImageFormat {
  match format.to_ascii_lowercase().as_str() {
    "jpg" | "jpeg" => ImageFormat::Jpeg,
    "png" => ImageFormat::Png,
    "webp" => ImageFormat::WebP,
    "avif" => ImageFormat::Avif,
    "gif" => ImageFormat::Gif,
    "ico" => ImageFormat::Ico,
    other => {
      print_error_and_exit(&format!("unsupported format: {other}"));
    }
  }
}

fn image_format_from_path(path: &str) -> Option<ImageFormat> {
  let extension = Path::new(path).extension()?.to_str()?;
  match extension.to_ascii_lowercase().as_str() {
    "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
    "png" => Some(ImageFormat::Png),
    "webp" => Some(ImageFormat::WebP),
    "avif" => Some(ImageFormat::Avif),
    "gif" => Some(ImageFormat::Gif),
    "ico" => Some(ImageFormat::Ico),
    _ => None,
  }
}

fn mime_type(format: ImageFormat) -> &'static str {
  match format {
    ImageFormat::Jpeg => "image/jpeg",
    ImageFormat::Png => "image/png",
    ImageFormat::WebP => "image/webp",
    ImageFormat::Avif => "image/avif",
    ImageFormat::Gif => "image/gif",
    ImageFormat::Ico => "image/x-icon",
    _ => "application/octet-stream",
  }
}

fn encode_jpeg(img: &DynamicImage, quality: u8) -> Vec<u8> {
  let mut out_buf = Vec::new();
  let rgb = img.to_rgb8();
  let (w, h) = (rgb.width(), rgb.height());
  let buf = rgb.as_raw();
  let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out_buf, quality);
  enc
    .write_image(buf, w, h, image::ExtendedColorType::Rgb8)
    .expect("failed to encode jpeg");
  out_buf
}

fn encode_png(img: &DynamicImage) -> Vec<u8> {
  let mut out_buf = Vec::new();
  let rgba = img.to_rgba8();
  let (w, h) = (rgba.width(), rgba.height());
  let buf = rgba.as_raw();
  let enc = image::codecs::png::PngEncoder::new(&mut out_buf);
  enc
    .write_image(buf, w, h, image::ExtendedColorType::Rgba8)
    .expect("error: failed to encode png");
  out_buf
}

fn encode_webp(img: &DynamicImage, quality: u8) -> Vec<u8> {
  let rgba = img.to_rgba8();
  let (w, h) = (rgba.width(), rgba.height());
  let stride = w as i32 * 4;
  unsafe {
    let mut out_buf = std::ptr::null_mut();
    let size = libwebp_sys::WebPEncodeRGBA(rgba.as_ptr(), w as i32, h as i32, stride, quality as f32, &mut out_buf);
    let bytes = std::slice::from_raw_parts(out_buf, size).to_vec();
    libwebp_sys::WebPFree(out_buf.cast());
    bytes
  }
}

fn encode_avif(img: &DynamicImage, quality: u8) -> Vec<u8> {
  let mut out_buf = Vec::new();
  let rgba = img.to_rgba8();
  let (w, h) = (rgba.width(), rgba.height());
  let buf = rgba.as_raw();
  let enc = image::codecs::avif::AvifEncoder::new_with_speed_quality(&mut out_buf, 5, quality);
  enc
    .write_image(buf, w, h, image::ExtendedColorType::Rgba8)
    .expect("error: failed to encode avif");
  out_buf
}

fn encode_ico(img: &DynamicImage) -> Vec<u8> {
  let mut out_buf = Vec::new();
  let rgba = img.to_rgba8();
  let (w, h) = (rgba.width(), rgba.height());
  let buf = rgba.as_raw();
  let enc = image::codecs::ico::IcoEncoder::new(&mut out_buf);
  enc
    .write_image(buf, w, h, image::ExtendedColorType::Rgba8)
    .expect("error: failed to encode icon");
  out_buf
}

fn print_help_and_exit() {
  print!(
    r#"Usage: reimg [OPTIONS] [input_file] [output_file]

Options:
  -w, --width   <width>          Set the width of the output image
  -h, --height  <height>         Set the height of the output image
  -s, --size    <width>x<height> Set the width and height of the output image
  --scale       <scale>          Scale the output image by a factor [default: 1.0]
    --2x                         Scale the output image by 2x
    --3x                         Scale the output image by 3x
    --4x                         Scale the output image by 4x
    --5x                         Scale the output image by 5x
  --fit         <fit>            Set the fit mode for the resize operation [possible values: cover, contain, scale-down]
    --cover                      Resize the image to fill the given dimensions, cropping if necessary
    --contain                    Resize the image to fit the given dimensions
    --scale-down                 Resize the image to fit the given dimensions, but not larger than the original
  -q, --quality <quality>        Set the quality of the output image [default: 85]
  --format      <format>         Set the format of the output image [possible values: jpeg, png, webp, avif, ico]
  --data-url                     Show the data url of the resized image
  -i, --info                     Show image metadata
"#
  );
  std::process::exit(0);
}

fn print_error_and_exit(message: &str) -> ! {
  let _ = writeln!(io::stderr(), "error: {message}");
  std::process::exit(1);
}
