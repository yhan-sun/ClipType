import 'package:flutter/foundation.dart';

enum InjectionMode { keyboard, clipboard, auto, code }

extension InjectionModeLabel on InjectionMode {
  String get label => switch (this) {
    InjectionMode.keyboard => 'Keyboard',
    InjectionMode.clipboard => 'Clipboard',
    InjectionMode.auto => 'Auto',
    InjectionMode.code => 'Code',
  };

  String get wireName => switch (this) {
    InjectionMode.keyboard => 'keyboard',
    InjectionMode.clipboard => 'clipboard',
    InjectionMode.auto => 'auto',
    InjectionMode.code => 'code',
  };
}

@immutable
class AppSettings {
  const AppSettings({
    required this.enabled,
    required this.notifications,
    required this.startAtLogin,
    required this.mode,
    required this.charactersPerSecond,
    required this.jitterPercent,
    required this.typoProbabilityPercent,
    required this.autoClipboardThreshold,
    required this.triggerHotkey,
    required this.cancelHotkey,
  });

  factory AppSettings.defaults() => const AppSettings(
    enabled: true,
    notifications: true,
    startAtLogin: false,
    mode: InjectionMode.auto,
    charactersPerSecond: 40,
    jitterPercent: 0,
    typoProbabilityPercent: 0,
    autoClipboardThreshold: 256,
    triggerHotkey: 'ctrl+alt+shift+v',
    cancelHotkey: 'ctrl+alt+shift+x',
  );

  factory AppSettings.fromMap(Map<Object?, Object?> value) {
    final modeName = value['mode'] as String? ?? 'auto';
    final mode = InjectionMode.values.firstWhere(
      (candidate) => candidate.wireName == modeName,
      orElse: () => InjectionMode.auto,
    );
    return AppSettings(
      enabled: value['enabled'] as bool? ?? true,
      notifications: value['notifications'] as bool? ?? true,
      startAtLogin: value['startAtLogin'] as bool? ?? false,
      mode: mode,
      charactersPerSecond: _asInt(value['charactersPerSecond'], 40),
      jitterPercent: _asInt(value['jitterPercent'], 0),
      typoProbabilityPercent: _asInt(value['typoProbabilityPercent'], 0),
      autoClipboardThreshold: _asInt(value['autoClipboardThreshold'], 256),
      triggerHotkey: value['triggerHotkey'] as String? ?? '',
      cancelHotkey: value['cancelHotkey'] as String? ?? '',
    );
  }

  final bool enabled;
  final bool notifications;
  final bool startAtLogin;
  final InjectionMode mode;
  final int charactersPerSecond;
  final int jitterPercent;
  final int typoProbabilityPercent;
  final int autoClipboardThreshold;
  final String triggerHotkey;
  final String cancelHotkey;

  AppSettings copyWith({
    bool? enabled,
    bool? notifications,
    bool? startAtLogin,
    InjectionMode? mode,
    int? charactersPerSecond,
    int? jitterPercent,
    int? typoProbabilityPercent,
    int? autoClipboardThreshold,
    String? triggerHotkey,
    String? cancelHotkey,
  }) {
    return AppSettings(
      enabled: enabled ?? this.enabled,
      notifications: notifications ?? this.notifications,
      startAtLogin: startAtLogin ?? this.startAtLogin,
      mode: mode ?? this.mode,
      charactersPerSecond: charactersPerSecond ?? this.charactersPerSecond,
      jitterPercent: jitterPercent ?? this.jitterPercent,
      typoProbabilityPercent:
          typoProbabilityPercent ?? this.typoProbabilityPercent,
      autoClipboardThreshold:
          autoClipboardThreshold ?? this.autoClipboardThreshold,
      triggerHotkey: triggerHotkey ?? this.triggerHotkey,
      cancelHotkey: cancelHotkey ?? this.cancelHotkey,
    );
  }

  Map<String, Object> toMap() => {
    'enabled': enabled,
    'notifications': notifications,
    'startAtLogin': startAtLogin,
    'mode': mode.wireName,
    'charactersPerSecond': charactersPerSecond,
    'jitterPercent': jitterPercent,
    'typoProbabilityPercent': typoProbabilityPercent,
    'autoClipboardThreshold': autoClipboardThreshold,
    'triggerHotkey': triggerHotkey,
    'cancelHotkey': cancelHotkey,
  };

  String? validationCode() {
    if (triggerHotkey.trim().isEmpty || cancelHotkey.trim().isEmpty) {
      return 'missing_hotkeys';
    }
    if (triggerHotkey == cancelHotkey) {
      return 'different_hotkeys';
    }
    if (charactersPerSecond < 1 || charactersPerSecond > 250) {
      return 'characters_per_second';
    }
    if (jitterPercent < 0 || jitterPercent > 95) {
      return 'jitter_percent';
    }
    if (typoProbabilityPercent < 0 || typoProbabilityPercent > 25) {
      return 'typo_probability_percent';
    }
    if (autoClipboardThreshold < 1) {
      return 'auto_clipboard_threshold';
    }
    return null;
  }

  String? validationError() => switch (validationCode()) {
    null => null,
    'missing_hotkeys' =>
      'Record both a Trigger and Cancel shortcut before saving.',
    'different_hotkeys' => 'Trigger and Cancel shortcuts must be different.',
    'characters_per_second' =>
      'Characters per second must be between 1 and 250.',
    'jitter_percent' => 'Jitter must be between 0% and 95%.',
    'typo_probability_percent' =>
      'Corrected typo probability must be between 0% and 25%.',
    'auto_clipboard_threshold' =>
      'The Auto clipboard threshold must be at least 1.',
    _ => 'The settings are invalid.',
  };

  @override
  bool operator ==(Object other) {
    return other is AppSettings &&
        other.enabled == enabled &&
        other.notifications == notifications &&
        other.startAtLogin == startAtLogin &&
        other.mode == mode &&
        other.charactersPerSecond == charactersPerSecond &&
        other.jitterPercent == jitterPercent &&
        other.typoProbabilityPercent == typoProbabilityPercent &&
        other.autoClipboardThreshold == autoClipboardThreshold &&
        other.triggerHotkey == triggerHotkey &&
        other.cancelHotkey == cancelHotkey;
  }

  @override
  int get hashCode => Object.hash(
    enabled,
    notifications,
    startAtLogin,
    mode,
    charactersPerSecond,
    jitterPercent,
    typoProbabilityPercent,
    autoClipboardThreshold,
    triggerHotkey,
    cancelHotkey,
  );

  static int _asInt(Object? value, int fallback) {
    return value is int ? value : fallback;
  }
}
