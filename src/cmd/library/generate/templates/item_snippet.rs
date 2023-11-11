#[allow(clippy::needless_raw_string_hashes)]
pub const TEMPLATE: &str = r##"@startuml
{%- block header %}{% endblock header %}
{%- block include_mode %}
' configures the library{% if data.snippet_mode == "Remote" %}
!global $LIB_BASE_LOCATION="{{ data.remote_url }}"{% elif data.snippet_mode == "Local" %}
!global $INCLUSION_MODE="local"
!global $LIB_BASE_LOCATION="{{ data.path_to_base }}"{% endif %}
{% endblock include_mode -%}

{%- block loader %}
' loads the library's bootstrap
!include $LIB_BASE_LOCATION/bootstrap.puml

' loads the package bootstrap
include('{{ data.package_urn }}/bootstrap')

' loads the Item which embeds the element {{ data.procedure_name }}
include('{{ data.item_urn }}')
{% endblock loader -%}

{%- block procedures %}
' renders the element
{%- if data.element_shape == "Icon" %}
{{ data.procedure_name }}('{{ data.variable_name }}', '{{ data.primary_label }}', '{{ data.technical_label | default(value="an optional tech label") }}', '{{ data.description_label | default(value="an optional description") }}')
{% elif data.element_shape == "IconCard" %}
{{ data.procedure_name }}('{{ data.variable_name }}', '{{ data.primary_label }}', '{{ data.description_label | default(value="an optional description") }}')
{% elif data.element_shape == "IconGroup" %}
{{ data.procedure_name }}('{{ data.variable_name }}', '{{ data.primary_label }}', '{{ data.technical_label | default(value="an optional tech label") }}') {
    note as note
        the content of the group
    end note
}
{% elif data.element_shape == "Group" %}
{{ data.procedure_name }}('{{ data.variable_name }}', '{{ data.primary_label }}', '{{ data.technical_label | default(value="an optional tech label") }}') {
    note as note
        the content of the group
    end note
}
{% endif -%}
{% endblock procedures -%}
{% block footer %}{% endblock footer -%}
@enduml"##;
