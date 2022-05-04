# Changelog

## [Unreleased](https://github.com/tmorin/plantuml-generator/compare/v1.1.0...HEAD) (2022-05-04)

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

* in templates, `element.type` are no more suffix with `Element`, i.e. `{% ifelement.type == "CustomElement" -%}` becomes `{% if element.type == "Custom"-%}`

### Fixes

* icons with transparent background could not be properly generated to sprites
  ([0323dad](https://github.com/tmorin/plantuml-generator/commit/0323dadc7f7bee26a971abd355b32aada6398402))

## v0.1.0 (2021-07-14)

### Features

* generate a library and diagrams
  ([c6dc1c0](https://github.com/tmorin/plantuml-generator/commit/c6dc1c04192bd521036806b4f894c22420ff3bc9))
