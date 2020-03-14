#!/bin/sh
set -e
for size in 192 512 ; do
  output="icon${size}.png"
  inkscape --without-gui \
	   --export-png="${output}" \
	   --export-width="${size}" \
	   --export-height="${size}" \
	   icon2.svg
  optipng -strip all "${output}"
done
