import 'package:flutter/services.dart';

import '../model/app_settings.dart';

class NativeBridge {
  NativeBridge({MethodChannel? methods, EventChannel? events})
    : _methods = methods ?? const MethodChannel('io.cliptype/native'),
      _events = events ?? const EventChannel('io.cliptype/events');

  final MethodChannel _methods;
  final EventChannel _events;

  Stream<Map<Object?, Object?>> get events {
    return _events
        .receiveBroadcastStream()
        .where((value) => value is Map)
        .map((value) => Map<Object?, Object?>.from(value as Map));
  }

  Future<Map<Object?, Object?>> getState() async {
    return _mapResult(await _methods.invokeMethod<Object?>('getState'));
  }

  Future<Map<Object?, Object?>> getInterfaceLanguage() async {
    return _mapResult(
      await _methods.invokeMethod<Object?>('getInterfaceLanguage'),
    );
  }

  Future<Map<Object?, Object?>> setInterfaceLanguage(String language) async {
    return _mapResult(
      await _methods.invokeMethod<Object?>('setInterfaceLanguage', {
        'language': language,
      }),
    );
  }

  Future<Map<Object?, Object?>> saveSettings(AppSettings settings) async {
    return _mapResult(
      await _methods.invokeMethod<Object?>('saveSettings', settings.toMap()),
    );
  }

  Future<Map<Object?, Object?>> probeHotkeys(
    String trigger,
    String cancel,
  ) async {
    return _mapResult(
      await _methods.invokeMethod<Object?>('probeHotkeys', {
        'triggerHotkey': trigger,
        'cancelHotkey': cancel,
      }),
    );
  }

  Future<Map<Object?, Object?>> applyHotkeys(
    String trigger,
    String cancel,
  ) async {
    return _mapResult(
      await _methods.invokeMethod<Object?>('applyHotkeys', {
        'triggerHotkey': trigger,
        'cancelHotkey': cancel,
      }),
    );
  }

  Future<Map<Object?, Object?>> trigger() async {
    return _mapResult(await _methods.invokeMethod<Object?>('trigger'));
  }

  Future<Map<Object?, Object?>> cancel() async {
    return _mapResult(await _methods.invokeMethod<Object?>('cancel'));
  }

  Future<Map<Object?, Object?>> requestAccessibility() async {
    return _mapResult(
      await _methods.invokeMethod<Object?>('requestAccessibility'),
    );
  }

  Future<Map<Object?, Object?>> openAccessibilitySettings() async {
    return _mapResult(
      await _methods.invokeMethod<Object?>('openAccessibilitySettings'),
    );
  }

  Future<Map<Object?, Object?>> setStartAtLogin(bool enabled) async {
    return _mapResult(
      await _methods.invokeMethod<Object?>('setStartAtLogin', {
        'enabled': enabled,
      }),
    );
  }

  Future<Map<Object?, Object?>> quit() async {
    return _mapResult(await _methods.invokeMethod<Object?>('quit'));
  }

  Map<Object?, Object?> _mapResult(Object? value) {
    if (value is Map) {
      return Map<Object?, Object?>.from(value);
    }
    return const <Object?, Object?>{'result': 'native_failure'};
  }
}
