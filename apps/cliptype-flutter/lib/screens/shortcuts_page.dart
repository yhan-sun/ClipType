import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../model/app_settings.dart';
import '../state/settings_controller.dart';
import '../widgets/hotkey_recorder.dart';
import '../widgets/setting_card.dart';

class ShortcutsPage extends StatefulWidget {
  const ShortcutsPage({required this.controller, super.key});

  final SettingsController controller;

  @override
  State<ShortcutsPage> createState() => _ShortcutsPageState();
}

class _ShortcutsPageState extends State<ShortcutsPage> {
  late String _trigger;
  late String _cancel;
  bool _hasLocalDraft = false;

  SettingsController get controller => widget.controller;

  @override
  void initState() {
    super.initState();
    _trigger = controller.settings.triggerHotkey;
    _cancel = controller.settings.cancelHotkey;
  }

  @override
  void didUpdateWidget(covariant ShortcutsPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.controller != controller) {
      _trigger = controller.settings.triggerHotkey;
      _cancel = controller.settings.cancelHotkey;
      _hasLocalDraft = false;
      return;
    }
    if (_hasLocalDraft &&
        _trigger == controller.settings.triggerHotkey &&
        _cancel == controller.settings.cancelHotkey) {
      _hasLocalDraft = false;
    }
    if (!_hasLocalDraft &&
        (_trigger != controller.settings.triggerHotkey ||
            _cancel != controller.settings.cancelHotkey)) {
      _trigger = controller.settings.triggerHotkey;
      _cancel = controller.settings.cancelHotkey;
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final validation = _staticValidation();
    return PageContent(
      children: [
        PageHeader(
          title: l10n.shortcuts,
          description: l10n.shortcutsDescription,
          trailing: AutoSaveIndicator(controller: controller),
        ),
        const SizedBox(height: 26),
        SettingCard(
          title: l10n.globalCommands,
          subtitle: l10n.globalCommandsSubtitle,
          child: Column(
            children: [
              HotkeyRecorder(
                title: l10n.triggerShortcut,
                value: _trigger,
                onChanged: (value) => _setShortcut(trigger: value),
                onCleared: () => _setShortcut(trigger: ''),
              ),
              const SizedBox(height: 12),
              HotkeyRecorder(
                title: l10n.cancelShortcut,
                value: _cancel,
                onChanged: (value) => _setShortcut(cancel: value),
                onCleared: () => _setShortcut(cancel: ''),
              ),
              if (validation != null) ...[
                const SizedBox(height: 14),
                Align(
                  alignment: Alignment.centerLeft,
                  child: Text(
                    l10n.validationMessage(validation),
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.error,
                    ),
                  ),
                ),
              ],
            ],
          ),
        ),
        const SizedBox(height: 16),
        SettingCard(
          title: l10n.osAvailability,
          subtitle: l10n.osAvailabilitySubtitle,
          child: Column(
            children: [
              Row(
                children: [
                  Expanded(
                    child: _AvailabilityRow(
                      label: l10n.trigger,
                      value: controller.triggerAvailability,
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: _AvailabilityRow(
                      label: l10n.cancel,
                      value: controller.cancelAvailability,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 16),
              Align(
                alignment: Alignment.centerLeft,
                child: OutlinedButton.icon(
                  onPressed: validation == null
                      ? () => controller.probeHotkeys(_trigger, _cancel)
                      : null,
                  icon: const Icon(Icons.search),
                  label: Text(l10n.checkAvailability),
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        AutoSaveFooter(controller: controller, onReset: _restoreDefaults),
      ],
    );
  }

  void _setShortcut({String? trigger, String? cancel}) {
    setState(() {
      if (trigger != null) _trigger = trigger;
      if (cancel != null) _cancel = cancel;
      _hasLocalDraft = true;
    });
    controller.updateSettings(_draft(), debounce: Duration.zero);
  }

  void _restoreDefaults() {
    final defaults = AppSettings.defaults();
    setState(() {
      _trigger = defaults.triggerHotkey;
      _cancel = defaults.cancelHotkey;
      _hasLocalDraft = true;
    });
    controller.updateSettings(_draft(), debounce: Duration.zero);
  }

  AppSettings _draft() {
    return controller.settings.copyWith(
      triggerHotkey: _trigger,
      cancelHotkey: _cancel,
    );
  }

  String? _staticValidation() {
    if (_trigger.isEmpty || _cancel.isEmpty) {
      return 'shortcuts_required';
    }
    if (_trigger == _cancel) return 'different_hotkeys';
    return null;
  }
}

class _AvailabilityRow extends StatelessWidget {
  const _AvailabilityRow({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    final good = value == 'available';
    return Row(
      children: [
        Expanded(child: Text(label)),
        StatusPill(label: context.l10n.availabilityLabel(value), good: good),
      ],
    );
  }
}
