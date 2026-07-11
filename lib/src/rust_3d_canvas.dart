import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:irondash_engine_context/irondash_engine_context.dart';

import 'package:flutter_rust_3d/src/rust/api/simple.dart';
import 'package:flutter_rust_3d/src/rust/core/scene.dart';
import 'rust_3d_controller.dart';

typedef Rust3DOnCreated = void Function(Rust3DController controller);
typedef Rust3DOnTick = void Function(Rust3DController controller, double elapsedSec, double deltaSec);

class Rust3DCanvas extends StatefulWidget {
  final int width;
  final int height;
  final Rust3DOnCreated? onCreated;
  final Rust3DOnTick? onTick;
  final Widget? loading;
  final Color backgroundColor;

  const Rust3DCanvas({
    super.key,
    this.width = 1280,
    this.height = 720,
    this.onCreated,
    this.onTick,
    this.loading,
    this.backgroundColor = const Color(0xFF1E1E28),
  });

  @override
  State<Rust3DCanvas> createState() => _Rust3DCanvasState();
}

class _Rust3DCanvasState extends State<Rust3DCanvas>
    with SingleTickerProviderStateMixin {
  Scene3D? _scene;
  int? _textureId;
  late final Ticker _ticker;
  Rust3DController? _controller;
  DateTime? _lastTickTime;

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
      width: widget.width,
      height: widget.height,
    );

    final controller = Rust3DController.wrap(scene);
    setState(() {
      _scene = scene;
      _textureId = textureId;
      _controller = controller;
    });
    widget.onCreated?.call(controller);
    _ticker.start();
  }

  void _onTick(Duration elapsed) async {
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

    await renderNativeFrame(
      scene: scene,
      width: widget.width,
      height: widget.height,
    );
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
      return widget.loading ??
          const Center(child: CircularProgressIndicator(color: Colors.white));
    }
    return Scaffold(
      backgroundColor: widget.backgroundColor,
      body: Center(
        child: SizedBox(
          width: widget.width.toDouble(),
          height: widget.height.toDouble(),
          child: Texture(textureId: textureId),
        ),
      ),
    );
  }
}
