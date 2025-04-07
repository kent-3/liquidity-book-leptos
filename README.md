# liquidity-book-leptos

This is a Client-Side-Rendered App demonstrating how to interact with the
[liquidity-book](https://github.com/kent-3/liquidity-book) contracts,
using the [Leptos](https://leptos.dev/) framework.

## Getting Started

If you don’t already have it installed, you can install Trunk by running

```bash
cargo install trunk
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
trunk build --features charts --release --public-url "https://kent-3.github.io/liquidity-book-leptos/"
```

`trunk build` will create a number of build artifacts in a `dist/` directory.
Publishing `dist` somewhere online should be all you need to deploy your app.
This should work very similarly to deploying any JavaScript application.

## Features

### Chart Visualization

Charts functionality is optional and gated behind the charts feature flag to optimize compilation times.
Enable charts by adding the --features charts flag to your build or serve commands.

#### Chart Development

To work on charts independently from the main application:

```bash
cd crates/charts
trunk serve --open
```

This launches a standalone development environment specifically for chart components, allowing for faster iteration and testing of chart functionality.
The chart development server runs on 127.0.0.1:8081, enabling you to run it concurrently with the main application's development server.

### Network Configuration

Configure which blockchain network the application connects to using feature flags:

```bash
# For development network
trunk serve --features devnet

# With charts enabled on development network
trunk serve --features "devnet charts"
```
