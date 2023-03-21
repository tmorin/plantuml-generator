# Changelog

## [Unreleased](https://github.com/tmorin/plantuml-generator/compare/v1.8.2...HEAD) (2023-03-21)

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
([bc9f6d9](https://github.com/tmorin/plantuml-generator/commit/bc9f6d9ebc425fa800fc75a40ec3aa23ec708ddf))

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
