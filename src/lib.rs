mod compiler;
mod instruction;
mod template;
mod error;

/* 
TODO:
- Implement parsing using Jinja2-like syntax
    - If/else {% if foo %}{% else %}{% endif %}
    - For {% for foo in bar.baz %}{% endfor %}
    - Comments {# Foo bar baz #}
    - Whitespace stripping {{- foo.bar -}}
    - Call {% call macro_name %}
    - Indexing {{ foo.bar[5] }} {{ foo.bar[index] }}
- Implement parse error handling by calculating the line/column when an error occurs
- Implement evaluation
*/