pub const TEMPLATE: &str = r##"@startuml
{%- block header %}{% endblock header %}

{%- block include %}
' configures the library
!global $INCLUSION_MODE="local"
!global $LIB_BASE_LOCATION="{{ data.path_to_base }}"
{% endblock include -%}

{%- block loader %}
' loads the library's bootstrap
!include $LIB_BASE_LOCATION/bootstrap.puml

' loads the package bootstrap
include('{{ data.package_urn }}/bootstrap')
{% endblock loader -%}

{%- block content %}
{% endblock content -%}
{%- block footer %}{% endblock footer %}
@enduml"##;
