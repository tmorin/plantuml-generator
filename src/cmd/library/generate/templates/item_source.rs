#[allow(clippy::needless_raw_string_hashes)]
pub const TEMPLATE: &str = r##"{% block header -%}
' definition of the Item {{ data.item_urn }}
{% endblock header -%}

{%- block sprites %}
{%- for sprite in sprites | default(value=[]) %}
{{ sprite }}
{% endfor -%}
{% endblock sprites -%}

{%- block elements %}
{%- for element in data.elements %}
{%- if element.type == "Icon" %}
!procedure {{ element.procedure_name }}($id, $name="", $tech="", $desc="")
  IconElement($id, '{{ element.stereotype_name }}', '{{ element.icon_urn }}', $name, $tech, $desc)
!endprocedure
{%- elif element.type == "IconCard" %}
!procedure {{ element.procedure_name }}($id, $funcName="", $content="")
  IconCardElement($id, '{{ element.stereotype_name }}', '<${{ element.sprite_name }}>', '{{ element.family_name }}', $funcName, $content)
!endprocedure
{%- elif element.type == "IconGroup" %}
!procedure {{ element.procedure_name }}($id, $name='{{ element.default_label }}', $tech='')
  IconGroupElement($id, '{{ element.stereotype_name }}', '<${{ element.sprite_name }}>', $name, $tech)
!endprocedure
{%- elif element.type == "Group" %}
!procedure {{ element.procedure_name }}($id, $name='{{ element.default_label }}', $tech='')
  GroupElement($id, '{{ element.stereotype_name }}', $name, $tech)
!endprocedure
{%- endif %}
{% endfor -%}
{% endblock elements -%}

{% block footer %}{% endblock footer -%}"##;
