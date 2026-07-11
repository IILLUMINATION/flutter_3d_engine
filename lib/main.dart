import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter_3d_engine/src/rust/api/simple.dart';
import 'package:flutter_3d_engine/src/rust/core/scene.dart';
import 'package:flutter_3d_engine/src/rust/frb_generated.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const AppRoot());
}

class AppRoot extends StatelessWidget {
  const AppRoot({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        backgroundColor: Colors.black,
        body: SafeArea(child: Center(child: Viewport3D())),
      ),
    );
  }
}

class Viewport3D extends StatefulWidget {
  const Viewport3D({super.key});

  @override
  State<Viewport3D> createState() => _Viewport3DState();
}

class _Viewport3DState extends State<Viewport3D>
    with SingleTickerProviderStateMixin {
  static const int bufferWidth = 320;
  static const int bufferHeight = 240;

  late final Ticker _ticker;
  Scene3D? _scene;
  ui.Image? _frame;
  DateTime? _lastTick;

  @override
  void initState() {
    super.initState();
    _ticker = createTicker(_onTick);
    _initScene();
  }

  Future<void> _initScene() async {
    final scene = await createScene();
    setState(() => _scene = scene);
    _ticker.start();
  }

  Future<void> _onTick(Duration elapsed) async {
    final scene = _scene;
    if (scene == null) return;

    await updateScene(scene: scene, dt: 0.016);
    final bytes = await renderScene(
      scene: scene,
      width: bufferWidth,
      height: bufferHeight,
    );

    final image = await _createImage(bytes);
    if (!mounted) return;
    setState(() => _frame = image);

    final now = DateTime.now();
    if (_lastTick != null) {
      final dt = now.difference(_lastTick!);
      final fps = 1000.0 / dt.inMilliseconds;
      print("Real Flutter FPS: ${fps.toStringAsFixed(1)}  (dt=${dt.inMilliseconds}ms)");
    }
    _lastTick = now;
  }

  Future<ui.Image> _createImage(Uint8List bytes) async {
    final buffer = await ui.ImmutableBuffer.fromUint8List(bytes);
    final descriptor = ui.ImageDescriptor.raw(
      buffer,
      width: bufferWidth,
      height: bufferHeight,
      pixelFormat: ui.PixelFormat.rgba8888,
    );
    final codec = await descriptor.instantiateCodec(
      targetWidth: bufferWidth,
      targetHeight: bufferHeight,
    );
    final frameInfo = await codec.getNextFrame();
    descriptor.dispose();
    return frameInfo.image;
  }

  @override
  void dispose() {
    _ticker.dispose();
    _frame?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final frame = _frame;
    return Stack(
      children: [
        SizedBox.expand(
          child: frame != null
              ? CustomPaint(painter: _FramePainter(frame))
              : const Center(
                  child: CircularProgressIndicator(color: Colors.white),
                ),
        ),
      ],
    );
  }
}

class _FramePainter extends CustomPainter {
  final ui.Image image;
  _FramePainter(this.image);

  @override
  void paint(Canvas canvas, Size size) {
    final src = Rect.fromLTWH(0, 0, image.width.toDouble(), image.height.toDouble());
    final dst = Rect.fromLTWH(0, 0, size.width, size.height);
    final paint = Paint()..filterQuality = FilterQuality.low;
    canvas.drawImageRect(image, src, dst, paint);
  }

  @override
  bool shouldRepaint(covariant _FramePainter oldDelegate) => true;
}
