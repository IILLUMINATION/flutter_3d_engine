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

  final Set<PhysicalKeyboardKey> _pressedKeys = {};
  bool _mouseCaptured = false;

  int _selectedColorIndex = 0;
  double _fps = 0.0;

  static const List<Color> _palette = [
    Color(0xFFE27F2D),
    Color(0xFF4CAF50),
    Color(0xFF2196F3),
    Color(0xFFF44336),
    Color(0xFFFFEB3B),
    Color(0xFFFFFFFF),
    Color(0xFF000000),
    Color(0xFF9C27B0),
    Color(0xFFFF9800),
    Color(0xFF795548),
  ];

  Color get _selectedColor => _palette[_selectedColorIndex];

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
      switch (event.physicalKey) {
        case PhysicalKeyboardKey.digit1: setState(() => _selectedColorIndex = 0); return true;
        case PhysicalKeyboardKey.digit2: setState(() => _selectedColorIndex = 1); return true;
        case PhysicalKeyboardKey.digit3: setState(() => _selectedColorIndex = 2); return true;
        case PhysicalKeyboardKey.digit4: setState(() => _selectedColorIndex = 3); return true;
        case PhysicalKeyboardKey.digit5: setState(() => _selectedColorIndex = 4); return true;
        case PhysicalKeyboardKey.digit6: setState(() => _selectedColorIndex = 5); return true;
        case PhysicalKeyboardKey.digit7: setState(() => _selectedColorIndex = 6); return true;
        case PhysicalKeyboardKey.digit8: setState(() => _selectedColorIndex = 7); return true;
        case PhysicalKeyboardKey.digit9: setState(() => _selectedColorIndex = 8); return true;
        case PhysicalKeyboardKey.digit0: setState(() => _selectedColorIndex = 9); return true;
        default: _pressedKeys.add(event.physicalKey);
      }
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

    if (deltaSec > 0.0) {
      _fps = 1.0 / deltaSec;
    }
  }

  void _spawnCube() {
    if (_controller == null) return;
    final c = _selectedColor;
    final r = (c.r * 255.0).round().clamp(0, 255) / 255.0;
    final g = (c.g * 255.0).round().clamp(0, 255) / 255.0;
    final b = (c.b * 255.0).round().clamp(0, 255) / 255.0;
    final id = _controller!.spawnCubeInFront(r: r, g: g, b: b);
    setState(() => _cubeIds.add(id));
  }

  void _destroyLookedBlock() {
    if (_controller == null) return;
    if (_controller!.destroyLookedBlock()) {
      setState(() {
        if (_cubeIds.isNotEmpty) _cubeIds.removeLast();
      });
    }
  }

  void _onPointerDown(PointerDownEvent event) {
    if (event.buttons == kSecondaryMouseButton) {
      _spawnCube();
    } else if (event.buttons == kMiddleMouseButton) {
      _destroyLookedBlock();
    } else if (event.buttons == kPrimaryMouseButton) {
      setState(() => _mouseCaptured = !_mouseCaptured);
    }
  }

  void _onPointerHover(PointerHoverEvent event) {
    if (_controller == null || !_mouseCaptured) return;
    _controller!.orbitCamera(event.delta.dx, event.delta.dy);
  }

  @override
  Widget build(BuildContext context) {
    final cursor = _mouseCaptured ? SystemMouseCursors.none : SystemMouseCursors.basic;

    return Scaffold(
      body: MouseRegion(
        cursor: cursor,
        child: Listener(
          onPointerDown: _onPointerDown,
          onPointerHover: _onPointerHover,
          child: Stack(
            children: <Widget>[
              Positioned.fill(
                child: FullScreenCanvas(onCreated: _onCreated, onTick: _onTick),
              ),
              if (_mouseCaptured)
                const Center(
                  child: IgnorePointer(child: Crosshair()),
                ),
              if (!_mouseCaptured)
                Positioned.fill(
                  child: Center(
                    child: IgnorePointer(
                      child: Text(
                        'Click — Capture / release mouse\nRight-click — Spawn cube',
                        textAlign: TextAlign.center,
                        style: TextStyle(color: Colors.white54, fontSize: 13, height: 1.6),
                      ),
                    ),
                  ),
                ),
              if (!_mouseCaptured)
                Positioned(
                  top: 16,
                  right: 16,
                  child: Container(
                    padding: const EdgeInsets.all(12),
                    width: 180,
                    decoration: BoxDecoration(
                      color: const Color(0xCC1A1A1E),
                      border: Border.all(color: Colors.white12),
                    ),
                    child: const Column(
                      mainAxisSize: MainAxisSize.min,
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: <Widget>[
                        Text('FPS Sandbox',
                          style: TextStyle(color: Color(0xFFE27F2D), fontWeight: FontWeight.w600, fontSize: 13),
                        ),
                        SizedBox(height: 10),
                        Text('WASD  —  Move', style: TextStyle(color: Colors.white54, fontSize: 10)),
                        SizedBox(height: 2),
                        Text('Space  —  Jump', style: TextStyle(color: Colors.white54, fontSize: 10)),
                        SizedBox(height: 2),
                        Text('Click  —  Capture mouse', style: TextStyle(color: Colors.white54, fontSize: 10)),
                        SizedBox(height: 2),
                        Text('R-click  —  Spawn cube', style: TextStyle(color: Colors.white54, fontSize: 10)),
                        SizedBox(height: 2),
                        Text('M-click  —  Destroy cube', style: TextStyle(color: Colors.white54, fontSize: 10)),
                        SizedBox(height: 2),
                        Text('1-9, 0  —  Select color', style: TextStyle(color: Colors.white54, fontSize: 10)),
                      ],
                    ),
                  ),
                ),
                Positioned(
                  top: 8,
                  left: 8,
                  child: Text(
                    '${_fps.toStringAsFixed(0)} FPS',
                    style: const TextStyle(color: Colors.white38, fontSize: 11, fontFamily: 'monospace'),
                  ),
                ),
                if (_mouseCaptured)
                  Positioned(
                    bottom: 0,
                    left: 0,
                    right: 0,
                    child: Center(
                      child: ClipRRect(
                        borderRadius: const BorderRadius.vertical(top: Radius.circular(4)),
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
                          color: const Color(0xAA1E1E22),
                          child: Row(
                            mainAxisSize: MainAxisSize.min,
                            children: List.generate(_palette.length, (i) {
                              final selected = i == _selectedColorIndex;
                              return Container(
                                width: selected ? 28 : 24,
                                height: selected ? 28 : 24,
                                margin: const EdgeInsets.symmetric(horizontal: 1),
                                decoration: BoxDecoration(
                                  color: _palette[i],
                                  border: Border.all(
                                    color: selected ? Colors.white : Colors.white30,
                                    width: selected ? 2.5 : 1,
                                  ),
                                ),
                              );
                            }),
                          ),
                        ),
                      ),
                    ),
                  ),
            ],
          ),
        ),
      ),
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
  Scene3D? _scene;
  int? _textureId;
  late final Ticker _ticker;
  Rust3DController? _controller;
  DateTime? _lastTickTime;
  int _lastWidth = 0;
  int _lastHeight = 0;

  @override
  void initState() {
    super.initState();
    _ticker = createTicker(_onTick);
    WidgetsBinding.instance.addPostFrameCallback((_) => _init());
  }

  Size _renderSize() {
    final size = MediaQueryData.fromView(View.of(context)).size;
    final dpr = MediaQueryData.fromView(View.of(context)).devicePixelRatio;
    return Size(size.width * dpr, size.height * dpr);
  }

  Future<void> _init() async {
    final scene = await createScene();

    final renderSize = _renderSize();
    final width = renderSize.width.toInt();
    final height = renderSize.height.toInt();

    final handle = await EngineContext.instance.getEngineHandle();
    final textureId = await initNativeTexture(
      scene: scene,
      engineHandle: handle,
      width: width,
      height: height,
    );

    _lastWidth = width;
    _lastHeight = height;

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

    renderNativeFrame(scene: scene, width: _lastWidth, height: _lastHeight);
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
