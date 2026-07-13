<p align="center">
  <img src="https://meander.sbs/cdn/demo.gif" width="720" alt="Demo"/>
</p>

<p align="center">
  <a href="https://pub.dev"><img src="https://img.shields.io/badge/pub-v0.1.0-blue" alt="pub version"></a>
  <a href="https://github.com/IILLUMINATION/flutter_3d_engine/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-green" alt="license"></a>
  <a href="https://dart.dev"><img src="https://img.shields.io/badge/Dart-3.12+-blue" alt="dart"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-stable-orange" alt="rust"></a>
  <a href="https://wgpu.rs"><img src="https://img.shields.io/badge/wgpu-24-red" alt="wgpu"></a>
</p>

---

# Flutter 3D Engine

GPU-accelerated 3D engine for Flutter. Renders through **Rust + wgpu** directly into an **irondash** native texture — zero pixel copies into Dart.

See the [demo app](example/) — a Minecraft-style FPS sandbox.

## Stack

| Layer | Tech |
|---|---|
| Rendering | **wgpu** (Vulkan / Metal / DX12), WGSL shaders |
| Physics | **Rapier3D** — rigid bodies, collisions, raycasting |
| Texture bridge | **irondash** — GPU buffer visible to Flutter, no copies |
| FFI | **flutter_rust_bridge** 2.12 |

## Quick start

```bash
git clone https://github.com/IILLUMINATION/flutter_3d_engine
cd flutter_3d_engine
flutter_rust_bridge_codegen generate
cd example && flutter run -d linux
```

## Minimal usage

```dart
final scene = await createScene();
final handle = await EngineContext.instance.getEngineHandle();
final id = await initNativeTexture(scene: scene, engineHandle: handle, width: 1280, height: 720);

final ctrl = Rust3DController.wrap(scene, textureId: id);
ctrl.initDefaultCamera();

void onTick(double dt) {
  ctrl.physicsStep(dt);
  renderNativeFrame(scene: scene, width: 1280, height: 720);
}

ctrl.spawnCubeInFront(r: 1.0, g: 0.27, b: 0.0);
```

## API

```dart
// Camera
controller.orbitCamera(dx, dy);
controller.movePlayer(dx, dz);
controller.jumpPlayer();

// World editing
controller.spawnCubeInFront(r: 1.0, g: 0.27, b: 0.0);
controller.destroyLookedBlock();

// Simulation
controller.physicsStep(dt);

// GPU texture
final id = await initNativeTexture(scene, engineHandle, width, height);
renderNativeFrame(scene, width, height);
```

## How it works

1. An **irondash texture** is created — Flutter gets its ID, shows it with `Texture(textureId: id)`.
2. Rust spins up a **wgpu device**, renders the scene straight into that same texture. No pixels travel through Dart.
3. **Rapier3D** steps the physics each frame. The camera rides on the player's rigid body.
4. **Raycasting** finds the block under the crosshair. Surface normals snap placement to the grid.

## Status

- Stable: rendering, physics, camera, block spawn/destroy, 31 unit tests
- Planned: pointer lock, textures, chunk generation, model loading

## License

MIT — use it, fork it, build on it. Keep the copyright notice and attribution.

By [IILLUMINATION](https://github.com/IILLUMINATION).
