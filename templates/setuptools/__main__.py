{{ name | comment_license("#") }}
{% for line in license.text %}
{{ line | comment_license("#") }}
{% endfor %}

def main() -> None:
    """Main function."""


if __name__ == "__main__":
    main()
