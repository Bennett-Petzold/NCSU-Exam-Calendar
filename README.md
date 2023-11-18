# GitHub Pages Host
https://bennett-petzold.github.io/NCSU-Exam-Calendar/

Go here to use the calendar.

# NCSU Exam Calendar
This repackages the NCSU exam calendar (https://studentservices.ncsu.edu/calendars/exam-calendar/) into a form I,
at least,
find easier to search than CTRL-F and manual review on the official page.

This is a first brush at anything major in HTML, React-style architecture, Rust Dioxus layout, and WASM.
It is not (but one day may be) elegant or pretty.

## Using In-Browser
See the first item to use the github pages host.
This is pushed to the docs/ folder.

To host locally install both dixous-cli (`cargo install dioxus-cli`) and the rust target `wasm32-unknown-unknown`.
Download and build dioxus-cli locally -- parts of the WASM ecosystem require exact version matching at the moment.
Run with `diouxs serve --features web`.

## Using Desktop
Make sure you have WebView (https://dioxuslabs.com/learn/0.4/getting\_started/desktop).
Run `cargo run --features desktop --bin ncsu_exam_desktop`.
To bundle for desktop, switch the default bin from web to desktop and use `bundle` from the dioxus-cli.

## Using CLI
`cargo run --bin ncsu_exam_cli`.
Currently just creates the JSON that is fed into the GUI.
Can be taken directly from target as a standalone binary.

## Publishing updates
Use `dx build --features web --release` for the up to date web version before merging to main (https://dioxuslabs.com/learn/0.4/cookbook/publishing).
The release build is drastically smaller (85 MB -> 5.2 MB), which is nicer for both GitHub and users.

## Graphics
Yes, the styling is the defaults.
No, I do not plan on improving them.
Tailwind is set-up to use `input.css`,
if you want to improve style feel free to edit that and uncomment line 38 in `Dioxus.toml` to enable the tailwind CSS.

Unlike the CSS, I am interested in comments about the actual HTML layout.

## Current Issues
- Major refactor planned to make code less of a monolithic mess
- Have not figured out a way to pull the JSON from the server we are hosted on, instead of injecting it into the application
- Looks like it is impossible to download the NCSU webpage for reformatting on the live application due to CORS
- Version numbers and web default data file are currently manual
