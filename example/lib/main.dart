import 'dart:ui'; // Нужен для BackdropFilter (размытие Blender)

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
        scaffoldBackgroundColor: const Color(
          0xFF1E1E22,
        ), // Цвет вьюпорта Blender
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

  // Состояние перетаскивания (для HUD индикатора)
  bool _isDraggingObject = false;

  static const double renderWidth = 1280;
  static const double renderHeight = 720;

  void _onCreated(Rust3DController controller) {
    _controller = controller;
    // Инициализируем дефолтную камеру в Rust
    _controller?.initDefaultCamera();
  }

  // Шаг физики — вызывается Ticker-ом в пакете
  void _onTick(
    Rust3DController controller,
    double elapsedSec,
    double deltaSec,
  ) {
    controller.physicsStep(deltaSec.clamp(0.0, 0.03));
  }

  void _spawnCube() {
    final rng = math.Random();
    final x = (rng.nextDouble() - 0.5) * 2.0;
    final z = (rng.nextDouble() - 0.5) * 2.0;
    final y = 4.0 + rng.nextDouble() * 2.0;

    // Blender-овские оранжевые оттенки
    final r = 0.9 + rng.nextDouble() * 0.1;
    final g = 0.45 + rng.nextDouble() * 0.15;
    final b = 0.2 + rng.nextDouble() * 0.1;

    _controller?.addCube(x: x, y: y, z: z, r: r, g: g, b: b).then((id) {
      setState(() => _cubeIds.add(id));
    });
  }

  // --- Единый роутер жестов мыши/тача (Передаем сырой ввод в Rust) ---

  void _onPointerDown(PointerDownEvent event) async {
    if (_controller == null) return;

    final hit = await _controller!.handlePointerDown(
      screenX: event.localPosition.dx,
      screenY: event.localPosition.dy,
      screenWidth: renderWidth,
      screenHeight: renderHeight,
    );

    setState(() {
      _isDraggingObject = hit;
    });
  }

  void _onPointerMove(PointerMoveEvent event) {
    if (_controller == null) return;

    if (_isDraggingObject) {
      // Тащим объект по осям в Rust
      _controller!.handlePointerMove(
        screenX: event.localPosition.dx,
        screenY: event.localPosition.dy,
        screenWidth: renderWidth,
        screenHeight: renderHeight,
      );
    } else {
      // Вращаем камеру напрямую в Rust (передаем только смещение dx, dy!)
      _controller!.orbitCamera(dx: event.delta.dx, dy: event.delta.dy);
    }
  }

  void _onPointerUp(PointerUpEvent event) {
    if (_controller == null) return;

    if (_isDraggingObject) {
      _controller!.handlePointerUp();
      setState(() {
        _isDraggingObject = false;
      });
    }
  }

  void _onPointerSignal(PointerSignalEvent event) {
    if (event is PointerScrollEvent && _controller != null) {
      // Зуммируем камеру напрямую в Rust (передаем только дельту скролла)
      _controller!.zoomCamera(delta: event.scrollDelta.dy);
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
            // Вьюпорт 3D рендеринга
            Positioned.fill(
              child: Center(
                child: SizedBox(
                  width: renderWidth,
                  height: renderHeight,
                  child: Texture(
                    textureId: _controller != null ? _controller!.textureId : 0,
                  ),
                ),
              ),
            ),

            // Скрытый Canvas для инициализации и тиков
            Rust3DCanvas(
              width: renderWidth,
              height: renderHeight,
              onCreated: _onCreated,
              onTick: _onTick,
            ),

            // Элегантная панель управления в стиле Blender Dark UI
            Positioned(
              top: 24,
              right: 24,
              child: ClipRRect(
                borderRadius: BorderRadius.circular(12),
                child: BackdropFilter(
                  filter: ImageFilter.blur(
                    sigmaX: 10,
                    sigmaY: 10,
                  ), // Размытие стекла
                  child: Container(
                    padding: const EdgeInsets.all(16),
                    width: 220,
                    decoration: BoxDecoration(
                      color: const Color(
                        0xFF28282E,
                      ).withOpacity(0.75), // Матовый темно-серый
                      borderRadius: BorderRadius.circular(12),
                      border: Border.all(
                        color: Colors.white.withOpacity(0.08),
                        width: 1,
                      ),
                    ),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        ElevatedButton.icon(
                          onPressed: _spawnCube,
                          icon: const Icon(Icons.add_box_rounded, size: 16),
                          label: const Text(
                            'Spawn Cube',
                            style: TextStyle(
                              fontWeight: FontWeight.w600,
                              letterSpacing: 0.5,
                            ),
                          ),
                          style: ElevatedButton.styleFrom(
                            backgroundColor: const Color(
                              0xFFE27F2D,
                            ), // Оранжевый цвет Blender
                            foregroundColor: Colors.white,
                            elevation: 0,
                            padding: const EdgeInsets.symmetric(vertical: 12),
                            shape: RoundedRectangleBorder(
                              borderRadius: BorderRadius.circular(8),
                            ),
                          ),
                        ),
                        const SizedBox(height: 16),
                        _buildDivider(),
                        const SizedBox(height: 12),
                        _buildInfoRow(
                          'Cubes Count',
                          '${_cubeIds.length}',
                          isValueBold: true,
                        ),
                        const SizedBox(height: 12),
                        _buildDivider(),
                        const SizedBox(height: 12),
                        Row(
                          children: [
                            Container(
                              width: 8,
                              height: 8,
                              decoration: BoxDecoration(
                                shape: BoxShape.circle,
                                color: _isDraggingObject
                                    ? const Color(0xFFE27F2D)
                                    : Colors.greenAccent,
                              ),
                            ),
                            const SizedBox(width: 8),
                            Text(
                              _isDraggingObject
                                  ? 'Dragging Node'
                                  : 'Orbit Camera Active',
                              style: const TextStyle(
                                fontSize: 11,
                                color: Colors.white70,
                                fontWeight: FontWeight.w500,
                              ),
                            ),
                          ],
                        ),
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

  Widget _buildDivider() {
    return Divider(
      color: Colors.white.withOpacity(0.05),
      height: 1,
      thickness: 1,
    );
  }

  Widget _buildInfoRow(String label, String value, {bool isValueBold = false}) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(
          label,
          style: TextStyle(
            color: Colors.white.withOpacity(0.45),
            fontSize: 11,
            fontWeight: FontWeight.w400,
          ),
        ),
        Text(
          value,
          style: TextStyle(
            color: Colors.white.withOpacity(0.85),
            fontSize: 11,
            fontFamily: 'monospace',
            fontWeight: isValueBold ? FontWeight.bold : FontWeight.w400,
          ),
        ),
      ],
    );
  }
}
