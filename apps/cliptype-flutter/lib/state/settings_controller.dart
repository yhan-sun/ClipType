import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import '../l10n/app_localizations.dart';
import '../model/app_settings.dart';
import '../model/app_status.dart';
import '../services/native_bridge.dart';

class SettingsController extends ChangeNotifier {
  SettingsController({NativeBridge? bridge})
    : bridge = bridge ?? NativeBridge();

  final NativeBridge bridge;
  AppSettings settings = AppSettings.defaults();
  AppStatus status = AppStatus.initial();
  bool loading = true;
  bool saving = false;
  ClipTypeLanguage language = ClipTypeLanguage.english;
  String triggerAvailability = 'not_checked';
  String cancelAvailability = 'not_checked';
  String overallAvailability = 'not_checked';

  StreamSubscription<Map<Object?, Object?>>? _events;
  Timer? _activeObservation;
  bool _initialized = false;
  String? _messageCode;
  String? _errorCode;
  String? _validationCode;

  ClipTypeLocalizations get l10n => ClipTypeLocalizations(language.locale);
  String? get message => _message(_messageCode);
  String? get error {
    final validationCode = _validationCode;
    if (validationCode != null) return l10n.validationMessage(validationCode);
    final errorCode = _errorCode;
    return errorCode == null ? null : _errorMessage(errorCode);
  }

  Future<void> initialize() async {
    if (_initialized) return;
    _initialized = true;
    _events = bridge.events.listen(_handleEvent);
    try {
      final result = await bridge.getInterfaceLanguage();
      language = ClipTypeLanguageValues.fromCode(result['language'] as String?);
    } catch (_) {
      // English remains the safe default if the native shell is unavailable.
    }
    await refresh();
  }

  void setLanguage(ClipTypeLanguage next) {
    if (language == next) return;
    language = next;
    notifyListeners();
    unawaited(_sendLanguage(next));
  }

  Future<void> _sendLanguage(ClipTypeLanguage next) async {
    try {
      await bridge.setInterfaceLanguage(next.code);
    } catch (_) {
      // The Flutter UI remains localized if an older native shell is in use.
    }
  }

  Future<void> refresh() async {
    try {
      final state = await bridge.getState();
      final nextSettings = AppSettings.fromMap(state);
      final nextStatus = AppStatus.fromMap(state);
      settings = nextSettings;
      status = nextStatus;
      final bridgeError = state['bridgeError'] as String?;
      _errorCode = bridgeError;
      _syncActiveObservation();
    } on PlatformException catch (exception) {
      _errorCode = exception.code;
    } catch (_) {
      _errorCode = 'bridge_unavailable';
    } finally {
      loading = false;
      notifyListeners();
    }
  }

  Future<bool> save(AppSettings proposed) async {
    final validationCode = proposed.validationCode();
    if (validationCode != null) {
      _messageCode = null;
      _errorCode = null;
      _validationCode = validationCode;
      notifyListeners();
      return false;
    }
    saving = true;
    _errorCode = null;
    _validationCode = null;
    notifyListeners();
    try {
      final result = await bridge.saveSettings(proposed);
      if (_isFailure(result)) {
        _errorCode = 'result:${result['result'] as String?}';
        return false;
      }
      settings = proposed;
      _messageCode = 'settings_applied';
      await refresh();
      return true;
    } catch (_) {
      _errorCode = 'settings_failed';
      return false;
    } finally {
      saving = false;
      notifyListeners();
    }
  }

  Future<void> probeHotkeys(String trigger, String cancel) async {
    try {
      final result = await bridge.probeHotkeys(trigger, cancel);
      triggerAvailability = result['trigger'] as String? ?? 'unknown';
      cancelAvailability = result['cancel'] as String? ?? 'unknown';
      overallAvailability = result['overall'] as String? ?? 'unknown';
      _messageCode = 'availability:$overallAvailability';
      _errorCode = overallAvailability == 'available'
          ? null
          : 'availability:$overallAvailability';
    } catch (_) {
      triggerAvailability = 'unknown';
      cancelAvailability = 'unknown';
      overallAvailability = 'unknown';
      _errorCode = 'availability_failed';
    }
    notifyListeners();
  }

  Future<void> trigger() async {
    try {
      final result = await bridge.trigger();
      final resultCode = result['result'] as String?;
      _messageCode = 'result:$resultCode';
      if (resultCode == 'permission_required') {
        // A trigger is an explicit user action. Make the safe failure
        // actionable without attempting to change macOS consent.
        try {
          await bridge.openAccessibilitySettings();
        } catch (_) {
          // Keep the fixed permission message visible if Settings cannot open.
        }
      }
      _syncActiveObservation(force: true);
    } catch (_) {
      _errorCode = 'trigger_failed';
    }
    notifyListeners();
  }

  Future<void> cancel() async {
    try {
      final result = await bridge.cancel();
      _messageCode = 'result:${result['result'] as String?}';
    } catch (_) {
      _errorCode = 'cancel_failed';
    }
    notifyListeners();
  }

  Future<void> requestAccessibility() async {
    try {
      final result = await bridge.requestAccessibility();
      _messageCode = 'result:${result['result'] as String?}';
      await refresh();
    } catch (_) {
      _errorCode = 'permission_request_failed';
      notifyListeners();
    }
  }

  Future<void> openAccessibilitySettings() async {
    try {
      final result = await bridge.openAccessibilitySettings();
      _messageCode = 'result:${result['result'] as String?}';
      await refresh();
    } catch (_) {
      _errorCode = 'system_settings_failed';
    }
    notifyListeners();
  }

  Future<void> disposeAsync() async {
    _activeObservation?.cancel();
    await _events?.cancel();
    dispose();
  }

  void _handleEvent(Map<Object?, Object?> event) {
    final type = event['type'] as String?;
    if (type == null) return;
    if (type == 'hotkeyApplied') {
      overallAvailability = 'available';
      triggerAvailability = 'available';
      cancelAvailability = 'available';
    }
    if (type == 'hotkeyConflict') {
      overallAvailability = event['availability'] as String? ?? 'conflict';
      _errorCode = 'availability:$overallAvailability';
    }
    if (type == 'sessionStarted') {
      _messageCode = 'session_started';
    }
    if (type == 'sessionCompleted' ||
        type == 'sessionCancelled' ||
        type == 'sessionFailed') {
      _messageCode = 'completion:${event['completion'] as String?}';
    }
    unawaited(refresh());
  }

  void _syncActiveObservation({bool force = false}) {
    if ((force || status.active) && _activeObservation == null) {
      _activeObservation = Timer.periodic(const Duration(milliseconds: 120), (
        _,
      ) async {
        await refresh();
        if (!status.active) {
          _activeObservation?.cancel();
          _activeObservation = null;
        }
      });
    }
    if (!status.active && _activeObservation != null) {
      _activeObservation?.cancel();
      _activeObservation = null;
    }
  }

  bool _isFailure(Object? result) {
    return result is! String ||
        !{'ok', 'started', 'cancel_requested'}.contains(result);
  }

  String? _message(String? code) {
    if (code == null) return null;
    if (code == 'settings_applied') return l10n.settingsApplied;
    if (code == 'session_started') return l10n.sessionStarted;
    if (code.startsWith('result:')) {
      return l10n.resultMessage(code.substring('result:'.length));
    }
    if (code.startsWith('availability:')) {
      return l10n.availabilityMessage(code.substring('availability:'.length));
    }
    if (code.startsWith('completion:')) {
      return l10n.completionMessage(code.substring('completion:'.length));
    }
    return l10n.resultMessage(null);
  }

  String _errorMessage(String code) {
    if (code.startsWith('result:')) {
      return l10n.resultMessage(code.substring('result:'.length));
    }
    if (code.startsWith('availability:')) {
      return l10n.availabilityMessage(code.substring('availability:'.length));
    }
    return l10n.errorMessage(code);
  }

  @override
  void dispose() {
    _activeObservation?.cancel();
    _events?.cancel();
    super.dispose();
  }
}
