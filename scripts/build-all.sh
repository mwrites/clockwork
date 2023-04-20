#!/usr/bin/env bash

set -e

usage() {
  exitcode=0
  if [[ -n "$1" ]]; then
    exitcode=1
    echo "Error: $*"
  fi
  cat <<EOF
usage: $0 [+<cargo version>] [--release] [--target <target triple>] <install directory>
EOF
  exit $exitcode
}

# Set default target triple from 'cargo -vV'
defaultTargetTriple=$(cargo -vV | grep 'host:' | cut -d ' ' -f2)

# Set build flags
maybeRustVersion=
installDir=
buildVariant=debug
maybeReleaseFlag=
targetTriple="$defaultTargetTriple"
while [[ -n $1 ]]; do
  if [[ ${1:0:1} = - ]]; then
    case $1 in
      --release)
        maybeReleaseFlag=--release
        buildVariant=release
        shift
        ;;
      --target)
        targetTriple=$2
        shift 2
        ;;
      *)
        usage "Unknown option: $1"
        ;;
    esac
  elif [[ ${1:0:1} = + ]]; then
    maybeRustVersion=$1
    shift
  else
    installDir=$1
    shift
  fi
done

# If target triple is still unset, use default
if [[ -z "$targetTriple" ]]; then
  targetTriple="$defaultTargetTriple"
fi

# Print final configuration
echo "Using Rust version: ${maybeRustVersion:1}"
echo "Build variant: $buildVariant"
echo "Target triple: $targetTriple"
echo "Install directory: $installDir"
echo "Release flag: ${maybeReleaseFlag:---not-set}"

# set -x
# echo $PATH
# export PATH=~/.local/share/solana/install/active_release/bin:$PATH
# ls ~/.local/share/solana/install/active_release/bin
# command -v cargo-build-bpf
# cargo-build-bpf -V
# command -v cargo-build-sbf
# cargo-build-sbf -V
# command -v anchor
# anchor -V
# rustup toolchain list

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

# Build programs
(
  set -x
  anchor build
)

# Define lib extension
case $targetTriple in
  *darwin*)
    pluginFilename=libclockwork_plugin.dylib
    ;;
  *)
    pluginFilename=libclockwork_plugin.so
    ;;
esac

# Build the repo
(
  set -x
  cargo "$maybeRustVersion" build --locked $maybeReleaseFlag "${binArgs[@]}" --lib --target "$targetTriple"

  # Copy binaries
  cp -fv "target/$targetTriple/$buildVariant/$pluginFilename" "$installDir"/lib
  for bin in "${BINS[@]}"; do
    rm -fv "$installDir/bin/$bin"

    cp -fv "target/$targetTriple/$buildVariant/$bin" "$installDir/bin"
  done

  cp -fv "target/deploy/clockwork_network_program.so" "$installDir/lib"
  cp -fv "target/deploy/clockwork_thread_program.so" "$installDir/lib"
  cp -fv "target/deploy/clockwork_webhook_program.so" "$installDir/lib"
)

# Success message
echo "Done after $SECONDS seconds"
echo 
echo "To use these binaries:"
echo "  export PATH=\"$installDir\"/bin:\"\$PATH\""
