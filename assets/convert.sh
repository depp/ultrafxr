#!/bin/sh
set -e
for size in 192 512 ; do
  output="tile${size}.png"
  inkscape --without-gui \
	   --export-png="${output}" \
	   --export-width="${size}" \
	   --export-height="${size}" \
	   tile.svg
  optipng -strip all "${output}"
done
