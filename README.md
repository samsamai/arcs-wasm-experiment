# ARCS (A Rust CAD System) Wasm Experiment

## About

This repo contains an experiment to create a basic example app using the [ARCS](https://github.com/Michael-F-Bryan/arcs) crate
running on the web.

To begin with, [wasm-pack-template](https://rustwasm.github.io/docs/book/game-of-life/hello-world.html) was used to make a starter rust-webassembly app and then ARCS was added in.

## Setup

There are 2 requirements: wasm-pack and npm. Make sure you have them installed.

### wasm-pack

[Get wasm-pack here!](https://rustwasm.github.io/wasm-pack/installer/)

### npm

[Follow these instructions to install npm.](https://www.npmjs.com/get-npm)

## Usage

Build the wasm from the main dir:

```
wasm-pack build
```

Install npm modules from the `www` dir:

```
cd www
npm install
```

Run the server from the `www` dir:

```
npm run start
```

View the result in the browser using the address shown when the server is started:

```
.
.
ℹ ｢wds｣: Project is running at http://localhost:8080/
ℹ ｢wds｣: webpack output is served from /
.
.
```
