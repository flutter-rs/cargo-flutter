# Cargo plugin for building flutter applications

## Getting started
```sh
cargo install cargo-flutter
```

## Usage

- Create a flutter-rs app

    `git clone https://github.com/flutter-rs/flutter-app-template`

- Run a flutter-rs app in dev mode

    `cargo flutter run`

- Bundle a flutter-rs app for distribution

    `cargo flutter --format appimage build --release`

- Run `flutter_driver` tests

    `cargo flutter --dart-main test_driver/app.dart --drive run`

## Supported targets
- x86_64-unknown-linux-gnu

## Supported formats
- AppImage

## License
ISC License

Copyright (c) 2019, flutter-rs

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE
OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.
