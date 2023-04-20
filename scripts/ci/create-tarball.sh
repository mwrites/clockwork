#!/usr/bin/env bash
set -e

usage() {
  echo "Usage: $0 [--target <target triple>]"
  exit 1
}

TARGET=$(cargo -vV | awk '/host:/ {print $2}')
while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      TARGET=$2
      shift 2
      ;;
    *)
      usage
      ;;
  esac
done

RELEASE_BASENAME="${RELEASE_BASENAME:=clockwork-geyser-plugin-release}"
TARBALL_BASENAME="${TARBALL_BASENAME:="$RELEASE_BASENAME"}"

echo --- Creating release tarball
(
  var=$(pwd)
  echo "The current working directory $var"

  set -x
  rm -rf "${RELEASE_BASENAME:?}"/
  mkdir "${RELEASE_BASENAME}"/

  cat > "${RELEASE_BASENAME}"/version.yml << EOL
  channel: ${CI_TAG}
  commit: ${COMMIT}
  target: ${TARGET}
EOL

  var=$(pwd)
  echo "The current working directory $var"

  source ./scripts/ci/rust-version.sh stable
  ./scripts/build-all.sh +"${rust_stable:?}" --release --target "$TARGET" "${RELEASE_BASENAME}-${TARGET}"

  rm -rf "${RELEASE_BASENAME}"
  mv "${RELEASE_BASENAME}-${TARGET}" "${RELEASE_BASENAME}"

  tar cvf "${TARBALL_BASENAME}".tar "${RELEASE_BASENAME}"
  bzip2 -f "${TARBALL_BASENAME}".tar
  cp "${RELEASE_BASENAME}"/version.yml "${TARBALL_BASENAME}-${TARGET}".yml
)

# Make CHANNEL available to include in the software version information
export CHANNEL

echo --- ok
