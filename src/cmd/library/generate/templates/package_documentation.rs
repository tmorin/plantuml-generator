pub const TEMPLATE: &str = r##"# {{ data.package_name }}
{% block header %}{% endblock header -%}

{% block bootstrap %}
## Bootstrap

The bootstrap may provide PlantUML artifacts like constants, procedures or style statements.

```plantuml
' loads the {{ data.package_name }} bootstrap
include('{{ data.package_urn }}/bootstrap')
```

An additional include can be used to load all items in one shot.
 ```plantuml
' loads the {{ data.package_name }} bootstrap
include('{{ data.package_urn }}/bootstrap')
' loads all items of {{ data.package_name }}
include('{{ data.package_urn }}/full')
```

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
