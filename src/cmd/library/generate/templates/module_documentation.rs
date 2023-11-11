#[allow(clippy::needless_raw_string_hashes)]
pub const TEMPLATE: &str = r##"# {{ data.module_name }}
{%- block header %}{% endblock header %}

{% set nbr_items = data.items_with_family | length + data.items_without_family | length -%}
The module contains {{ nbr_items }} items.

{% set families = data.items_with_family | map(attribute="family") | unique | sort -%}
{% for family in families -%}
- [{{ family }}](#family-{{ family | lower }})
{% endfor %}

{% if data.items_without_family | length > 0 -%}
| |Name|
|:---:|---|
{% for item in data.items_without_family | sort(attribute="item_urn") -%}
| ![illustration of {{item.item_urn }}]({{ data.path_to_base }}/{{ item.illustration }}) | [{{ item.item_urn }}]({{ data.path_to_base }}/{{ item.item_urn }}.md) |
{% endfor %}
{% endif -%}

{% set items_by_families = data.items_with_family | group_by(attribute="family") -%}
{% for family in families -%}
<span id="family-{{ family | lower }}"></span>
## {{ family }}
| |Name|
|:---:|---|
{% for item in items_by_families[family] | sort(attribute="item_urn") -%}
| ![illustration of {{item.item_urn }}]({{ data.path_to_base }}/{{ item.illustration }}) | [{{ item.item_urn }}]({{ data.path_to_base }}/{{ item.item_urn }}.md) |
{% endfor %}
{% endfor %}

{% block footer %}{% endblock footer -%}"##;
