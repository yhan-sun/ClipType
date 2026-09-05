import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';

import 'l10n/app_localizations.dart';
import 'model/app_status.dart';
import 'screens/about_page.dart';
import 'screens/overview_page.dart';
import 'screens/shortcuts_page.dart';
import 'screens/system_page.dart';
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
          borderRadius: BorderRadius.circular(14),
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
  bool _explainedSettingsTrigger = false;

  SettingsController get controller => widget.controller;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final feedback = controller.error ?? controller.message;
    final compact = MediaQuery.sizeOf(context).width < 860;
    return Scaffold(
      appBar: AppBar(
        titleSpacing: 20,
        title: Row(
          children: [
            Icon(
              Icons.keyboard_command_key_rounded,
              color: Theme.of(context).colorScheme.primary,
            ),
            const SizedBox(width: 10),
            const Text('ClipType'),
            if (!compact) ...[
              const SizedBox(width: 14),
              StatusPill(
                label: l10n.phaseLabel(controller.status.phase),
                tone: _phaseTone(controller.status.phase),
              ),
            ],
          ],
        ),
        actions: [
          if (compact)
            IconButton(
              onPressed: _primaryActionCallback(),
              tooltip: _primaryActionLabel(l10n),
              icon: Icon(_primaryActionIcon()),
            )
          else
            FilledButton.tonalIcon(
              onPressed: _primaryActionCallback(),
              icon: Icon(_primaryActionIcon()),
              label: Text(_primaryActionLabel(l10n)),
            ),
          const SizedBox(width: 8),
        ],
      ),
      body: LayoutBuilder(
        builder: (context, constraints) {
          final extended = constraints.maxWidth >= 980;
          return Row(
            children: [
              NavigationRail(
                extended: extended,
                selectedIndex: _page,
                onDestinationSelected: (value) => setState(() => _page = value),
                labelType: extended ? null : NavigationRailLabelType.all,
                leading: const SizedBox(height: 12),
                destinations: [
                  NavigationRailDestination(
                    icon: const Icon(Icons.dashboard_outlined),
                    selectedIcon: const Icon(Icons.dashboard_rounded),
                    label: Text(l10n.text('Overview', '概览')),
                  ),
                  NavigationRailDestination(
                    icon: const Icon(Icons.tune_outlined),
                    selectedIcon: const Icon(Icons.tune_rounded),
                    label: Text(l10n.text('Input', '输入')),
                  ),
                  NavigationRailDestination(
                    icon: const Icon(Icons.keyboard_alt_outlined),
                    selectedIcon: const Icon(Icons.keyboard_alt),
                    label: Text(l10n.shortcuts),
                  ),
                  NavigationRailDestination(
                    icon: const Icon(Icons.settings_outlined),
                    selectedIcon: const Icon(Icons.settings),
                    label: Text(l10n.text('System', '系统')),
                  ),
                  NavigationRailDestination(
                    icon: const Icon(Icons.info_outline),
                    selectedIcon: const Icon(Icons.info),
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
                  padding: const EdgeInsets.fromLTRB(20, 8, 8, 8),
                  child: Row(
                    children: [
                      Icon(
                        controller.error == null
                            ? Icons.info_outline
                            : Icons.error_outline,
                        size: 18,
                      ),
                      const SizedBox(width: 9),
                      Expanded(child: Text(feedback)),
                      if (controller.readinessReason == 'permission')
                        TextButton(
                          onPressed: () => setState(() => _page = 3),
                          child: Text(l10n.text('Open System', '打开系统页')),
                        ),
                      IconButton(
                        onPressed: controller.clearFeedback,
                        tooltip: l10n.text('Dismiss', '关闭提示'),
                        icon: const Icon(Icons.close, size: 18),
                      ),
                    ],
                  ),
                ),
              ),
            ),
    );
  }

  Widget _pageBody() => switch (_page) {
    0 => OverviewPage(
      controller: controller,
      onOpenInput: () => setState(() => _page = 1),
      onOpenShortcuts: () => setState(() => _page = 2),
      onOpenSystem: () => setState(() => _page = 3),
      onTrigger: _confirmAndTrigger,
    ),
    1 => TypingPage(controller: controller),
    2 => ShortcutsPage(controller: controller),
    3 => SystemPage(
      controller: controller,
      onLanguageChanged: widget.onLanguageChanged,
    ),
    _ => AboutPage(controller: controller),
  };

  VoidCallback? _primaryActionCallback() {
    if (controller.status.active) return controller.cancel;
    return switch (controller.readinessReason) {
      'bridge' => null,
      'disabled' || 'permission' => () => setState(() => _page = 3),
      'shortcuts' => () => setState(() => _page = 2),
      _ => _confirmAndTrigger,
    };
  }

  String _primaryActionLabel(ClipTypeLocalizations l10n) {
    if (controller.status.active) return l10n.text('Stop typing', '停止输入');
    return switch (controller.readinessReason) {
      'bridge' => l10n.text('Runtime unavailable', '运行时不可用'),
      'disabled' => l10n.text('Enable ClipType', '启用 ClipType'),
      'permission' => l10n.text('Grant access', '完成授权'),
      'shortcuts' => l10n.text('Set shortcuts', '设置快捷键'),
      _ => l10n.text('Start typing', '开始输入'),
    };
  }

  IconData _primaryActionIcon() {
    if (controller.status.active) return Icons.stop_rounded;
    return switch (controller.readinessReason) {
      'bridge' => Icons.error_outline,
      'disabled' => Icons.power_settings_new,
      'permission' => Icons.security_outlined,
      'shortcuts' => Icons.keyboard_alt_outlined,
      _ => Icons.play_arrow_rounded,
    };
  }

  Future<void> _confirmAndTrigger() async {
    if (!controller.readyForInput) return;
    if (!_explainedSettingsTrigger) {
      final l10n = context.l10n;
      final proceed = await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          icon: const Icon(Icons.open_in_new_rounded),
          title: Text(
            l10n.text('Type into the previous app?', '要向上一个应用开始输入吗？'),
          ),
          content: Text(
            l10n.text(
              'ClipType will hide this settings window, return focus to the app you were using before it, then begin the bounded input session. The global shortcut does not show this confirmation.',
              'ClipType 会隐藏此设置窗口，把焦点交还给你之前使用的应用，然后开始有界输入会话。使用全局快捷键触发时不会出现此确认。',
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: Text(MaterialLocalizations.of(context).cancelButtonLabel),
            ),
            FilledButton(
              onPressed: () => Navigator.pop(context, true),
              child: Text(l10n.text('Continue', '继续')),
            ),
          ],
        ),
      );
      if (proceed != true || !mounted) return;
      _explainedSettingsTrigger = true;
    }
    await controller.trigger();
  }

  StatusTone _phaseTone(SessionPhase phase) => switch (phase) {
    SessionPhase.idle =>
      controller.readyForInput ? StatusTone.success : StatusTone.neutral,
    SessionPhase.preparing || SessionPhase.injecting => StatusTone.active,
    SessionPhase.cancelling => StatusTone.warning,
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
              Icons.keyboard_command_key_rounded,
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
