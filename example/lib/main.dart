import 'package:flutter/material.dart';
import 'package:flutter_rust_3d/flutter_rust_3d.dart';
import 'package:flutter_rust_3d/src/rust/frb_generated.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const AppRoot());
}

class AppRoot extends StatelessWidget {
  const AppRoot({super.key});

  @override
  Widget build(BuildContext context) {
    return const MaterialApp(
      home: DemoScreen(),
    );
  }
}

class DemoScreen extends StatefulWidget {
  const DemoScreen({super.key});

  @override
  State<DemoScreen> createState() => _DemoScreenState();
}

class _DemoScreenState extends State<DemoScreen> {
  BigInt? _cubeId;
  double _rotationY = 0;

  void _onCreated(Rust3DController controller) {
    controller.setCamera(px: 0, py: 2, pz: 5, tx: 0, ty: 0, tz: 0);
    controller.addCube(x: 0, y: 0, z: 0, r: 1.0, g: 0.4, b: 0.2).then((id) {
      setState(() => _cubeId = id);
    });
  }

  void _onTick(Rust3DController controller, double elapsedSec, double deltaSec) {
    final id = _cubeId;
    if (id == null) return;
    _rotationY += deltaSec * 1.0;
    controller.setNodeTransform(
      nodeId: id,
      ry: _rotationY,
    );
  }

  @override
  Widget build(BuildContext context) {
    return Rust3DCanvas(
      width: 1280,
      height: 720,
      onCreated: _onCreated,
      onTick: _onTick,
    );
  }
}
