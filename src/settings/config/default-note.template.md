---
title: {{note.title}}
{%- if note.tags %}
tags:
{%- for tag in note.tags %}
    - {{ tag }}
{%- endfor -%}
{%- endif %}
---
