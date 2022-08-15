pub const TEMPLATE: &str = r##"{% block header %}{% endblock header %}
{% block library_bootstrap %}{{ library_bootstrap }}{% endblock library_bootstrap %}
{% block package_bootstrap %}{{ package_bootstrap }}{% endblock package_bootstrap %}
{% block package_items %}{{ package_items }}{% endblock package_items %}
{% block footer %}{% endblock footer %}"##;
