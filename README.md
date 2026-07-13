# Flutter 3D Engine

GPU-ускоренный 3D-движок для Flutter. Рендеринг через Rust + wgpu напрямую в нативную текстуру irondash, без копирования пикселей в Dart.

## Стек

| Слой | Технология |
|---|---|
| Рендеринг | wgpu (Vulkan/Metal/DX12), WGSL-шейдеры |
| Физика | Rapier3D — rigid bodies, коллизии, raycasting |
| Текстура | irondash — GPU-буфер виден Flutter'у без копий |
| Мост Rust↔Dart | flutter_rust_bridge 2.12 |

## Быстрый старт

```bash
git clone https://github.com/IILLUMINATION/flutter_3d_engine
cd flutter_3d_engine
flutter_rust_bridge_codegen generate
cd example && flutter run -d linux
```

## Минимальное использование

```dart
final scene = await createScene();
final handle = await EngineContext.instance.getEngineHandle();
final textureId = await initNativeTexture(scene: scene, engineHandle: handle, width: 1280, height: 720);

final ctrl = Rust3DController.wrap(scene, textureId: textureId);
ctrl.initDefaultCamera();

// Игровой цикл
void onTick(double dt) {
  ctrl.physicsStep(dt);
  renderNativeFrame(scene: scene, width: 1280, height: 720);
}

// Спавн куба
ctrl.spawnCubeInFront(r: 1.0, g: 0.27, b: 0.0);
```

## API

```dart
// Камера
controller.orbitCamera(dx, dy);    // поворот мышью
controller.movePlayer(dx, dz);     // WASD
controller.jumpPlayer();

// Спавн и удаление
controller.spawnCubeInFront();     // перед игроком
controller.destroyLookedBlock();   // блок под прицелом

// Физика
controller.physicsStep(dt);        // шаг симуляции

// Текстура
final id = await initNativeTexture(scene, engineHandle, width, height);
renderNativeFrame(scene, width, height);  // отрисовать кадр
```

## Структура

```
lib/                          # Dart-пакет
├── src/rust_3d_canvas.dart   # Виджет + Ticker
├── src/rust_3d_controller.dart  # Обёртка над FFI
└── src/rust/api/simple.dart  # Dart FFI

rust/src/
├── shader.wgsl               # WGSL (Blinn-Phong, instanced)
├── api/simple.rs             # FFI Rust → Dart
└── core/
    ├── scene.rs              # Scene3D — физика, камера, спавн
    ├── renderer_gpu.rs       # wgpu-рендерер
    ├── renderer.rs           # CPU-wireframe (deprecated)
    ├── present.rs            # IrondashTexturePresenter, FrameSink
    └── math.rs               # Vector3, Transform

example/                      # Демо (FPS-песочница, отдельно)
```

Демо-пример в `example/` — отдельный пакет. Он показывает, как всё собрать вместе (WASD, захват мыши, хотбар, выбор цветов), но не является частью библиотеки.

## Состояние

Стабильно: рендеринг, физика, камера, спавн/удаление, 31 юнит-тест.

В планах: Pointer Lock, текстуры, чанковая генерация, загрузка моделей.

## Лицензия

MIT
