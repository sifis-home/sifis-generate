{{ name | comment_license("#") }}
{% for line in license.text %}
{{ line | comment_license("#") }}
{% endfor %}
"""Setup routine."""
import setuptools

setuptools.setup()
