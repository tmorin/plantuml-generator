# Changelog

## [Unreleased](https://github.com/tmorin/plantuml-generator/compare/v1.14.0...15a38728e5e8785f725314056f983155c4a1f4ad) (2024-09-30)

### Fixes

* **library:** icons are not loaded when using remote loader
([15a3872](https://github.com/tmorin/plantuml-generator/commit/15a38728e5e8785f725314056f983155c4a1f4ad))

## [v1.14.0](https://github.com/tmorin/plantuml-generator/compare/v1.13.0...v1.14.0) (2024-09-21)

### Features

* add the discovery patterns for PlantUML files as a configuration entry
([dbdbc6a](https://github.com/tmorin/plantuml-generator/commit/dbdbc6a69b61df1f4f371ee51a5e496c506b39f6)),
closes [#15](https://github.com/tmorin/plantuml-generator/issues/15)
* add an option to append additional PlantUML arguments when generating
diagrams
([9cc0a2f](https://github.com/tmorin/plantuml-generator/commit/9cc0a2f4597e7da118b71561b41d0a47987e0a74)),
closes [#11](https://github.com/tmorin/plantuml-generator/issues/11)
* upgrade PlantUML to 1.2024.7 switching to the GitHub repository
([65f49b8](https://github.com/tmorin/plantuml-generator/commit/65f49b8e38e31df3422bf9d1a1a65928b214d533)),
closes [#12](https://github.com/tmorin/plantuml-generator/issues/12)

### Fixes

* deactivate some clippy issues
([d55c14a](https://github.com/tmorin/plantuml-generator/commit/d55c14a3731e2f55f9cbfc4f1a10c49af3c68a75))
* complete clippy issues
([dba3ce0](https://github.com/tmorin/plantuml-generator/commit/dba3ce0fec1f73211309ed292cd7245815222b51))

## [v1.13.0](https://github.com/tmorin/plantuml-generator/compare/v1.12.1...v1.13.0) (2023-04-25)

### Features

* **workspace:** install artifacts from the workspace manifest
([95a4422](https://github.com/tmorin/plantuml-generator/commit/95a442237e2ea39baa284f1ce7f7d3b9e0eaaf22))
* **workspace:** initialize a workspace
([5e93ecb](https://github.com/tmorin/plantuml-generator/commit/5e93ecb88015075d87acb1244391fb27ff73385c))

### Fixes

* improve the README.md with the full list of available commands
([a444424](https://github.com/tmorin/plantuml-generator/commit/a4444249af24fb0d044a3580fe036c1f2fdf8f22))
* **workspace:** prevent initialization of an initialized workspace
([5808f76](https://github.com/tmorin/plantuml-generator/commit/5808f76e5e1bcc03fe3a7be8b3f36ee3faf5522e))

### [v1.12.1](https://github.com/tmorin/plantuml-generator/compare/v1.12.0...v1.12.1) (2023-04-21)

#### Fixes

* remove the display of the executed command during the execution of
`scripts/install_pgen.sh`
([5d29de4](https://github.com/tmorin/plantuml-generator/commit/5d29de41640dace78137a0a1842154d04ae0bf75))

## [v1.12.0](https://github.com/tmorin/plantuml-generator/compare/v1.11.0...v1.12.0) (2023-04-20)

### Features

* add support the targets `powerpc64le-unknown-linux-gnu` and
`s390x-unknown-linux-gnu`
([80aaf8c](https://github.com/tmorin/plantuml-generator/commit/80aaf8c0a7ed785fc87c2df264d1b0d388dde81d))

## [v1.11.0](https://github.com/tmorin/plantuml-generator/compare/v1.10.0...v1.11.0) (2023-04-15)

### Features

* **docker:** make the Docker Image rootless
([46fd232](https://github.com/tmorin/plantuml-generator/commit/46fd2322751727c1358b80337400f9d774828917))

## [v1.10.0](https://github.com/tmorin/plantuml-generator/compare/v1.9.0...v1.10.0) (2023-04-12)

### Features

* add a command to generate the JSON Schema of the library manifest
([5f88b6f](https://github.com/tmorin/plantuml-generator/commit/5f88b6f2f9a30006d057323fd910ed92503030eb))

## [v1.9.0](https://github.com/tmorin/plantuml-generator/compare/v1.8.2...v1.9.0) (2023-03-21)

### Features

* **release:** include only the binary in tarballs
([47e989c](https://github.com/tmorin/plantuml-generator/commit/47e989ce7818cb44340dbbacb562bd50f1e5f01a))
* **docker:** upgrade the jdk version
([7a77180](https://github.com/tmorin/plantuml-generator/commit/7a77180ce56e62abbb7e9f6380e2911c6759e1e6))

### Fixes

* **completion:** fix the generation of completion for shells
([7d2472e](https://github.com/tmorin/plantuml-generator/commit/7d2472e2ad685e902c31c3c878d8cc4e19d9f7ba))

### [v1.8.2](https://github.com/tmorin/plantuml-generator/compare/v1.8.1...v1.8.2) (2023-03-18)

#### Fixes

* the names of the sprites were wrong in the documentation
([71e0dfe](https://github.com/tmorin/plantuml-generator/commit/71e0dfe54cac5523d81c5cba01b57ff83620f54a))

### [v1.8.1](https://github.com/tmorin/plantuml-generator/compare/v1.8.0...v1.8.1) (2023-03-18)

#### Fixes

* the sprite word was badly typed
([45fc689](https://github.com/tmorin/plantuml-generator/commit/45fc689d1bfacba3f5e5b055dd417a0357ff29ca))

## [v1.8.0](https://github.com/tmorin/plantuml-generator/compare/v1.7.0...v1.8.0) (2023-03-18)

### Features

* add the reference of the sprites in the documentation
([6fd37b6](https://github.com/tmorin/plantuml-generator/commit/6fd37b6faa75cbab0f911d914ad893cb50d23742))

## [v1.7.0](https://github.com/tmorin/plantuml-generator/compare/v1.6.0...v1.7.0) (2022-08-15)

### Features

* add a SUMMARY.md for the library
([29b46f1](https://github.com/tmorin/plantuml-generator/commit/29b46f1cf7f116f3e1ca234a49ec262d46a12423))

## [v1.6.0](https://github.com/tmorin/plantuml-generator/compare/v1.5.1...v1.6.0) (2022-08-15)

### Features

* add a new loader to load the library bootstrap, the package bootstrap and
all package's items in one `!include` statement
([d32ea1e](https://github.com/tmorin/plantuml-generator/commit/d32ea1ed20b1344313d383185b3a274af780198f))

### Fixes

* remove useless `@startuml` and `@enduml` directives
([f8e554b](https://github.com/tmorin/plantuml-generator/commit/f8e554bcfbbee18a7345871cdc8c1b9d6c3c8cd3))

### [v1.5.1](https://github.com/tmorin/plantuml-generator/compare/v1.5.0...v1.5.1) (2022-08-15)

#### Fixes

* add the documentation about the full loader
([83a97b4](https://github.com/tmorin/plantuml-generator/commit/83a97b41571261c79dcf0ffd26d557d9ac83decc))

## [v1.5.0](https://github.com/tmorin/plantuml-generator/compare/v1.4.0...v1.5.0) (2022-08-13)

### Features

* add a "full" template for package in order to get every thing from a single
a file
([63ae6ec](https://github.com/tmorin/plantuml-generator/commit/63ae6ec7d28cdc08ab37fdfc94ef13de6c7398e0))

## [v1.4.0](https://github.com/tmorin/plantuml-generator/compare/v1.3.0...v1.4.0) (2022-08-02)

### Features

* added optional description to Icon Items (#1)
([bc9f6d9](https://github.com/tmorin/plantuml-generator/commit/bc9f6d9ebc425fa800fc75a40ec3aa23ec708ddf)),
closes [#1](https://github.com/tmorin/plantuml-generator/issues/1)

## [v1.3.0](https://github.com/tmorin/plantuml-generator/compare/v1.2.0...v1.3.0) (2022-06-07)

### Features

* add the completion sub-command
([f0a9549](https://github.com/tmorin/plantuml-generator/commit/f0a9549ac8b372fe5ac9a11657b44ef6b1cfa01a))

## [v1.2.0](https://github.com/tmorin/plantuml-generator/compare/v1.1.0...v1.2.0) (2022-05-04)

### Features

* upgrade the PlantUML version to 1.2022.4
([4f8bf4e](https://github.com/tmorin/plantuml-generator/commit/4f8bf4eeae1f374661d78608abcc85928fb5496e))

## [v1.1.0](https://github.com/tmorin/plantuml-generator/compare/v1.0.1...v1.1.0) (2021-12-03)

### Features

* upgrade the PlantUML version to 1.2021.15
([7985feb](https://github.com/tmorin/plantuml-generator/commit/7985feb90c963fa6d11dd6b688fecf9a10db97b8))

### [v1.0.1](https://github.com/tmorin/plantuml-generator/compare/v1.0.0...v1.0.1) (2021-09-22)

#### Fixes

* sprites for card elements are too small
([072c571](https://github.com/tmorin/plantuml-generator/commit/072c57155bd77443056bb523a5da32f37ec46b66))

## [v1.0.0](https://github.com/tmorin/plantuml-generator/compare/v0.1.0...v1.0.0) (2021-09-14)

### ⚠ BREAKING CHANGE

* in templates, `element.type` are no more suffix with `Element`, i.e. `{% if element.type == "CustomElement" -%}` becomes `{% if element.type == "Custom" -%}`


### Fixes

* icons with transparent background could not be properly generated to sprites
([0323dad](https://github.com/tmorin/plantuml-generator/commit/0323dadc7f7bee26a971abd355b32aada6398402))

## v0.1.0 (2021-07-14)

### Features

* generate a library and diagrams
([c6dc1c0](https://github.com/tmorin/plantuml-generator/commit/c6dc1c04192bd521036806b4f894c22420ff3bc9))
