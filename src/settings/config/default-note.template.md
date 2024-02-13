---
title: {{note.title}}
{%- if note.tags %}
tags:
{%- for tag in note.tags %}
    - {{ tag }}
{%- endfor -%}
{%- if note.url -%}
url: {{ note.url }}
{%- endif %}
{%- endif %}
---
