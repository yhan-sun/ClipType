import 'package:cliptype_flutter/model/app_settings.dart';
import 'package:cliptype_flutter/model/hotkey_spec.dart';
import 'package:cliptype_flutter/l10n/app_localizations.dart';
import 'package:cliptype_flutter/widgets/hotkey_recorder.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('default settings stay inside the product validation bounds', () {
    final settings = AppSettings.defaults();

    expect(settings.validationError(), isNull);
    expect(settings.toMap()['mode'], 'auto');
    expect(settings.triggerHotkey, 'ctrl+alt+shift+v');
  });

  test('invalid settings are rejected before native calls', () {
    final invalid = AppSettings.defaults().copyWith(
      charactersPerSecond: 0,
      triggerHotkey: 'ctrl+v',
      cancelHotkey: 'ctrl+v',
    );

    expect(invalid.validationError(), contains('different'));
  });

  test('hotkey parser identifies clear keys', () {
    expect(
      HotkeySpec.isClear(
        const KeyDownEvent(
          logicalKey: LogicalKeyboardKey.backspace,
          physicalKey: PhysicalKeyboardKey.backspace,
          timeStamp: Duration.zero,
        ),
      ),
      isTrue,
    );
  });

  test('simplified Chinese mode localizes core settings vocabulary', () {
    final l10n = ClipTypeLocalizations(const Locale('zh'));

    expect(l10n.general, '常规');
    expect(l10n.interfaceLanguage, '界面语言');
    expect(l10n.modeLabel(InjectionMode.clipboard), '剪贴板');
    expect(l10n.permissionLabel('not_granted'), '未授权');
    expect(l10n.resultMessage('permission_required'), contains('辅助功能'));
  });

  testWidgets('recorder captures only a focused complete combination', (
    tester,
  ) async {
    var value = 'ctrl+v';
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: HotkeyRecorder(
            title: 'Trigger shortcut',
            value: value,
            onChanged: (next) => value = next,
            onCleared: () => value = '',
          ),
        ),
      ),
    );

    await tester.tap(find.text('Record'));
    await tester.pump();
    await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
    expect(value, 'ctrl+v');

    await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
    await tester.sendKeyDownEvent(LogicalKeyboardKey.keyC);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.keyC);
    await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
    await tester.pump();
    expect(value, 'ctrl+c');

    await tester.tap(find.text('Record'));
    await tester.pump();
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    expect(value, 'ctrl+c');

    await tester.tap(find.text('Clear'));
    expect(value, isEmpty);
  });
}
