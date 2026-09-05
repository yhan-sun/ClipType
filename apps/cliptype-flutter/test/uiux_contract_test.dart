import 'package:cliptype_flutter/model/app_settings.dart';
import 'package:cliptype_flutter/model/app_status.dart';
import 'package:cliptype_flutter/model/build_info.dart';
import 'package:cliptype_flutter/screens/about_page.dart';
import 'package:cliptype_flutter/screens/overview_page.dart';
import 'package:cliptype_flutter/screens/typing_page.dart';
import 'package:cliptype_flutter/state/settings_controller.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

Widget host(Widget child) => MaterialApp(home: Scaffold(body: child));

void main() {
  testWidgets(
    'Code mode exposes corrected-typo control instead of disabling it',
    (tester) async {
      final controller = SettingsController();
      controller.settings = AppSettings.defaults().copyWith(
        mode: InjectionMode.code,
      );

      await tester.pumpWidget(host(TypingPage(controller: controller)));

      expect(find.text('Code input'), findsOneWidget);
      final sliders = tester.widgetList<Slider>(find.byType(Slider)).toList();
      expect(sliders.length, 2);
      expect(sliders.every((slider) => slider.onChanged != null), isTrue);
      controller.dispose();
    },
  );

  testWidgets(
    'temporarily empty speed field does not turn the saved value into zero',
    (tester) async {
      final controller = SettingsController();
      controller.settings = AppSettings.defaults().copyWith(
        mode: InjectionMode.keyboard,
      );
      await tester.pumpWidget(host(TypingPage(controller: controller)));

      final field = find.byType(TextField).first;
      await tester.enterText(field, '');
      await tester.pump(const Duration(milliseconds: 400));

      expect(controller.settings.charactersPerSecond, 40);
      controller.dispose();
    },
  );

  testWidgets(
    'About renders runtime build information instead of a hard-coded beta',
    (tester) async {
      final controller = SettingsController();
      controller.buildInfo = const BuildInfo(
        version: '0.2.0-beta.7',
        build: '6',
        architecture: 'Apple Silicon arm64',
        channel: 'Apple Silicon testing preview',
        signing: 'Ad-hoc signed',
        notarization: 'Not notarized',
      );

      await tester.pumpWidget(host(AboutPage(controller: controller)));

      expect(find.text('0.2.0-beta.7 (6)'), findsOneWidget);
      expect(find.text('0.2.0-beta.2'), findsNothing);
      controller.dispose();
    },
  );

  testWidgets('Overview makes missing permission the next setup action', (
    tester,
  ) async {
    final controller = SettingsController();
    controller.settings = AppSettings.defaults();
    controller.status = const AppStatus(
      phase: SessionPhase.idle,
      backend: null,
      completion: null,
      permission: 'not_granted',
      startup: 'not_registered',
      generation: 0,
      batchesCompleted: 0,
      hotkeysRegistered: true,
      bridgeAvailable: true,
    );

    await tester.pumpWidget(
      host(
        OverviewPage(
          controller: controller,
          onOpenInput: () {},
          onOpenShortcuts: () {},
          onOpenSystem: () {},
          onTrigger: () {},
        ),
      ),
    );

    expect(find.text('Accessibility permission required'), findsOneWidget);
    expect(find.text('Grant Accessibility'), findsOneWidget);
    controller.dispose();
  });
}
