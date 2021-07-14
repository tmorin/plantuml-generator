# plantuml-generator

[![Docker Image Version (latest semver)](https://img.shields.io/docker/v/thibaultmorin/plantuml-generator?label=thibaultmorin%2Fplantuml-generator)](https://hub.docker.com/r/thibaultmorin/plantuml-generator)

> A command line utility to generate stuff with and for PlantUML.

## Introduction

`plantuml-generator` is a re-write of the crafted NodeJS app provided by [tmorin/plantuml-libs].
It provides commands to generate the stuff for PlantUML diagrams like library of PlantUML resources or jut PlantUML diagrams rendering.

[tmorin/plantuml-libs]: https://github.com/tmorin/plantuml-libs

## Dependencies

- `libssl-dev`
- `java`, version > 11
- `inkscape`, version > 1, required only if used to generate a library

## Install

By cargo
```shell
cargo install plantuml-generator
```

By Docker
```shell
docker run --rm thibaultmorin/plantuml-generator --help
```

From the binaries available in the [GitHub Release]:
```shell
plantuml-generator --help
```

[GitHub Release]: https://github.com/tmorin/plantuml-generator/releases

## Commands

The tool provides two commands:

- `library generate` generates a PlantUML library based on a provided manifest
- `diagram generate` generates `.puml` discovered recursively in the file system

## Release

- https://lib.rs/crates/convco
- https://lib.rs/crates/cargo-release

```shell
cargo install convco
cargo install cargo-release
```
