# reimg

A simple CLI tool for web image resizing, using [image-rs](https://github.com/image-rs/image).

## Installation

```bash
cargo install reimg
```

You can also download the prebuilt binary from the [release page](https://github.com/ije/reimg/releases)

## Usage

```bash
reimg -w 512 -h 512 --cover input.jpg output.avif
```

reimg uses stdin/stdout as the input/output if no input/output files provided:

```js
const out = await Bun.$`reimg -w ${512} -h ${512} --cover -f avif < ${Bun.file("input.jpg")}`.quiet()
const res = new Response(out.arrayBuffer(), {
  headers: {
    "Content-Type": "image/avif"
  }
})
```

## Options

```
$ reimg

Usage: reimg [OPTIONS] [input_file] [output_file]

Options:
  -w, --width   <width>          Set the width of the output image
  -h, --height  <height>         Set the height of the output image
  -s, --size    <width>x<height> Set the width and height of the output image
  --scale       <scale>          Scale the output image by a factor [default: 1.0]
    --2x                         Scale the output image by 2x
    --3x                         Scale the output image by 3x
    --4x                         Scale the output image by 4x
  --fit         <fit>            Set the fit mode for the resize operation [possible values: cover, contain, scale-down]
    --cover                      Resize the image to fill the given dimensions, cropping if necessary
    --contain                    Resize the image to fit the given dimensions
    --scale-down                 Resize the image to fit the given dimensions, but not larger than the original
  -q, --quality <quality>        Set the quality of the output image [default: 85]
  --format      <format>         Set the format of the output image [possible values: jpeg, png, webp, avif, ico]
  --data-url                     Show the data url of the resized image
  -i, --info                     Show image metadata
```

## License

MIT
