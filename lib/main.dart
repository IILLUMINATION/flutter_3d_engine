import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter_3d_engine/src/rust/api/simple.dart';
import 'package:flutter_3d_engine/src/rust/core/scene.dart';
import 'package:flutter_3d_engine/src/rust/frb_generated.dart';
import 'package:irondash_engine_context/irondash_engine_context.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const AppRoot());
}

class AppRoot extends StatelessWidget {
  const AppRoot({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: const Viewport3D(),
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
  static const int width = 1280;
  static const int height = 720;

  Scene3D? _scene;
  int? _textureId;
  late final Ticker _ticker;
  DateTime? _lastTick;

  @override
  void initState() {
    super.initState();
    _ticker = createTicker(_onTick);
    _init();
  }

  Future<void> _init() async {
    final scene = await createScene();

    final handle = await EngineContext.instance.getEngineHandle();
    final textureId = await initNativeTexture(
      scene: scene,
      engineHandle: handle,
      width: width,
      height: height,
    );

    setState(() {
      _scene = scene;
      _textureId = textureId;
    });
    _ticker.start();
  }

  void _onTick(Duration elapsed) async {
    final scene = _scene;
    if (scene == null) return;

    final now = DateTime.now();
    if (_lastTick != null) {
      final dt = now.difference(_lastTick!);
      final fps = 1000.0 / dt.inMilliseconds;
      print("Real Flutter FPS: ${fps.toStringAsFixed(1)}  (dt=${dt.inMilliseconds}ms)");
    }
    _lastTick = now;

    await updateScene(scene: scene, dt: 0.016);
    await renderNativeFrame(scene: scene, width: width, height: height);
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
    return Scaffold(
      backgroundColor: const Color(0xFF1E1E28),
      body: Center(
        child: SizedBox(
          width: width.toDouble(),
          height: height.toDouble(),
          child: Texture(textureId: textureId),
        ),
      ),
    );
  }
}
