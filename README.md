# source2-dumper
Aims to simplify transitions between Source 2 game updates in static analysis.

## Building & running
```sh
cargo build
cargo run -- --process cs2.exe
```

## Roadmap
- [x] Dump modules (`/output/modules/{module}/{module}_DD_MM_YYYY.{extension}`)
- [ ] Dump schema system
- [ ] Dump interfaces
- [x] Configuration (`config.json`)
- [ ] IDA script
- [ ] Binja script
