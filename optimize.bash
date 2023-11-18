#!/usr/bin/env bash

set -eo pipefail

cd "$(dirname "$0")"

if command -v 'dioxus' > /dev/null; then
	dioxus="dixous"
fi

if command -v 'dx' > /dev/null; then
	dioxus="dx"
fi

if ! [[ -v dioxus ]]; then
	printf '%s/n' 'No dioxus-cli installation found, see https://dioxuslabs.com/learn/0.4/CLI/installation'
	exit
fi

if ! command -v 'wasm-opt' > /dev/null; then
	printf '%s\n' 'No wasm-opt installation found, download with https://github.com/WebAssembly/binaryen'
	exit
fi

dx build --features web --release
printf '%s\n' 'dioxus output size:'
du -sh docs/

find ./docs -iname "*.wasm" -exec sh -c 'wasm-opt "$1" -o "$1" -all -Oz' shell {} \;
printf '%s\n' 'wasm-opt size:'
du -sh docs/
