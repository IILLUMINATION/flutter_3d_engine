import 'dart:math' as math;
import 'dart:ui';

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:irondash_engine_context/irondash_engine_context.dart';
import 'package:flutter_rust_3d/flutter_rust_3d.dart';
import 'package:flutter_rust_3d/src/rust/frb_generated.dart';
import 'package:flutter_rust_3d/src/rust/api/simple.dart';
import 'package:flutter_rust_3d/src/rust/core/scene.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await SystemChrome.setPreferredOrientations([
    DeviceOrientation.landscapeLeft,
    DeviceOrientation.landscapeRight,
  ]);
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

  static const double renderWidth = 800;
  static const double renderHeight = 450;

  final Set<PhysicalKeyboardKey> _pressedKeys = {};
  bool _isRotatingCamera = false;

  @override
  void initState() {
    super.initState();
    HardwareKeyboard.instance.addHandler(_onKeyEvent);
  }

  @override
  void dispose() {
    HardwareKeyboard.instance.removeHandler(_onKeyEvent);
    super.dispose();
  }

  bool _onKeyEvent(KeyEvent event) {
    if (event is KeyDownEvent) {
      _pressedKeys.add(event.physicalKey);
    } else if (event is KeyUpEvent) {
      _pressedKeys.remove(event.physicalKey);
    }
    return true;
  }

  void _onCreated(Rust3DController controller) {
    _controller = controller;
    controller.initDefaultCamera();
  }

  void _onTick(Rust3DController controller, double elapsedSec, double deltaSec) {
    controller.physicsStep(deltaSec.clamp(0.0, 0.05));

    double dx = 0.0;
    double dz = 0.0;
    if (_pressedKeys.contains(PhysicalKeyboardKey.keyW)) dz += 1.0;
    if (_pressedKeys.contains(PhysicalKeyboardKey.keyS)) dz -= 1.0;
    if (_pressedKeys.contains(PhysicalKeyboardKey.keyA)) dx -= 1.0;
    if (_pressedKeys.contains(PhysicalKeyboardKey.keyD)) dx += 1.0;
    if (dx != 0.0 || dz != 0.0) {
      controller.movePlayer(dx: dx, dz: dz);
    } else {
      controller.movePlayer(dx: 0.0, dz: 0.0);
    }

    if (_pressedKeys.contains(PhysicalKeyboardKey.space)) {
      controller.jumpPlayer();
    }
  }

  void _onTap() {
    if (_controller == null) return;
    final id = _controller!.spawnCubeInFront();
    setState(() => _cubeIds.add(id));
  }

  void _onPointerDown(PointerDownEvent event) {
    _isRotatingCamera = true;
    setState(() {});
  }

  void _onPointerMove(PointerMoveEvent event) {
    if (_controller == null || !_isRotatingCamera) return;
    _controller!.orbitCamera(event.delta.dx, event.delta.dy);
  }

  void _onPointerUp(PointerUpEvent event) {
    _isRotatingCamera = false;
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    final cursor = _isRotatingCamera ? SystemMouseCursors.none : SystemMouseCursors.basic;

    return Scaffold(
      body: MouseRegion(
        cursor: cursor,
        child: GestureDetector(
          onTap: _onTap,
          child: Listener(
            onPointerDown: _onPointerDown,
            onPointerMove: _onPointerMove,
            onPointerUp: _onPointerUp,
            child: Stack(
              children: [
                Positioned.fill(
                  child: FullScreenCanvas(
                    onCreated: _onCreated,
                    onTick: _onTick,
                  ),
                ),
                const Center(
                  child: IgnorePointer(
                    child: Crosshair(),
                  ),
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
                        width: 190,
                        decoration: BoxDecoration(
                          color: const Color(0xFF28282E).withOpacity(0.75),
                          borderRadius: BorderRadius.circular(12),
                          border: Border.all(color: Colors.white.withOpacity(0.08), width: 1),
                        ),
                        child: Column(
                          mainAxisSize: MainAxisSize.min,
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: [
                            const Text('FPS Sandbox',
                              textAlign: TextAlign.center,
                              style: TextStyle(
                                color: Color(0xFFE27F2D),
                                fontWeight: FontWeight.w700,
                                fontSize: 13,
                                letterSpacing: 0.5,
                              ),
                            ),
                            const SizedBox(height: 14),
                            const Divider(color: Colors.white10, height: 1, thickness: 1),
                            const SizedBox(height: 12),
                            _infoRow('Cubes Count', '${_cubeIds.length}', bold: true),
                            const SizedBox(height: 12),
                            const Divider(color: Colors.white10, height: 1, thickness: 1),
                            const SizedBox(height: 12),
                            _controlsRow(),
                          ],
                        ),
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
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

  Widget _controlsRow() {
    return const Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('WASD — Move', style: TextStyle(color: Colors.white54, fontSize: 10)),
        SizedBox(height: 2),
        Text('Space — Jump', style: TextStyle(color: Colors.white54, fontSize: 10)),
        SizedBox(height: 2),
        Text('Hold mouse — Look', style: TextStyle(color: Colors.white54, fontSize: 10)),
        SizedBox(height: 2),
        Text('Click — Spawn cube', style: TextStyle(color: Colors.white54, fontSize: 10)),
      ],
    );
  }
}

class FullScreenCanvas extends StatefulWidget {
  final Rust3DOnCreated? onCreated;
  final Rust3DOnTick? onTick;

  const FullScreenCanvas({super.key, this.onCreated, this.onTick});

  @override
  State<FullScreenCanvas> createState() => _FullScreenCanvasState();
}

class _FullScreenCanvasState extends State<FullScreenCanvas>
    with SingleTickerProviderStateMixin {
  static const int renderWidth = 800;
  static const int renderHeight = 450;

  Scene3D? _scene;
  int? _textureId;
  late final Ticker _ticker;
  Rust3DController? _controller;
  DateTime? _lastTickTime;

  @override
  void initState() {
    super.initState();
    _ticker = createTicker(_onTick);
    WidgetsBinding.instance.addPostFrameCallback((_) => _init());
  }

  Future<void> _init() async {
    final scene = await createScene();

    final handle = await EngineContext.instance.getEngineHandle();
    final textureId = await initNativeTexture(
      scene: scene,
      engineHandle: handle,
      width: renderWidth,
      height: renderHeight,
    );

    final controller = Rust3DController.wrap(scene, textureId: textureId);
    setState(() {
      _scene = scene;
      _textureId = textureId;
      _controller = controller;
    });
    widget.onCreated?.call(controller);
    _ticker.start();
  }

  void _onTick(Duration elapsed) {
    final scene = _scene;
    if (scene == null) return;

    final now = DateTime.now();
    final elapsedSec = elapsed.inMicroseconds / 1000000.0;
    final deltaSec = _lastTickTime == null
        ? 0.016
        : now.difference(_lastTickTime!).inMicroseconds / 1000000.0;
    _lastTickTime = now;

    final ctrl = _controller;
    if (ctrl != null) {
      widget.onTick?.call(ctrl, elapsedSec, deltaSec);
    }

    renderNativeFrame(scene: scene, width: renderWidth, height: renderHeight);
  }

  @override
  void dispose() {
    _ticker.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final textureId = _textureId;
    if (textureId == null) {
      return const Center(child: CircularProgressIndicator(color: Colors.white));
    }
    return Texture(textureId: textureId);
  }
}

class Crosshair extends StatelessWidget {
  const Crosshair({super.key});

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 20,
      height: 20,
      child: CustomPaint(
        painter: _CrosshairPainter(),
      ),
    );
  }
}

class _CrosshairPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = Colors.white.withOpacity(0.45)
      ..strokeWidth = 1.0
      ..strokeCap = StrokeCap.round;

    final cx = size.width / 2;
    final cy = size.height / 2;
    final gap = 3.0;
    final arm = 6.0;

    canvas.drawLine(Offset(cx - gap - arm, cy), Offset(cx - gap, cy), paint);
    canvas.drawLine(Offset(cx + gap, cy), Offset(cx + gap + arm, cy), paint);
    canvas.drawLine(Offset(cx, cy - gap - arm), Offset(cx, cy - gap), paint);
    canvas.drawLine(Offset(cx, cy + gap), Offset(cx, cy + gap + arm), paint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
