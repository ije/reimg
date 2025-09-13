# reimg

A simple CLI tool for web image resizing, using [image-rs](https://github.com/image-rs/image).

## Installation

```bash
cargo install reimg
```

## Usage

```bash
reimg -w 512 -h 512 --cover -f avif < input.jpg > output.avif
```

This tool uses stdin/stdout to read and write data. That allows you to use it as a pipeline. For example, you can use it in a Bun app to resize an image.

```js
import { $ } from "bun";

const out = await $`reimg -w ${512} -h ${512} --cover -f avif < ${Bun.file("input.jpg")}`;
const res = new Response(out.arrayBuffer(), {
  headers: {
    "Content-Type": "image/avif",
  },
});
```

## Options

```
$ reimg

Usage: reimg [OPTIONS] < input_image_file > output_image_file

Options:
  -w, --width <width>     Set the width of the output image
  -h, --height <height>   Set the height of the output image
  -fit <fit>              Set the fit mode for the resize operation [possible values: cover, contain, scale-down]
    --cover               Resize the image to fill the given dimensions, cropping if necessary
    --contain             Resize the image to fit the given dimensions
    --scale-down          Resize the image to fit the given dimensions, but not larger than the original
  -q, --quality <quality> Set the quality of the output image [default: 85]
  -f, --format <format>   Set the format of the output image [possible values: jpeg, png, webp, avif, ico ]
  -i, --info              Show image metadata
```

## License

MIT
