#!/bin/bash

# browserify
yarn --silent browserify js/app.js js/reload.js -o www/js/bundle.js

# post-process css with tailwind
yarn tailwindcss -i styles/input.css -o www/styles/app.css