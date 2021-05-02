pub const TEMPLATE: &str = r##"@startuml
{% block header %}{% endblock header %}
{% block content %}{% endblock content %}
{% block footer %}{% endblock footer %}
@enduml"##;
