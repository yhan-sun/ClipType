import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../lib/l10n/app_localizations.dart';

void main() {
  test('terminal failures keep distinct labels in both languages', () {
    const reasons = [
      'modifier_conflict',
      'target_evidence_unavailable',
      'target_disappeared',
      'partial_input',
      'progress_unknown',
      'blocked_cause_unknown',
      'native_failure',
      'internal_invariant',
      'modifier_timeout',
    ];
    for (final locale in [const Locale('en'), const Locale('zh')]) {
      final strings = ClipTypeLocalizations(locale);
      final seen = <String>{};
      for (final reason in reasons) {
        final message = strings.completionMessage(reason);
        expect(message, isNot(strings.completionMessage('failed')));
        expect(strings.completionLabel(reason), message);
        expect(seen.add(message), isTrue);
      }
    }
  });
}
