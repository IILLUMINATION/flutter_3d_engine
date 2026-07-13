import 'package:flutter_rust_3d/src/rust/api/simple.dart' as ffi;
import 'package:flutter_rust_3d/src/rust/core/scene.dart';

class Rust3DController {
  final Scene3D _scene;
  int _textureId = 0;

  Rust3DController._(this._scene, this._textureId);
  static Rust3DController wrap(Scene3D scene, {int textureId = 0}) =>
      Rust3DController._(scene, textureId);

  Scene3D get scene => _scene;
  int get textureId => _textureId;

  Future<BigInt> addCube({
    double x = 0, double y = 0, double z = 0,
    double r = 1, double g = 0, double b = 0,
  }) => ffi.addCube(scene: _scene, x: x, y: y, z: z, r: r, g: g, b: b);

  void physicsStep(double dt) => ffi.physicsStep(scene: _scene, dt: dt);

  void initDefaultCamera() => ffi.initDefaultCamera(scene: _scene);

  void orbitCamera(double dx, double dy) => ffi.orbitCamera(scene: _scene, dx: dx, dy: dy);

  void zoomCamera(double delta) => ffi.zoomCamera(scene: _scene, delta: delta);

  void movePlayer({double dx = 0, double dz = 0}) =>
      ffi.movePlayer(scene: _scene, dx: dx, dz: dz);

  void jumpPlayer() => ffi.jumpPlayer(scene: _scene);

  BigInt spawnCubeInFront({
    double r = 1.0,
    double g = 0.27,
    double b = 0.0,
  }) => ffi.spawnCubeInFront(scene: _scene, r: r, g: g, b: b);
}
