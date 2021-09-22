# Changelog

## [1.0.1](https://github.com/tmorin/plantuml-generator/compare/v1.0.0...v1.0.1) (2021-09-20)

### Fixes

* sprites for card elements are too small 072c571


## [v1.0.0](https://github.com/tmorin/plantuml-generator/compare/v0.1.0...v1.0.0) (2021-09-14)

### ⚠ BREAKING CHANGE

* in templates, `element.type` are no more suffix with `Element`, i.e. `{% if element.type == "CustomElement" -%}` becomes `{% if element.type == "Custom" -%}`

### Fixes

* icons with transparent background could not be properly generated to sprites 0323dad


## v0.1.0 (2021-07-14)

### Features

* generate a library and diagrams c6dc1c0


