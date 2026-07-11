import 'package:flutter_rust_3d/src/rust/api/simple.dart' as ffi;
import 'package:flutter_rust_3d/src/rust/core/scene.dart';

class Rust3DController {
  final Scene3D _scene;

  Rust3DController._(this._scene);
  static Rust3DController wrap(Scene3D scene) => Rust3DController._(scene);

  Scene3D get scene => _scene;

  Future<BigInt> addCube({
    double x = 0,
    double y = 0,
    double z = 0,
    double r = 1,
    double g = 0,
    double b = 0,
  }) {
    return ffi.addCube(
      scene: _scene,
      x: x,
      y: y,
      z: z,
      r: r,
      g: g,
      b: b,
    );
  }

  void setNodeTransform({
    required BigInt nodeId,
    double px = 0,
    double py = 0,
    double pz = 0,
    double rx = 0,
    double ry = 0,
    double rz = 0,
    double sx = 1,
    double sy = 1,
    double sz = 1,
  }) {
    ffi.updateNodeTransform(
      scene: _scene,
      nodeId: nodeId,
      px: px,
      py: py,
      pz: pz,
      rx: rx,
      ry: ry,
      rz: rz,
      sx: sx,
      sy: sy,
      sz: sz,
    );
  }

  void setCamera({
    double px = 0,
    double py = 2,
    double pz = 5,
    double tx = 0,
    double ty = 0,
    double tz = 0,
  }) {
    ffi.updateCamera(
      scene: _scene,
      px: px,
      py: py,
      pz: pz,
      tx: tx,
      ty: ty,
      tz: tz,
    );
  }

  void physicsStep(double dt) => ffi.physicsStep(scene: _scene, dt: dt);

  bool handlePointerDown(double sx, double sy, double sw, double sh) =>
      ffi.handlePointerDown(scene: _scene, screenX: sx, screenY: sy, screenWidth: sw, screenHeight: sh);

  void handlePointerMove(double sx, double sy, double sw, double sh) =>
      ffi.handlePointerMove(scene: _scene, screenX: sx, screenY: sy, screenWidth: sw, screenHeight: sh);

  void handlePointerUp() => ffi.handlePointerUp(scene: _scene);

  Future<void> setFov(double fov) => ffi.setCameraFov(scene: _scene, fov: fov);
}
