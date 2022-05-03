{{ name | comment_license("#") }}
{% for line in license.text %}
{{ line | comment_license("#") }}
{% endfor %}
"""Init file."""
