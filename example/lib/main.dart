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
  BigInt? _cubeId;
  double _rotationY = 0;

  double _theta = 0.0;
  double _phi = 0.2;
  double _radius = 5.0;
  bool _autoRotate = true;

  static const double _panSensitivity = 0.007;
  static const double _zoomStep = 0.5;
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
    controller.addCube(x: 0, y: 0, z: 0, r: 1.0, g: 0.4, b: 0.2).then((id) {
      setState(() => _cubeId = id);
    });
  }

  void _onTick(Rust3DController controller, double elapsedSec, double deltaSec) {
    if (_autoRotate) {
      _rotationY += deltaSec * 1.0;
      controller.setNodeTransform(nodeId: _cubeId!, ry: _rotationY);
    }
  }

  void _onPanStart(DragStartDetails d) {
    setState(() => _autoRotate = false);
  }

  void _onPanUpdate(DragUpdateDetails d) {
    _theta -= d.delta.dx * _panSensitivity;
    _phi += d.delta.dy * _panSensitivity;
    _phi = _phi.clamp(_phiMin, _phiMax);
    _updateCameraPosition();
    setState(() {});
  }

  void _onPointerSignal(PointerSignalEvent event) {
    if (event is PointerScrollEvent) {
      _radius -= event.scrollDelta.dy * _zoomStep;
      _radius = _radius.clamp(_minRadius, _maxRadius);
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
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        IconButton(
                          icon: Icon(
                            _autoRotate ? Icons.pause : Icons.play_arrow,
                            color: Colors.white,
                            size: 20,
                          ),
                          onPressed: () =>
                              setState(() => _autoRotate = !_autoRotate),
                          tooltip: _autoRotate
                              ? 'Stop rotation'
                              : 'Start rotation',
                          padding: EdgeInsets.zero,
                          constraints: const BoxConstraints(),
                        ),
                        const SizedBox(width: 4),
                        Text(
                          _autoRotate ? 'Auto' : 'Manual',
                          style: const TextStyle(
                            color: Colors.white70,
                            fontSize: 12,
                          ),
                        ),
                      ],
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
