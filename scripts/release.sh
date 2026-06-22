#!/usr/bin/env bash
# Copyright (c) 2026 AltDrive, LLC
# SPDX-License-Identifier: Apache-2.0
# Nyx Backup Recovery - https://nyxbackup.com
#
# Publish a release: checksum the installers staged in dist/ for the current
# VERSION and create a GitHub Release tagged vVERSION with every artifact
# attached.  A recovery tool is downloaded under duress onto possibly-untrusted
# machines, so a published SHA-256 manifest is the minimum trust bar - this
# script always emits one.
#
# This does NOT build anything.  Build first with the per-platform scripts:
#   scripts/windows/build_windows_x86_64.sh && scripts/windows/build_recover_msi_x86_64.sh
#   scripts/windows/build_windows_arm64.sh  && scripts/windows/build_recover_msi_arm64.sh
#   scripts/linux/build_linux_x86_64.sh     && scripts/linux/build_recover_deb_x86_64.sh
#   scripts/linux/build_linux_arm64.sh      && scripts/linux/build_recover_deb_arm64.sh
#   scripts/linux/build_recover_rpm.sh [--arch arm64]   (repackages the .deb as .rpm)
#   scripts/macos/build_recover_pkg_arm64.sh    (on a Mac)
#
# Prerequisites:
#   - Artifacts present in dist/ for the current VERSION.
#   - `gh` CLI authenticated (gh auth login) and a GitHub remote configured.
#
# Usage:
#   scripts/release.sh             # checksum dist/, create the GitHub release
#   scripts/release.sh --dry-run   # checksum + print the gh command; do not publish

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(tr -d ' \t\n\r' < VERSION)"
TAG="v${VERSION}"
DRY_RUN=0
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=1

# Collect only the artifacts matching this VERSION (skip stale dist/ leftovers).
shopt -s nullglob
ARTIFACTS=( dist/*"${VERSION}"*.msi dist/*"${VERSION}"*.deb dist/*"${VERSION}"*.rpm \
            dist/*"${VERSION}"*.dmg dist/*"${VERSION}"*.pkg )
shopt -u nullglob

if [[ ${#ARTIFACTS[@]} -eq 0 ]]; then
    echo "ERROR: no dist/ artifacts found for version ${VERSION}." >&2
    echo "  Build the installers first (see header)." >&2
    exit 1
fi

echo "Version: ${VERSION}   Tag: ${TAG}"
echo "Artifacts:"
printf '  %s\n' "${ARTIFACTS[@]}"

# SHA-256 manifest, relative names so `sha256sum -c` works from inside dist/.
SUMS_NAME="SHA256SUMS-${VERSION}.txt"
( cd dist && sha256sum $(for a in "${ARTIFACTS[@]}"; do basename "$a"; done) > "${SUMS_NAME}" )
SUMS="dist/${SUMS_NAME}"
echo "Checksums (${SUMS}):"
cat "${SUMS}"

if [[ ${DRY_RUN} -eq 1 ]]; then
    echo
    echo "[dry-run] would run:"
    echo "  gh release create ${TAG} ${ARTIFACTS[*]} ${SUMS} --title \"Nyx Backup Recovery ${VERSION}\" --notes ..."
    exit 0
fi

command -v gh >/dev/null 2>&1 || {
    echo "ERROR: gh CLI not found or not authenticated (run: gh auth login)." >&2
    exit 1
}

gh release create "${TAG}" "${ARTIFACTS[@]}" "${SUMS}" \
    --title "Nyx Backup Recovery ${VERSION}" \
    --notes "Standalone, open-source recovery tool ${VERSION}. Verify your download against ${SUMS_NAME} before running it."

echo "Release ${TAG} created with ${#ARTIFACTS[@]} artifact(s) + checksums."
