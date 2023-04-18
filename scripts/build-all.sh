#!/usr/bin/env bash

set -e

usage() {
  exitcode=0
  if [[ -n "$1" ]]; then
    exitcode=1
    echo "Error: $*"
  fi
  cat <<EOF
usage: $0 [+<cargo version>] [--debug] <install directory>
EOF
  exit $exitcode
}

# Set build flags
maybeRustVersion=
installDir=
buildVariant=debug
maybeReleaseFlag=
while [[ -n $1 ]]; do
  if [[ ${1:0:1} = - ]]; then
    if [[ $1 = --release ]]; then
      maybeReleaseFlag=--release
      buildVariant=release
      shift
    else
      usage "Unknown option: $1"
    fi
  elif [[ ${1:0:1} = \+ ]]; then
    maybeRustVersion=$1
    shift
  else
    installDir=$1
    shift
  fi
done

# Get the output filetype
if [[ $OSTYPE == darwin* ]]; then
  libExt=dylib 
elif [[ $OSTYPE == linux* ]]; then
  libExt=so
else 
  echo OS unsupported
  exit 1
fi

# Check the install directory is provided
if [[ -z "$installDir" ]]; then
  usage "Install directory not specified"
  exit 1
fi

# Create the install directory
installDir="$(mkdir -p "$installDir"; cd "$installDir"; pwd)"
mkdir -p "$installDir/lib"
mkdir -p "$installDir/bin"
echo "Install location: $installDir ($buildVariant)"
cd "$(dirname "$0")"/..
SECONDS=0

# Enumerate the bins
BINS=(
  clockwork
)

# Create bin args
binArgs=()
for bin in "${BINS[@]}"; do 
  binArgs+=(--bin "$bin")
done

# Build the repo
(
  set -x
  rustc -V
  cargo "$maybeRustVersion" build --locked $maybeReleaseFlag "${binArgs[@]}" --lib
)

# Copy binaries
cp -fv "target/$buildVariant/libclockwork_plugin.$libExt" "$installDir"/lib
for bin in "${BINS}"; do
  rm -fv "$installDir/bin/$bin"
  cp -fv "target/$buildVariant/$bin" "$installDir"/bin
done


# Build programs
if command -v anchor &> /dev/null; then
  set -x
  anchor build
fi

# Success message
echo "Done after $SECONDS seconds"
echo 
echo "To use these binaries:"
echo "  export PATH=\"$installDir\"/bin:\"\$PATH\""
