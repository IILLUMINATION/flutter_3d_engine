# Flutter 3D Engine

3D-движок на стеке Flutter + Rust (flutter_rust_bridge v2). Чистое математическое ядро сцены на Rust, GPU-рендеринг через wgpu (Vulkan), headless-отрисовка в текстуру с передачей пикселей во Flutter.

## Архитектура

- `rust/src/core/` — изолированное 3D-ядро (математика, сцена, рендерер, FrameSink)
- `rust/src/api/` — FFI-мост во Flutter
- `rust/src/shader.wgsl` — WGSL шейдеры
- `lib/main.dart` — Flutter UI: Ticker-игровой цикл + CustomPainter

## Сборка

```bash
# Генерация моста
flutter_rust_bridge_codegen generate

# Rust-тесты
cd rust && cargo test

# Запуск
flutter run -d linux
```
