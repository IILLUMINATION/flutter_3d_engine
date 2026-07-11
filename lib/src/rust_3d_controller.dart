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

  Future<void> setNodeTransform({
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
    return ffi.updateNodeTransform(
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

  Future<void> setCamera({
    double px = 0,
    double py = 2,
    double pz = 5,
    double tx = 0,
    double ty = 0,
    double tz = 0,
  }) {
    return ffi.updateCamera(
      scene: _scene,
      px: px,
      py: py,
      pz: pz,
      tx: tx,
      ty: ty,
      tz: tz,
    );
  }

  Future<void> setFov(double fov) => ffi.setCameraFov(scene: _scene, fov: fov);
}
