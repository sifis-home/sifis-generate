{{ name | comment_license("#") }}
{% for line in license.text %}
{{ line | comment_license("#") }}
{% endfor %}

def test_sum():
    """Simple test."""
    assert 2 + 2 == 4

