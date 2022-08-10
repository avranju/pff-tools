#!/bin/bash

# get path to source root
scriptdir=$(dirname "$(realpath "$0")")
basedir=$(realpath "$scriptdir"/../..)
builddir="$scriptdir/build"

# re-create a temporary place to store build files
rm -rf "$builddir"
mkdir -p "$builddir"

# build the web project
pushd "$basedir/pff-web" || exit
yarn run build
popd || exit

# make release build of pff-web
pushd "$basedir" || exit
cargo build --release
cp "$basedir/target/release/pff-web" "$scriptdir/build/pff-web"

cp -r "$basedir/pff-web/www" "$scriptdir/build/www"
cp "$scriptdir/Dockerfile" "$scriptdir/build/Dockerfile"
popd || exit

# build docker image
pushd "$builddir" || exit
docker build -t avranju/pff-web:"$1" .
popd || exit
