import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';

import 'l10n/app_localizations.dart';
import 'screens/about_page.dart';
import 'screens/general_page.dart';
import 'screens/permissions_page.dart';
import 'screens/shortcuts_page.dart';
import 'screens/typing_page.dart';
import 'state/settings_controller.dart';
import 'widgets/setting_card.dart';

class ClipTypeApp extends StatefulWidget {
  const ClipTypeApp({super.key});

  @override
  State<ClipTypeApp> createState() => _ClipTypeAppState();
}

class _ClipTypeAppState extends State<ClipTypeApp> {
  late final SettingsController _controller;
  Locale _locale = const Locale('en');

  @override
  void initState() {
    super.initState();
    _controller = SettingsController();
    unawaited(_initialize());
  }

  Future<void> _initialize() async {
    await _controller.initialize();
    if (!mounted) return;
    setState(() => _locale = _controller.language.locale);
  }

  void _setLanguage(ClipTypeLanguage language) {
    _controller.setLanguage(language);
    setState(() => _locale = language.locale);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'ClipType',
      debugShowCheckedModeBanner: false,
      locale: _locale,
      localizationsDelegates: const [
        ClipTypeLocalizations.delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
      supportedLocales: ClipTypeLocalizations.supportedLocales,
      theme: _theme(Brightness.light),
      darkTheme: _theme(Brightness.dark),
      themeMode: ThemeMode.system,
      home: AnimatedBuilder(
        animation: _controller,
        builder: (context, child) {
          if (_controller.loading) return const _LoadingView();
          return ClipTypeShell(
            controller: _controller,
            onLanguageChanged: _setLanguage,
          );
        },
      ),
    );
  }

  ThemeData _theme(Brightness brightness) {
    final scheme = ColorScheme.fromSeed(
      seedColor: const Color(0xFF5269D8),
      brightness: brightness,
    );
    return ThemeData(
      colorScheme: scheme,
      useMaterial3: true,
      visualDensity: VisualDensity.standard,
      scaffoldBackgroundColor: scheme.surface,
      appBarTheme: AppBarTheme(
        elevation: 0,
        scrolledUnderElevation: 0,
        backgroundColor: scheme.surface,
        surfaceTintColor: Colors.transparent,
        titleTextStyle: TextStyle(
          color: scheme.onSurface,
          fontSize: 18,
          fontWeight: FontWeight.w700,
        ),
      ),
      cardTheme: CardThemeData(
        elevation: 0,
        margin: EdgeInsets.zero,
        color: scheme.surfaceContainerLow,
        surfaceTintColor: Colors.transparent,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(16),
          side: BorderSide(color: scheme.outlineVariant),
        ),
      ),
      inputDecorationTheme: const InputDecorationTheme(isDense: true),
      navigationRailTheme: NavigationRailThemeData(
        backgroundColor: scheme.surfaceContainerLow,
        indicatorColor: scheme.secondaryContainer,
        useIndicator: true,
        groupAlignment: -0.82,
        minExtendedWidth: 190,
        selectedIconTheme: IconThemeData(color: scheme.onSecondaryContainer),
        selectedLabelTextStyle: TextStyle(
          color: scheme.onSurface,
          fontWeight: FontWeight.w700,
        ),
      ),
    );
  }
}

class ClipTypeShell extends StatefulWidget {
  const ClipTypeShell({
    required this.controller,
    required this.onLanguageChanged,
    super.key,
  });

  final SettingsController controller;
  final ValueChanged<ClipTypeLanguage> onLanguageChanged;

  @override
  State<ClipTypeShell> createState() => _ClipTypeShellState();
}

class _ClipTypeShellState extends State<ClipTypeShell> {
  int _page = 0;

  SettingsController get controller => widget.controller;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final feedback = controller.error ?? controller.message;
    return Scaffold(
      appBar: AppBar(
        titleSpacing: 22,
        title: Row(
          children: [
            Icon(
              Icons.content_paste_go,
              color: Theme.of(context).colorScheme.primary,
            ),
            const SizedBox(width: 10),
            const Text('ClipType'),
            const SizedBox(width: 14),
            StatusPill(
              label: l10n.phaseLabel(controller.status.phase),
              good: !controller.status.active,
            ),
          ],
        ),
        actions: [
          FilledButton.tonalIcon(
            onPressed: controller.trigger,
            icon: const Icon(Icons.play_arrow_rounded),
            label: Text(l10n.trigger),
          ),
          const SizedBox(width: 4),
          TextButton.icon(
            onPressed: controller.status.active ? controller.cancel : null,
            icon: const Icon(Icons.stop_rounded),
            label: Text(l10n.cancel),
          ),
          PopupMenuButton<ClipTypeLanguage>(
            tooltip: l10n.interfaceLanguage,
            icon: const Icon(Icons.language),
            onSelected: widget.onLanguageChanged,
            itemBuilder: (context) => ClipTypeLanguage.values
                .map(
                  (language) => PopupMenuItem(
                    value: language,
                    child: Text(language.label),
                  ),
                )
                .toList(),
          ),
          const SizedBox(width: 12),
        ],
      ),
      body: LayoutBuilder(
        builder: (context, constraints) {
          final extended = constraints.maxWidth >= 960;
          return Row(
            children: [
              NavigationRail(
                extended: extended,
                selectedIndex: _page,
                onDestinationSelected: (value) => setState(() => _page = value),
                labelType: extended ? null : NavigationRailLabelType.all,
                leading: const SizedBox(height: 16),
                destinations: [
                  NavigationRailDestination(
                    icon: Icon(Icons.tune_outlined),
                    selectedIcon: Icon(Icons.tune),
                    label: Text(l10n.general),
                  ),
                  NavigationRailDestination(
                    icon: Icon(Icons.keyboard_alt_outlined),
                    selectedIcon: Icon(Icons.keyboard_alt),
                    label: Text(l10n.shortcuts),
                  ),
                  NavigationRailDestination(
                    icon: Icon(Icons.speed_outlined),
                    selectedIcon: Icon(Icons.speed),
                    label: Text(l10n.typing),
                  ),
                  NavigationRailDestination(
                    icon: Icon(Icons.security_outlined),
                    selectedIcon: Icon(Icons.security),
                    label: Text(l10n.permissions),
                  ),
                  NavigationRailDestination(
                    icon: Icon(Icons.info_outline),
                    selectedIcon: Icon(Icons.info),
                    label: Text(l10n.about),
                  ),
                ],
              ),
              const VerticalDivider(width: 1),
              Expanded(child: _pageBody()),
            ],
          );
        },
      ),
      bottomNavigationBar: feedback == null
          ? null
          : Material(
              color: controller.error == null
                  ? Theme.of(context).colorScheme.secondaryContainer
                  : Theme.of(context).colorScheme.errorContainer,
              child: SafeArea(
                top: false,
                child: Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 22,
                    vertical: 10,
                  ),
                  child: Row(
                    children: [
                      Icon(
                        controller.error == null
                            ? Icons.info_outline
                            : Icons.error_outline,
                        size: 18,
                        color: controller.error == null
                            ? Theme.of(context).colorScheme.onSecondaryContainer
                            : Theme.of(context).colorScheme.onErrorContainer,
                      ),
                      const SizedBox(width: 9),
                      Expanded(
                        child: Text(
                          feedback,
                          style: TextStyle(
                            color: controller.error == null
                                ? Theme.of(context)
                                      .colorScheme
                                      .onSecondaryContainer
                                : Theme.of(context)
                                      .colorScheme
                                      .onErrorContainer,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
    );
  }

  Widget _pageBody() => switch (_page) {
    0 => GeneralPage(
      controller: controller,
      onLanguageChanged: widget.onLanguageChanged,
    ),
    1 => ShortcutsPage(controller: controller),
    2 => TypingPage(controller: controller),
    3 => PermissionsPage(controller: controller),
    _ => const AboutPage(),
  };
}

class _LoadingView extends StatelessWidget {
  const _LoadingView();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.content_paste_go,
              size: 42,
              color: Theme.of(context).colorScheme.primary,
            ),
            const SizedBox(height: 16),
            Text(context.l10n.loading),
            const SizedBox(height: 14),
            const SizedBox(width: 180, child: LinearProgressIndicator()),
          ],
        ),
      ),
    );
  }
}
