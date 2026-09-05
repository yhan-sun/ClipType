import 'package:flutter/foundation.dart';

@immutable
class BuildInfo {
  const BuildInfo({
    required this.version,
    required this.build,
    required this.architecture,
    required this.channel,
    required this.signing,
    required this.notarization,
  });

  factory BuildInfo.fromMap(Map<Object?, Object?> value) => BuildInfo(
    version: _text(value['version'], 'Unknown'),
    build: _text(value['build'], 'Unknown'),
    architecture: _text(value['architecture'], 'Unknown'),
    channel: _text(value['channel'], 'Unknown'),
    signing: _text(value['signing'], 'Unknown'),
    notarization: _text(value['notarization'], 'Unknown'),
  );

  static const unknown = BuildInfo(
    version: 'Unknown',
    build: 'Unknown',
    architecture: 'Unknown',
    channel: 'Unknown',
    signing: 'Unknown',
    notarization: 'Unknown',
  );

  final String version;
  final String build;
  final String architecture;
  final String channel;
  final String signing;
  final String notarization;

  String diagnostics({
    required String permission,
    required String phase,
    required bool bridgeAvailable,
  }) => [
    'ClipType $version ($build)',
    'architecture=$architecture',
    'channel=$channel',
    'signing=$signing',
    'notarization=$notarization',
    'permission=$permission',
    'phase=$phase',
    'bridge=${bridgeAvailable ? 'available' : 'unavailable'}',
  ].join('\n');

  static String _text(Object? value, String fallback) {
    final text = value is String ? value.trim() : '';
    return text.isEmpty ? fallback : text;
  }
}
