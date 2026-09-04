import 'package:cliptype_flutter/model/app_settings.dart';
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
  final List<AppSettings> savedSettings = [];
  String nextSaveResult = 'ok';

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

  @override
  Future<Map<Object?, Object?>> saveSettings(AppSettings settings) async {
    savedSettings.add(settings);
    return {'result': nextSaveResult};
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

  test(
    'settings changes are auto-saved and rapid changes are coalesced',
    () async {
      final bridge = _FakeNativeBridge();
      final controller = SettingsController(bridge: bridge);
      final next = AppSettings.defaults().copyWith(
        enabled: false,
        notifications: false,
      );

      controller.updateSettings(
        AppSettings.defaults().copyWith(enabled: false),
        debounce: const Duration(milliseconds: 50),
      );
      controller.updateSettings(next, debounce: Duration.zero);
      await controller.flushPendingSaves();

      expect(bridge.savedSettings, [next]);
      expect(controller.settings, next);
      expect(controller.autoSaveStatus, AutoSaveStatus.saved);
      expect(controller.hasLocalSaveWork, isFalse);
      controller.dispose();
    },
  );

  test('invalid settings stay local and are not persisted', () async {
    final bridge = _FakeNativeBridge();
    final controller = SettingsController(bridge: bridge);
    final invalid = AppSettings.defaults().copyWith(triggerHotkey: '');

    controller.updateSettings(invalid, debounce: Duration.zero);
    await controller.flushPendingSaves();

    expect(bridge.savedSettings, isEmpty);
    expect(controller.validationError, contains('Trigger'));
    expect(controller.autoSaveStatus, AutoSaveStatus.error);
    controller.dispose();
  });

  test('a failed save can be retried without changing the setting', () async {
    final bridge = _FakeNativeBridge()..nextSaveResult = 'native_failure';
    final controller = SettingsController(bridge: bridge);
    final next = AppSettings.defaults().copyWith(enabled: false);

    controller.updateSettings(next, debounce: Duration.zero);
    await controller.flushPendingSaves();
    expect(controller.autoSaveStatus, AutoSaveStatus.error);
    expect(controller.canRetrySave, isTrue);

    bridge.nextSaveResult = 'ok';
    controller.retryFailedSave();
    await controller.flushPendingSaves();

    expect(bridge.savedSettings, [next, next]);
    expect(controller.settings, next);
    expect(controller.autoSaveStatus, AutoSaveStatus.saved);
    expect(controller.canRetrySave, isFalse);
    controller.dispose();
  });
}
