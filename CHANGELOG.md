# Changelog

## [Unreleased](https://github.com/tmorin/plantuml-generator/compare/v0.1.0...HEAD) (2021-09-14)

### ⚠ BREAKING CHANGE

* in templates, `element.type` are no more suffix with `Element`, i.e. `{% if element.type == "CustomElement" -%}` becomes `{% if element.type == "Custom" -%}`

### Fixes

* icons with transparent background could not be properly generated to sprites 0323dad


## v0.1.0 (2021-07-14)

### Features

* generate a library and diagrams c6dc1c0


