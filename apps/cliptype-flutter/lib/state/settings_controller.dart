import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import '../l10n/app_localizations.dart';
import '../model/app_settings.dart';
import '../model/build_info.dart';
import '../model/app_status.dart';
import '../services/native_bridge.dart';

enum AutoSaveStatus { saved, pending, saving, error }

class SettingsController extends ChangeNotifier {
  SettingsController({NativeBridge? bridge})
    : bridge = bridge ?? NativeBridge();

  static const autoSaveDebounce = Duration(milliseconds: 320);

  final NativeBridge bridge;
  AppSettings settings = AppSettings.defaults();
  AppStatus status = AppStatus.initial();
  BuildInfo buildInfo = BuildInfo.unknown;
  bool loading = true;
  bool saving = false;
  AutoSaveStatus autoSaveStatus = AutoSaveStatus.saved;
  ClipTypeLanguage language = ClipTypeLanguage.english;
  String triggerAvailability = 'not_checked';
  String cancelAvailability = 'not_checked';
  String overallAvailability = 'not_checked';

  StreamSubscription<Map<Object?, Object?>>? _events;
  Timer? _activeObservation;
  Timer? _saveDebounce;
  Future<void>? _saveWorker;
  AppSettings? _pendingSettings;
  AppSettings? _failedSettings;
  int _saveRevision = 0;
  int _lastSuccessfulRevision = 0;
  bool _initialized = false;
  bool _disposed = false;
  String? _messageCode;
  String? _errorCode;
  String? _validationCode;

  ClipTypeLocalizations get l10n => ClipTypeLocalizations(language.locale);
  String? get message => _message(_messageCode);
  String? get error => _errorCode == null ? null : _errorMessage(_errorCode!);
  String? get validationError =>
      _validationCode == null ? null : l10n.validationMessage(_validationCode);

  bool get hasLocalSaveWork =>
      _pendingSettings != null || _saveDebounce != null || _saveWorker != null;
  bool get canRetrySave => _failedSettings != null && _validationCode == null;
  bool get permissionGranted => status.permission == 'granted';
  bool get readyForInput =>
      settings.enabled &&
      permissionGranted &&
      status.hotkeysRegistered &&
      status.bridgeAvailable &&
      !status.active;

  String get readinessReason {
    if (!status.bridgeAvailable) return 'bridge';
    if (!settings.enabled) return 'disabled';
    if (!permissionGranted) return 'permission';
    if (!status.hotkeysRegistered) return 'shortcuts';
    if (status.active) return 'active';
    return 'ready';
  }

  void clearFeedback() {
    _messageCode = null;
    _errorCode = null;
    _notifyListeners();
  }

  void resetHotkeyAvailability() {
    triggerAvailability = 'not_checked';
    cancelAvailability = 'not_checked';
    overallAvailability = 'not_checked';
    _notifyListeners();
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
    try {
      buildInfo = BuildInfo.fromMap(await bridge.getBuildInfo());
    } catch (_) {
      buildInfo = BuildInfo.unknown;
    }
    await refresh();
  }

  void setLanguage(ClipTypeLanguage next) {
    if (language == next) return;
    language = next;
    _notifyListeners();
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
      if (!hasLocalSaveWork) settings = nextSettings;
      status = nextStatus;
      final bridgeError = state['bridgeError'] as String?;
      if (bridgeError != null || !_isSettingsSaveError(_errorCode)) {
        _errorCode = bridgeError;
      }
      _syncActiveObservation();
    } on PlatformException catch (exception) {
      _errorCode = exception.code;
    } catch (_) {
      _errorCode = 'bridge_unavailable';
    } finally {
      loading = false;
      _notifyListeners();
    }
  }

  /// Queues a validated settings snapshot for automatic persistence.
  ///
  /// UI controls call this as soon as their value changes. Continuous inputs
  /// such as text fields and sliders are coalesced for a short interval; a
  /// discrete control can pass `Duration.zero` for an immediate save. The
  /// latest complete snapshot wins, and saves are serialized so rapid changes
  /// cannot write an older value after a newer one.
  void updateSettings(
    AppSettings proposed, {
    Duration debounce = autoSaveDebounce,
  }) {
    _saveRevision += 1;
    _saveDebounce?.cancel();
    _saveDebounce = null;
    _pendingSettings = null;
    _failedSettings = null;
    _messageCode = null;
    _errorCode = null;
    _validationCode = proposed.validationCode();

    if (_validationCode != null) {
      autoSaveStatus = AutoSaveStatus.error;
      _notifyListeners();
      return;
    }

    _pendingSettings = proposed;
    autoSaveStatus = AutoSaveStatus.pending;
    _saveDebounce = Timer(debounce, () {
      _saveDebounce = null;
      unawaited(_drainAutosave());
    });
    _notifyListeners();
  }

  /// Persists one snapshot immediately. This remains available to callers
  /// that need an awaitable operation; normal UI controls use
  /// [updateSettings] and save automatically.
  Future<bool> save(AppSettings proposed) async {
    updateSettings(proposed, debounce: Duration.zero);
    if (_validationCode != null) return false;
    final revision = _saveRevision;
    await flushPendingSaves();
    return _lastSuccessfulRevision == revision;
  }

  /// Forces any queued valid snapshot to the native settings store.
  Future<void> flushPendingSaves() async {
    _saveDebounce?.cancel();
    _saveDebounce = null;
    await _drainAutosave();
  }

  /// Retries the most recent native save failure without requiring the user to
  /// change a setting again.
  void retryFailedSave() {
    final failed = _failedSettings;
    if (failed == null || _validationCode != null) return;
    updateSettings(failed, debounce: Duration.zero);
  }

  Future<void> _drainAutosave() async {
    final existing = _saveWorker;
    if (existing != null) {
      await existing;
      return;
    }

    final worker = _runAutosave();
    _saveWorker = worker;
    try {
      await worker;
    } finally {
      if (identical(_saveWorker, worker)) _saveWorker = null;
    }
  }

  Future<void> _runAutosave() async {
    while (_pendingSettings != null) {
      final proposed = _pendingSettings!;
      final revision = _saveRevision;
      _pendingSettings = null;
      saving = true;
      autoSaveStatus = AutoSaveStatus.saving;
      _notifyListeners();

      final result = await _persist(proposed);
      if (result) {
        settings = proposed;
        _failedSettings = null;
        _lastSuccessfulRevision = revision;
        if (revision == _saveRevision && _pendingSettings == null) {
          autoSaveStatus = AutoSaveStatus.saved;
        } else if (_pendingSettings != null) {
          autoSaveStatus = AutoSaveStatus.pending;
        } else {
          // A newer invalid edit superseded this successful write.
          autoSaveStatus = AutoSaveStatus.error;
        }
      } else if (_pendingSettings != null) {
        _failedSettings = proposed;
        autoSaveStatus = AutoSaveStatus.pending;
      } else {
        _failedSettings = proposed;
        autoSaveStatus = AutoSaveStatus.error;
      }
      _notifyListeners();
    }
    saving = false;
    if (_pendingSettings == null && autoSaveStatus == AutoSaveStatus.saving) {
      autoSaveStatus = AutoSaveStatus.saved;
    }
    _notifyListeners();
  }

  Future<bool> _persist(AppSettings proposed) async {
    try {
      final result = await bridge.saveSettings(proposed);
      final resultCode = result['result'];
      if (resultCode != 'ok') {
        _errorCode = resultCode is String
            ? 'result:$resultCode'
            : 'settings_failed';
        return false;
      }
      _errorCode = null;
      return true;
    } catch (_) {
      _errorCode = 'settings_failed';
      return false;
    }
  }

  Future<bool> applyHotkeys(String trigger, String cancel) async {
    triggerAvailability = 'checking';
    cancelAvailability = 'checking';
    overallAvailability = 'checking';
    _errorCode = null;
    _messageCode = null;
    _notifyListeners();
    try {
      final result = await bridge.applyHotkeys(trigger, cancel);
      final code = result['result'] as String? ?? 'unknown';
      if (code == 'ok' || code == 'available') {
        settings = settings.copyWith(
          triggerHotkey: trigger,
          cancelHotkey: cancel,
        );
        triggerAvailability = 'available';
        cancelAvailability = 'available';
        overallAvailability = 'available';
        _messageCode = 'availability:available';
        _errorCode = null;
        await refresh();
        return true;
      }
      triggerAvailability = code;
      cancelAvailability = code;
      overallAvailability = code;
      _errorCode = 'availability:$code';
      return false;
    } catch (_) {
      triggerAvailability = 'unknown';
      cancelAvailability = 'unknown';
      overallAvailability = 'unknown';
      _errorCode = 'availability_failed';
      return false;
    } finally {
      _notifyListeners();
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
    _notifyListeners();
  }

  Future<void> trigger() async {
    try {
      final result = await bridge.trigger();
      final resultCode = result['result'] as String?;
      _messageCode = 'result:$resultCode';
      _syncActiveObservation(force: true);
    } catch (_) {
      _errorCode = 'trigger_failed';
    }
    _notifyListeners();
  }

  Future<void> cancel() async {
    try {
      final result = await bridge.cancel();
      _messageCode = 'result:${result['result'] as String?}';
    } catch (_) {
      _errorCode = 'cancel_failed';
    }
    _notifyListeners();
  }

  Future<void> requestAccessibility() async {
    try {
      final result = await bridge.requestAccessibility();
      _messageCode = 'result:${result['result'] as String?}';
      await refresh();
    } catch (_) {
      _errorCode = 'permission_request_failed';
      _notifyListeners();
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
    _notifyListeners();
  }

  Future<void> disposeAsync() async {
    _activeObservation?.cancel();
    await _events?.cancel();
    dispose();
  }

  void _handleEvent(Map<Object?, Object?> event) {
    final type = event['type'] as String?;
    if (type == null) return;
    if (type == 'hotkeyApplied' || type == 'hotkeysApplied') {
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

  bool _isSettingsSaveError(String? code) {
    return code == 'settings_failed' || code?.startsWith('result:') == true;
  }

  String? _message(String? code) {
    if (code == null) return null;
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
    _disposed = true;
    _saveDebounce?.cancel();
    _saveDebounce = null;
    _activeObservation?.cancel();
    _events?.cancel();
    super.dispose();
  }

  void _notifyListeners() {
    if (!_disposed) notifyListeners();
  }
}
