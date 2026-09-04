import 'package:cliptype_flutter/services/native_bridge.dart';
import 'package:cliptype_flutter/state/settings_controller.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

class _FakeNativeBridge extends NativeBridge {
  _FakeNativeBridge()
    : super(
        methods: const MethodChannel('cliptype.test/native'),
        events: const EventChannel('cliptype.test/events'),
      );

  int openSettingsCalls = 0;

  @override
  Future<Map<Object?, Object?>> trigger() async {
    return const {'result': 'permission_required'};
  }

  @override
  Future<Map<Object?, Object?>> openAccessibilitySettings() async {
    openSettingsCalls += 1;
    return const {'result': 'settings_opened'};
  }

  @override
  Future<Map<Object?, Object?>> getState() async {
    return const {
      'phase': 'idle',
      'permission': 'not_granted',
      'startup': 'not_registered',
    };
  }
}

void main() {
  test('a permission failure opens the macOS Accessibility settings', () async {
    final bridge = _FakeNativeBridge();
    final controller = SettingsController(bridge: bridge);

    await controller.trigger();

    expect(bridge.openSettingsCalls, 1);
    expect(controller.message, contains('Accessibility permission'));
    controller.dispose();
  });
}
