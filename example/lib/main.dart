import 'dart:math' as math;

import 'package:flutter/gestures.dart';
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
  Rust3DController? _controller;
  final List<BigInt> _cubeIds = [];

  double _theta = 0.0;
  double _phi = 0.2;
  double _radius = 5.0;

  static const double _panSensitivity = 0.007;
  static const double _zoomStep = 0.005;
  static const double _minRadius = 2.0;
  static const double _maxRadius = 15.0;
  static const double _phiMin = -1.4;
  static const double _phiMax = 1.4;

  void _updateCameraPosition() {
    final x = _radius * math.cos(_phi) * math.sin(_theta);
    final y = _radius * math.sin(_phi);
    final z = _radius * math.cos(_phi) * math.cos(_theta);
    _controller?.setCamera(px: x, py: y, pz: z, tx: 0, ty: 0, tz: 0);
  }

  void _onCreated(Rust3DController controller) {
    _controller = controller;
    _updateCameraPosition();
  }

  // physics step + render — driven by Ticker in Rust3DCanvas
  void _onTick(Rust3DController controller, double elapsedSec, double deltaSec) {
    controller.physicsStep(deltaSec.clamp(0.0, 0.05));
  }

  void _spawnCube() {
    final rng = math.Random();
    final x = (rng.nextDouble() - 0.5) * 2.0;
    final z = (rng.nextDouble() - 0.5) * 2.0;
    _controller?.addCube(x: x, y: 4.0, z: z, r: rng.nextDouble(), g: rng.nextDouble(), b: rng.nextDouble()).then((id) {
      setState(() => _cubeIds.add(id));
    });
  }

  void _onPanStart(DragStartDetails d) {}

  void _onPanUpdate(DragUpdateDetails d) {
    _theta -= d.delta.dx * _panSensitivity;
    _phi   += d.delta.dy * _panSensitivity;
    _phi    = _phi.clamp(_phiMin, _phiMax);
    _updateCameraPosition();
    setState(() {});
  }

  void _onPointerSignal(PointerSignalEvent event) {
    if (event is PointerScrollEvent) {
      _radius -= event.scrollDelta.dy * _zoomStep;
      _radius  = _radius.clamp(_minRadius, _maxRadius);
      _updateCameraPosition();
      setState(() {});
    }
  }

  @override
  Widget build(BuildContext context) {
    return Listener(
      onPointerSignal: _onPointerSignal,
      child: GestureDetector(
        onPanStart: _onPanStart,
        onPanUpdate: _onPanUpdate,
        child: Stack(
          children: [
            Rust3DCanvas(
              width: 1280,
              height: 720,
              onCreated: _onCreated,
              onTick: _onTick,
            ),
            Positioned(
              top: 16,
              right: 16,
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                decoration: BoxDecoration(
                  color: Colors.black54,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    ElevatedButton.icon(
                      onPressed: _spawnCube,
                      icon: const Icon(Icons.add_box, size: 18),
                      label: const Text('Spawn Cube'),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: Colors.deepOrange,
                        foregroundColor: Colors.white,
                        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                        textStyle: const TextStyle(fontSize: 12),
                      ),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      'Cubes: ${_cubeIds.length}',
                      style: const TextStyle(color: Colors.white70, fontSize: 12),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      'θ: ${_theta.toStringAsFixed(2)}  φ: ${_phi.toStringAsFixed(2)}',
                      style: const TextStyle(color: Colors.white54, fontSize: 11),
                    ),
                    Text(
                      'r: ${_radius.toStringAsFixed(1)}',
                      style: const TextStyle(color: Colors.white54, fontSize: 11),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
