pub const TEMPLATE: &str = r##"
{%- block header %}{% endblock header -%}
{%- block content -%}
# {{ data.library_name }}

[Presentation](README.md)

{% for package in data.packages %}
# {{ package.package_urn }}
- [Presentation]({{ package.package_urn }}/README.md)
{%- for module in package.modules %}
- [{{ module.module_urn }}]({{ module.module_urn }}/README.md){% for item in module.items %}
    - [{{ item.item_urn }}]({{ item.item_urn }}.md){% endfor %}{% endfor %}
{% endfor %}
{% endblock content -%}
{% block footer %}{% endblock footer -%}"##;
