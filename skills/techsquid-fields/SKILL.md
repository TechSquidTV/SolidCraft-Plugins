---
name: techsquid-fields
description: Use when adding, renaming or removing a dialog field, editing a plugin.json manifest, writing error messages, or changing which SolidCraft host commands a TechSquid plugin runs.
---

# Fields, manifests and errors

## plugin.json

- `runtime.entries[]` is one dialog each: `name` (`make-<thing>`), `effect`, `slots` (the menu label) and `input_schema`. Each field has `title`, `description`, `type`, and a `default`, `enum`, `minimum`/`maximum` and `unit` where they apply. Write descriptions as plain instructions to the user.
- `runtime.commands` lists every host command the plugin runs; the host refuses the rest. A plugin's unit tests compare the manifest with the commands the code sends (its own list plus `component::commands()`), so add a new command to the manifest and to that test.
- `runtime.limits` holds the fuel, memory and deadline. Raise them only with measurements.
- `version` is the workspace version. Catalog-owned `enum` choices are generated: run `cargo run -p catalog-tool -- --write` and don't hand-edit them.

## Shared field names

Use the shared names wherever they fit: `size`, `custom_size`, `create`, `sketch`, `sketch_point`, `thread_modeled`. `create` takes the part, the cut, or both, through `plugin_kit::create`. `sketch` and `sketch_point` come through `place::parse_input`.

## Parsing

Start `parse` with `input::workflow_fields(v, include_str!("../plugin.json"), "<entry>")`, which refuses keys the packaged manifest doesn't list. Then read each field with `input::choice`, `input::flag`, `input::length_or`, `input::optional_length` and friends, and return `FieldError` for bad values.

## Errors

- `<field_key>: <reason>` is shown against that dialog field. `FieldError::new("size", "choose a listed wheel")` produces it.
- `<command.id>: <reason>` is reported as a failed host command.
- Anything else is a trap. Write reasons as what the user should do: `sketch_point: choose at most 32 points`.

## Breaking field changes

A renamed or retired key is refused, never aliased or given a fallback value. List the old key and its replacement under Older field names in the plugin's REFERENCE.md, mark the commit `!`, and add the migration to the release notes.
