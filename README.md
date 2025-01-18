# cs2-dumper
Aims to simplify transitions between CS2 updates in static analysis.

- Modules specified within the auto-generated `config.json` are dumped to `/output/modules/{module}/{module}_DD_MM_YYYY.{extension}`. The time either being from the PE header's timestamp on Windows or the current day on Linux.
