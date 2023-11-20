#!/usr/bin/env bash

set -eo pipefail

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

cargo build -p cli --release
web/optimize.bash bundle
dx bundle --bin ncsu_exam_calendar_desktop --release
