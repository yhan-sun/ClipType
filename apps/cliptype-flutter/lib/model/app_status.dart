import 'package:flutter/foundation.dart';

enum SessionPhase { idle, preparing, injecting, cancelling }

@immutable
class AppStatus {
  const AppStatus({
    required this.phase,
    required this.backend,
    required this.completion,
    required this.permission,
    required this.startup,
    required this.generation,
    required this.batchesCompleted,
  });

  factory AppStatus.initial() => const AppStatus(
    phase: SessionPhase.idle,
    backend: null,
    completion: null,
    permission: 'not_granted',
    startup: 'not_registered',
    generation: 0,
    batchesCompleted: 0,
  );

  factory AppStatus.fromMap(Map<Object?, Object?> value) {
    final phaseName = value['phase'] as String? ?? 'idle';
    return AppStatus(
      phase: SessionPhase.values.firstWhere(
        (candidate) => candidate.name == phaseName,
        orElse: () => SessionPhase.idle,
      ),
      backend: value['backend'] as String?,
      completion: value['completion'] as String?,
      permission: value['permission'] as String? ?? 'unknown',
      startup: value['startup'] as String? ?? 'unknown',
      generation: _asInt(value['generation']),
      batchesCompleted: _asInt(value['batchesCompleted']),
    );
  }

  final SessionPhase phase;
  final String? backend;
  final String? completion;
  final String permission;
  final String startup;
  final int generation;
  final int batchesCompleted;

  bool get active => phase != SessionPhase.idle;

  String get phaseLabel => switch (phase) {
    SessionPhase.idle => 'Ready',
    SessionPhase.preparing => 'Preparing',
    SessionPhase.injecting => 'Typing',
    SessionPhase.cancelling => 'Cancelling',
  };

  String get permissionLabel => switch (permission) {
    'granted' => 'Granted',
    'revoked' => 'Revoked',
    'not_requested' => 'Not requested',
    'not_granted' => 'Not granted',
    'not_required' => 'Not required',
    _ => 'Unknown',
  };

  String get startupLabel => switch (startup) {
    'enabled' => 'Enabled',
    'requires_approval' => 'Needs approval',
    'not_registered' => 'Off',
    'unsupported' => 'Unsupported',
    _ => 'Unknown',
  };

  static int _asInt(Object? value) => value is int ? value : 0;
}
