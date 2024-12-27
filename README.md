# liquidity-book-leptos

This is a Client-Side-Rendered App demonstrating how to interact with the
[liquidity-book](https://github.com/kent-3/liquidity-book) contracts,
using the [Leptos](https://leptos.dev/) framework.

## Getting Started

If you don’t already have it installed, you can install Trunk by running

```bash
cargo install trunk
```

To use nightly Rust, you can run

```bash
rustup toolchain install nightly
rustup default nightly
```

Make sure you've added the `wasm32-unknown-unknown` target so that Rust can
compile your code to WebAssembly to run in the browser.

```bash
rustup target add wasm32-unknown-unknown
```

## Tailwind

Trunk handles the Tailwind build step. Include a line like this in your `index.html` head:

```html
<link data-trunk rel="tailwind-css" href="input.css" />
```

## Developing

Start a development server at 127.0.0.1:8080:

```bash
trunk serve

# or start the server and open the app in a new browser tab
trunk serve --open
```

## Building

To create a production version of your app:

```bash
trunk build --release --public-url "https://kent-3.github.io/liquidity-book-leptos/"
```

`trunk build` will create a number of build artifacts in a `dist/` directory.
Publishing `dist` somewhere online should be all you need to deploy your app.
This should work very similarly to deploying any JavaScript application.
