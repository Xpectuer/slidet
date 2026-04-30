#!/usr/bin/env bash

set -euo pipefail

REPO_OWNER="Xpectuer"
REPO_NAME="slidet"
BINARY_NAME="slidet"

log() {
  printf '==> %s\n' "$*"
}

warn() {
  printf 'warning: %s\n' "$*" >&2
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

download() {
  local url="$1"
  local output="$2"

  if command -v curl >/dev/null 2>&1; then
    curl --fail --location --silent --show-error "$url" --output "$output"
  elif command -v wget >/dev/null 2>&1; then
    wget --quiet "$url" --output-document="$output"
  else
    die "curl or wget is required"
  fi
}

detect_suffix() {
  local os
  local arch

  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Darwin)
      case "$arch" in
        arm64|aarch64)
          printf 'macos-aarch64\n'
          ;;
        x86_64)
          printf 'macos-x86_64\n'
          ;;
        *)
          die "unsupported macOS architecture: $arch"
          ;;
      esac
      ;;
    Linux)
      case "$arch" in
        x86_64)
          printf 'linux-musl-x86_64\n'
          ;;
        *)
          die "unsupported Linux architecture: $arch"
          ;;
      esac
      ;;
    *)
      die "unsupported operating system: $os"
      ;;
  esac
}

choose_install_dir() {
  local path_entry

  IFS=':' read -r -a path_entries <<< "${PATH:-}"
  for path_entry in "${path_entries[@]}"; do
    case "$path_entry" in
      "$HOME/.local/bin"|"$HOME/bin")
        printf '%s\n' "$path_entry"
        return 0
        ;;
    esac
  done

  printf '%s\n' "$HOME/.local/bin"
}

ensure_path_configured() {
  local install_dir="$1"

  case ":${PATH:-}:" in
    *":$install_dir:"*)
      return 0
      ;;
  esac

  local shell_name rc_file
  shell_name="$(basename "${SHELL:-}")"

  case "$shell_name" in
    zsh)
      rc_file="$HOME/.zshrc"
      if ! grep -Fqs "$install_dir" "$rc_file" 2>/dev/null; then
        printf '\nexport PATH="%s:$PATH"\n' "$install_dir" >> "$rc_file"
        log "added $install_dir to PATH in $rc_file"
      fi
      ;;
    bash)
      rc_file="$HOME/.bashrc"
      if ! grep -Fqs "$install_dir" "$rc_file" 2>/dev/null; then
        printf '\nexport PATH="%s:$PATH"\n' "$install_dir" >> "$rc_file"
        log "added $install_dir to PATH in $rc_file"
      fi
      ;;
    fish)
      rc_file="$HOME/.config/fish/config.fish"
      mkdir -p "$(dirname "$rc_file")"
      if ! grep -Fqs "$install_dir" "$rc_file" 2>/dev/null; then
        printf '\nfish_add_path %s\n' "$install_dir" >> "$rc_file"
        log "added $install_dir to PATH in $rc_file"
      fi
      ;;
    *)
      warn "could not update PATH automatically for shell: ${SHELL:-unknown}"
      warn "add this directory to PATH manually: $install_dir"
      ;;
  esac
}

verify_checksum() {
  local archive_path="$1"
  local checksum_path="$2"
  local archive_dir checksum_name

  archive_dir="$(dirname "$archive_path")"
  checksum_name="$(basename "$checksum_path")"

  if command -v shasum >/dev/null 2>&1; then
    (cd "$archive_dir" && shasum -a 256 --check "$checksum_name")
  elif command -v sha256sum >/dev/null 2>&1; then
    (cd "$archive_dir" && sha256sum --check "$checksum_name")
  else
    warn "shasum/sha256sum not found; skipping checksum verification"
    return 0
  fi
}

main() {
  need_cmd uname
  need_cmd tar
  need_cmd mktemp
  need_cmd chmod
  need_cmd mkdir
  need_cmd find
  need_cmd install

  local suffix archive_name archive_url checksum_url
  local install_dir archive_path checksum_path unpack_dir binary_path

  suffix="$(detect_suffix)"

  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' EXIT

  install_dir="$(choose_install_dir)"
  mkdir -p "$install_dir"

  local latest_location version
  if command -v curl >/dev/null 2>&1; then
    latest_location="$(curl --silent --show-error --location --write-out '%{url_effective}' --output /dev/null "https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest")"
    version="${latest_location##*/}"
  elif command -v wget >/dev/null 2>&1; then
    latest_location="$(wget -S --max-redirect=0 "https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest" 2>&1 | awk '/^  Location: / {print $2}' | tail -n1 | tr -d '\r')"
    version="${latest_location##*/}"
  else
    die "curl or wget is required"
  fi

  [[ -n "$version" ]] || die "failed to resolve latest release version"

  archive_name="${BINARY_NAME}-${version}-${suffix}.tar.gz"
  archive_url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${version}/${archive_name}"
  checksum_url="${archive_url}.sha256"
  archive_path="${tmp_dir}/${archive_name}"
  checksum_path="${archive_path}.sha256"
  unpack_dir="${tmp_dir}/unpack"

  log "downloading ${archive_name}"
  download "$archive_url" "$archive_path"

  log "downloading checksum"
  download "$checksum_url" "$checksum_path"

  log "verifying checksum"
  verify_checksum "$archive_path" "$checksum_path"

  mkdir -p "$unpack_dir"
  tar -xzf "$archive_path" -C "$unpack_dir"

  binary_path="$(find "$unpack_dir" -type f -name "$BINARY_NAME" | head -n1)"
  [[ -n "$binary_path" ]] || die "failed to locate ${BINARY_NAME} in archive"

  install -m 0755 "$binary_path" "${install_dir}/${BINARY_NAME}"
  ensure_path_configured "$install_dir"

  log "installed ${BINARY_NAME} ${version} to ${install_dir}/${BINARY_NAME}"
  log "run '${BINARY_NAME} --version' to verify the installation"

  case ":${PATH:-}:" in
    *":$install_dir:"*)
      ;;
    *)
      warn "open a new shell or run: export PATH=\"${install_dir}:\$PATH\""
      ;;
  esac
}

main "$@"
