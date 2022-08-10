#!/bin/bash

# browserify & minify
echo "Building JS..."
yarn --silent browserify js/app.js js/reload.js | yarn uglifyjs --compress --mangle -o www/js/bundle.js

# post-process css with tailwind
echo "Building CSS..."
yarn tailwindcss --minify -i styles/input.css -o www/styles/app.css