pub const TEMPLATE: &str = r##"# {{ data.item_name }}
{%- block header %}{% endblock header %}

{% block content %}
```text
{{ data.item_urn }}
```

```text
include('{{ data.item_urn }}')
```
{% endblock content %}

{% block objects %}
{% if data.objects | length > 0 -%}
{% for object in data.objects %}| {{ object.name }} {% endfor %}|
{% for object in data.objects %}| :---: {% endfor %}|
{% for object in data.objects %}| ![illustration for {{ object.name }}]({{ data.path_to_base }}/{{ object.illustration_path }}) {% endfor %}|
{% endif -%}
{% endblock objects %}

{% set icons = data.objects | filter(attribute="type", value="Icon") -%}
{% block sprites %}
{% if icons | length > 0 -%}
## Sprites
The item provides the following stipes
{% for icon in icons %}
- `${{ icon.name }}Xs`
- `${{ icon.name }}Sm`
- `${{ icon.name }}Md`
- `${{ icon.name }}Lg`
{% endfor %}
{% endif -%}
{% endblock sprites %}

{% set elements = data.objects | filter(attribute="type", value="Element") -%}
{% block elements %}
{% if elements | length > 0 -%}
{% for element in elements %}
## {{ element.name }}

### Load remotely
```plantuml
{{ read_file_content(path=element.full_snippet_remote_path) }}
```

### Load locally
```plantuml
{{ read_file_content(path=element.full_snippet_local_path) }}
```
{% endfor %}
{% endif -%}
{% endblock elements -%}
{% block footer %}{% endblock footer -%}"##;
