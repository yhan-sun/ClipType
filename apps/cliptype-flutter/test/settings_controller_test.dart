import 'dart:async';

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
  AppSettings stateSettings = AppSettings.defaults();
  String nextSaveResult = 'ok';
  String nextHotkeyResult = 'ok';

  @override
  Stream<Map<Object?, Object?>> get events => const Stream.empty();

  @override
  Future<Map<Object?, Object?>> getInterfaceLanguage() async {
    return const {'language': 'en'};
  }

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
  Future<Map<Object?, Object?>> getBuildInfo() async {
    return const {
      'version': '0.2.0-beta.7',
      'build': '6',
      'architecture': 'Apple Silicon arm64',
      'channel': 'Apple Silicon testing preview',
      'signing': 'Ad-hoc signed',
      'notarization': 'Not notarized',
    };
  }

  @override
  Future<Map<Object?, Object?>> getState() async {
    return {
      ...stateSettings.toMap(),
      'phase': 'idle',
      'permission': 'not_granted',
      'startup': 'not_registered',
      'hotkeysRegistered': true,
      'bridgeAvailable': true,
    };
  }

  @override
  Future<Map<Object?, Object?>> saveSettings(AppSettings settings) async {
    savedSettings.add(settings);
    if (nextSaveResult == 'ok') stateSettings = settings;
    return {'result': nextSaveResult};
  }

  @override
  Future<Map<Object?, Object?>> applyHotkeys(
    String trigger,
    String cancel,
  ) async {
    if (nextHotkeyResult == 'ok') {
      stateSettings = stateSettings.copyWith(
        triggerHotkey: trigger,
        cancelHotkey: cancel,
      );
    }
    return {'result': nextHotkeyResult};
  }
}

void main() {
  test(
    'permission failure stays actionable without opening System Settings',
    () async {
      final bridge = _FakeNativeBridge();
      final controller = SettingsController(bridge: bridge);

      await controller.trigger();

      expect(bridge.openSettingsCalls, 0);
      expect(controller.message, contains('Accessibility permission'));
      controller.dispose();
    },
  );

  test('build information comes from the native bundle contract', () async {
    final bridge = _FakeNativeBridge();
    final controller = SettingsController(bridge: bridge);

    await controller.initialize();

    expect(controller.buildInfo.version, '0.2.0-beta.7');
    expect(controller.buildInfo.build, '6');
    controller.dispose();
  });

  test('successful shortcut replacement becomes the current pair', () async {
    final bridge = _FakeNativeBridge();
    final controller = SettingsController(bridge: bridge);

    final applied = await controller.applyHotkeys('ctrl+alt+v', 'ctrl+alt+x');

    expect(applied, isTrue);
    expect(controller.settings.triggerHotkey, 'ctrl+alt+v');
    expect(controller.overallAvailability, 'available');
    controller.dispose();
  });

  test(
    'shortcut conflict leaves the previous working pair unchanged',
    () async {
      final bridge = _FakeNativeBridge()..nextHotkeyResult = 'conflict';
      final controller = SettingsController(bridge: bridge);
      final previous = controller.settings;

      final applied = await controller.applyHotkeys('ctrl+alt+v', 'ctrl+alt+x');

      expect(applied, isFalse);
      expect(controller.settings, previous);
      expect(controller.overallAvailability, 'conflict');
      controller.dispose();
    },
  );

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
