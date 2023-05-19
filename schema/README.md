# iamthat json schemas

This directory contains [JSON Schema](https://json-schema.org/) files for
files produced and consumed by iamthat.

`.vscode/settings.json` shows how to configure Code to use them.

These schemas currently generate some spurious warnings due to bugs or
limitations in the `schemars` library used to generate them, specifically
<https://github.com/GREsau/schemars/issues/164> about flattened enums.
