#!/bin/sh
for file in *.*; do if [[ ! "$file" =~ \.webp$ ]]; then magick "$file" -resize 800x -quality 80 "${file%.*}.webp"; fi; done
