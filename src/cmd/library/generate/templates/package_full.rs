pub const TEMPLATE: &str = r##"@startuml
{% block header %}{% endblock header %}
{% block content %}
include('{{ data.package_urn }}/bootstrap')
{% for item in data.items -%}
include('{{ item.item_urn }}')
{% endfor %}
{% endblock content %}
{% block footer %}{% endblock footer %}
@enduml"##;
