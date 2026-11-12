#!/usr/bin/env bash
set -euo pipefail

if [[ ${1:-} == --help ]]; then
  cat <<'HELP'
Usage: ci/build-macos.sh

Build a release app for this Mac's architecture, ad-hoc sign it, and create:
  target/macos/WezTerm.app
  target/macos/WezTerm-macos.zip

Requires Rust/Cargo, CMake, pkg-config, and Xcode Command Line Tools.
Updates submodules to the revisions recorded in the current checkout.
Replaces the previous generated app and ZIP; does not install or notarize them.
HELP
  exit 0
fi
if [[ $# != 0 || $(uname -s) != Darwin ]]; then
  echo 'Run ci/build-macos.sh without arguments on macOS (or use --help).' >&2
  exit 1
fi

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo"
for tool in cargo rustc cmake pkg-config git xcrun tic zip ditto codesign otool lipo plutil; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "Missing $tool. Install Rust/Cargo, CMake, pkg-config, and Xcode Command Line Tools first." >&2
    exit 1
  fi
done
xcrun --find clang >/dev/null

arch=$(uname -m)
case "$arch" in
  arm64) host=aarch64-apple-darwin ;;
  x86_64) host=x86_64-apple-darwin ;;
  *) echo "Unsupported Mac architecture: $arch" >&2; exit 1 ;;
esac

git submodule update --init --recursive
# Otherwise libgit2-sys may link to Homebrew's libgit2, making the app depend
# on libraries outside the bundle. Explicit paths/target avoid Cargo overrides.
CARGO_TARGET_DIR="$repo/target" LIBGIT2_NO_PKG_CONFIG=1 LIBGIT2_NO_VENDOR=0 \
  cargo build --release --target "$host" \
    -p wezterm-gui -p wezterm -p wezterm-mux-server -p strip-ansi-escapes

output="$repo/target/macos"
stage=$(mktemp -d "$repo/target/macos-build.XXXXXXXX")
trap 'rm -rf "$stage"' EXIT

# Reuse the release packaging recipe, but isolate its ZIP/Homebrew sidecars
# from the checkout. TAG_NAME avoids its git lookup in this staging directory.
ln -s "$repo/assets" "$stage/assets"
ln -s "$repo/ci" "$stage/ci"
ln -s "$repo/termwiz" "$stage/termwiz"
if ! (cd "$stage" && TAG_NAME=local BUILD_REASON=Manual MACOS_TEAM_ID= \
  bash "$repo/ci/deploy.sh" "$repo/target/$host") >"$stage/package.log" 2>&1; then
  cat "$stage/package.log" >&2
  exit 1
fi
mv "$stage/WezTerm-macos-local/WezTerm.app" "$stage/WezTerm.app"
app="$stage/WezTerm.app"

# Fail rather than silently ship an app that needs this Mac's Homebrew install.
for binary in "$app/Contents/MacOS/"*; do
  lipo -verify_arch "$arch" "$binary"
  dependencies=$(otool -L "$binary")
  while IFS= read -r line; do
    case "$line" in
      $'\t'*)
        library=${line#$'\t'}
        library=${library%% \(*}
        case "$library" in
          /System/Library/*|/usr/lib/*) ;;
          *) echo "Unbundled dependency in $binary: $library" >&2; exit 1 ;;
        esac
        ;;
    esac
  done <<< "$dependencies"
done
plutil -lint "$app/Contents/Info.plist"
codesign --force --deep --sign - "$app"
codesign --verify --deep --strict "$app"
ditto -c -k --keepParent "$app" "$stage/WezTerm-macos.zip"

# Keep the previous build until the new bundle has passed verification.
mkdir -p "$output"
rm -rf "$output/WezTerm.app"
mv "$app" "$output/WezTerm.app"
mv -f "$stage/WezTerm-macos.zip" "$output/WezTerm-macos.zip"
printf '\nApp: %s\nZIP: %s\nAd-hoc signed for %s; not Apple-notarized.\n' \
  "$output/WezTerm.app" "$output/WezTerm-macos.zip" "$arch"
