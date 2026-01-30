# plantuml-generator

[![Docker Image Version (latest semver)](https://img.shields.io/docker/v/thibaultmorin/plantuml-generator?label=thibaultmorin%2Fplantuml-generator)](https://hub.docker.com/r/thibaultmorin/plantuml-generator)

> A command line utility to generate stuff with and for PlantUML.

## Introduction

`plantuml-generator` is a re-write of the crafted NodeJS app provided by [tmorin/plantuml-libs].
It provides commands to generate the stuff for PlantUML diagrams like library of PlantUML resources or just PlantUML diagrams rendering.

[tmorin/plantuml-libs]: https://github.com/tmorin/plantuml-libs

## Dependencies

- `libssl-dev`
- `java`, version > 11
- `inkscape`, version > 1.2
  - tested with `Inkscape 1.2 (1:1.2.1+202207142221+cd75a1ee6d)`
  - only required if used to generate a library

### Optional: GraphViz

**GraphViz is no longer required** for diagram generation. The tool automatically uses PlantUML's built-in [smetana layout engine](https://plantuml.com/smetana02) when GraphViz (`dot`) is not installed or the `GRAPHVIZ_DOT` environment variable is not set.

This automatic fallback allows you to generate diagrams without installing GraphViz, simplifying deployment and reducing dependencies. The smetana layout produces high-quality output comparable to GraphViz for most use cases.

If you prefer to use GraphViz or other layout engines, you can:
- Install GraphViz (the `dot` binary will be auto-detected)
- Set the `GRAPHVIZ_DOT` environment variable to point to your `dot` binary
- Override the layout engine using the `--args` option: `--args "-Playout=elk"` or `--args "-Playout=visjs"`

## Install

By Script
```shell
curl -s "https://raw.githubusercontent.com/tmorin/plantuml-generator/master/scripts/install_pgen.sh"  | bash
```

By Docker
```shell
docker run --rm thibaultmorin/plantuml-generator --help
```

By cargo
```shell
cargo install plantuml-generator
```

From the binaries available in the [GitHub Release]:
```shell
plantuml-generator --help
```

[GitHub Release]: https://github.com/tmorin/plantuml-generator/releases

## Commands

The tool provides the following commands:

- `library generate` generates a PlantUML library based on a provided manifest
- `library schema` Generate the JSON Schema of the library manifest
- `diagram generate` generates `.puml` discovered recursively in the file system
- `workspace init` generate a fresh workspace, i.e. a `.pgen-workspace.yaml` file
- `workspace install` install an artifact in the workspace

## Release

- https://lib.rs/crates/convco
- https://lib.rs/crates/cargo-release

```shell
cargo install convco
cargo install cargo-release
```
