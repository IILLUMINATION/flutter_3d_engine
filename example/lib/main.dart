import 'dart:math' as math;
import 'dart:ui';

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
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      theme: ThemeData.dark().copyWith(
        scaffoldBackgroundColor: const Color(0xFF1E1E22),
      ),
      home: const DemoScreen(),
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
  bool _isDragging = false;

  static const double renderWidth = 1280;
  static const double renderHeight = 720;

  void _onCreated(Rust3DController controller) {
    _controller = controller;
    controller.initDefaultCamera();
  }

  void _onTick(Rust3DController controller, double elapsedSec, double deltaSec) {
    controller.physicsStep(deltaSec.clamp(0.0, 0.05));
  }

  void _spawnCube() {
    final rng = math.Random();
    final x = (rng.nextDouble() - 0.5) * 2.0;
    final z = (rng.nextDouble() - 0.5) * 2.0;
    final y = 4.0 + rng.nextDouble() * 2.0;
    final r = 0.9 + rng.nextDouble() * 0.1;
    final g = 0.45 + rng.nextDouble() * 0.15;
    final b = 0.2 + rng.nextDouble() * 0.1;
    _controller?.addCube(x: x, y: y, z: z, r: r, g: g, b: b).then((id) {
      setState(() => _cubeIds.add(id));
    });
  }

  void _onPointerDown(PointerDownEvent event) {
    if (_controller == null) return;
    final hit = _controller!.handlePointerDown(
      event.localPosition.dx, event.localPosition.dy, renderWidth, renderHeight,
    );
    _isDragging = hit;
    setState(() {});
  }

  void _onPointerMove(PointerMoveEvent event) {
    if (_controller == null) return;
    if (_isDragging) {
      _controller!.handlePointerMove(
        event.localPosition.dx, event.localPosition.dy, renderWidth, renderHeight,
      );
    } else {
      _controller!.orbitCamera(event.delta.dx, event.delta.dy);
    }
    setState(() {});
  }

  void _onPointerUp(PointerUpEvent event) {
    _controller?.handlePointerUp();
    _isDragging = false;
    setState(() {});
  }

  void _onPointerSignal(PointerSignalEvent event) {
    if (event is PointerScrollEvent) {
      _controller?.zoomCamera(event.scrollDelta.dy);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Listener(
        onPointerDown: _onPointerDown,
        onPointerMove: _onPointerMove,
        onPointerUp: _onPointerUp,
        onPointerSignal: _onPointerSignal,
        child: Stack(
          children: [
            Rust3DCanvas(
              width: renderWidth.toInt(),
              height: renderHeight.toInt(),
              onCreated: _onCreated,
              onTick: _onTick,
            ),
            Positioned(
              top: 24,
              right: 24,
              child: ClipRRect(
                borderRadius: BorderRadius.circular(12),
                child: BackdropFilter(
                  filter: ImageFilter.blur(sigmaX: 10, sigmaY: 10),
                  child: Container(
                    padding: const EdgeInsets.all(16),
                    width: 220,
                    decoration: BoxDecoration(
                      color: const Color(0xFF28282E).withOpacity(0.75),
                      borderRadius: BorderRadius.circular(12),
                      border: Border.all(color: Colors.white.withOpacity(0.08), width: 1),
                    ),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        ElevatedButton.icon(
                          onPressed: _spawnCube,
                          icon: const Icon(Icons.add_box_rounded, size: 16),
                          label: const Text('Spawn Cube',
                            style: TextStyle(fontWeight: FontWeight.w600, letterSpacing: 0.5)),
                          style: ElevatedButton.styleFrom(
                            backgroundColor: const Color(0xFFE27F2D),
                            foregroundColor: Colors.white,
                            elevation: 0,
                            padding: const EdgeInsets.symmetric(vertical: 12),
                            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                          ),
                        ),
                        const SizedBox(height: 16),
                        const Divider(color: Colors.white10, height: 1, thickness: 1),
                        const SizedBox(height: 12),
                        _infoRow('Cubes Count', '${_cubeIds.length}', bold: true),
                        const SizedBox(height: 12),
                        const Divider(color: Colors.white10, height: 1, thickness: 1),
                        const SizedBox(height: 12),
                        Row(children: [
                          Container(width: 8, height: 8,
                            decoration: BoxDecoration(shape: BoxShape.circle,
                              color: _isDragging ? const Color(0xFFE27F2D) : Colors.greenAccent)),
                          const SizedBox(width: 8),
                          Text(_isDragging ? 'Dragging Node' : 'Orbit Camera Active',
                            style: const TextStyle(fontSize: 11, color: Colors.white70, fontWeight: FontWeight.w500)),
                        ]),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _infoRow(String label, String value, {bool bold = false}) {
    return Row(mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [
      Text(label, style: TextStyle(color: Colors.white.withOpacity(0.45), fontSize: 11)),
      Text(value, style: TextStyle(color: Colors.white.withOpacity(0.85), fontSize: 11, fontFamily: 'monospace', fontWeight: bold ? FontWeight.bold : FontWeight.w400)),
    ]);
  }
}
