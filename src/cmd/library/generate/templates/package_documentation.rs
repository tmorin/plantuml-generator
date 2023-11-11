#[allow(clippy::needless_raw_string_hashes)]
pub const TEMPLATE: &str = r##"# {{ data.package_name }}
{% block header %}{% endblock header -%}

{% block bootstrap %}
## Usage

### Bootstrap

The bootstrap may provide PlantUML artifacts like constants, procedures or style statements.

```plantuml
' loads the {{ data.package_name }} bootstrap
include('{{ data.package_urn }}/bootstrap')
```

{% if data.is_embedded_enabled == true -%}
### Full inclusion

An additional include can be used to load all items in one shot.

 ```plantuml
' loads the bootstrap of `{{ data.package_urn }}` and all related items
include('{{ data.package_urn }}/full')
```

### Single inclusion

Finally, another include can be used to load the library's bootstrap, the package's bootstrap and all items' resources in one `!include` statement.

Include remotely the resources:
```plantuml
' loads the library, the bootstrap of `{{ data.package_urn }}` and all related items
!include {{ data.remote_url }}/{{ data.package_urn }}/single.puml
```

Include locally the resources:
```plantuml
' configures the library
!global $INCLUSION_MODE="local"
' loads the library, the bootstrap of `{{ data.package_urn }}` and all related items
!include <the relative path to the /distribution directory>/{{ data.package_urn }}/single.puml
```
{% endif %}
{% endblock bootstrap %}

{% block modules %}
# Modules

The package provides {{ data.modules | length }} modules.
{% for module in data.modules %}
- [{{ module.module_urn }}]({{ data.path_to_base }}/{{ module.module_urn }}/README.md) with {{ module.nbr_items }} items{% endfor %}
{% endblock modules %}

{% block examples %}
# Examples

The package provides {{ data.examples | length }} examples.
{% for example in data.examples %}
## {{ example.name }}

![{{ example.name }}]({{ data.path_to_base }}/{{ example.destination }})<br>
[The source file.]({{ data.path_to_base }}/{{ example.source }})
{% endfor %}
{% endblock examples %}

{% block footer %}{% endblock footer -%}"##;
