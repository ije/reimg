# reimg

A simple CLI tool to resize images.

## Installation

```bash
cargo install reimg
```

## Usage

```bash
reimg -w 512 -h 512 --cover -f avif < input.jpg > output.avif
```

## Options

```
reimg -h

Usage: reimg [OPTIONS] < input_image_file > output_image_file

Options:
  -w, --width <width>     Set the width of the output image
  -h, --height <height>   Set the height of the output image
  --cover                 Resize the image to fill the given width and height, cropping if necessary
  --contain               Resize the image to fit the given width and height
  --scale-down            Resize the image to fit the given width and height, but not larger than the original
  -q, --quality <quality> Set the quality of the output image [default: 85]
  -f, --format <format>   Set the format of the output image [possible values: jpeg, png, webp, avif, ico ]
  -i, --info              Show image metadata
  -h, --help              Print help message
```

## License

MIT
