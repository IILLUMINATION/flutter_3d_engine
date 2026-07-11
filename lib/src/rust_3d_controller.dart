import 'package:flutter_rust_3d/src/rust/api/simple.dart' as ffi;
import 'package:flutter_rust_3d/src/rust/core/scene.dart';

class Rust3DController {
  final Scene3D _scene;

  Rust3DController._(this._scene);
  static Rust3DController wrap(Scene3D scene) => Rust3DController._(scene);

  Scene3D get scene => _scene;

  Future<BigInt> addCube({
    double x = 0, double y = 0, double z = 0,
    double r = 1, double g = 0, double b = 0,
  }) => ffi.addCube(scene: _scene, x: x, y: y, z: z, r: r, g: g, b: b);

  void physicsStep(double dt) => ffi.physicsStep(scene: _scene, dt: dt);

  void initDefaultCamera() => ffi.initDefaultCamera(scene: _scene);

  void orbitCamera(double dx, double dy) => ffi.orbitCamera(scene: _scene, dx: dx, dy: dy);

  void zoomCamera(double delta) => ffi.zoomCamera(scene: _scene, delta: delta);

  bool handlePointerDown(double sx, double sy, double sw, double sh) =>
      ffi.handlePointerDown(scene: _scene, screenX: sx, screenY: sy, screenWidth: sw, screenHeight: sh);

  void handlePointerMove(double sx, double sy, double sw, double sh) =>
      ffi.handlePointerMove(scene: _scene, screenX: sx, screenY: sy, screenWidth: sw, screenHeight: sh);

  void handlePointerUp() => ffi.handlePointerUp(scene: _scene);
}
