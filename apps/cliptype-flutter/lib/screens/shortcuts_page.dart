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
  bool _dirty = false;

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
    if (!_dirty && oldWidget.controller.settings != controller.settings) {
      _trigger = controller.settings.triggerHotkey;
      _cancel = controller.settings.cancelHotkey;
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final draft = controller.settings.copyWith(
      triggerHotkey: _trigger,
      cancelHotkey: _cancel,
    );
    final validation = _staticValidation();
    return ListView(
      padding: const EdgeInsets.fromLTRB(32, 30, 32, 32),
      children: [
        PageHeader(
          title: l10n.shortcuts,
          description: l10n.shortcutsDescription,
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
                onChanged: (value) => setState(() {
                  _trigger = value;
                  _dirty = true;
                }),
                onCleared: () => setState(() {
                  _trigger = '';
                  _dirty = true;
                }),
              ),
              const SizedBox(height: 12),
              HotkeyRecorder(
                title: l10n.cancelShortcut,
                value: _cancel,
                onChanged: (value) => setState(() {
                  _cancel = value;
                  _dirty = true;
                }),
                onCleared: () => setState(() {
                  _cancel = '';
                  _dirty = true;
                }),
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
        SettingCard(
          child: SaveBar(
            saving: controller.saving,
            error: controller.error,
            onReset: () => setState(() {
              final defaults = AppSettings.defaults();
              _trigger = defaults.triggerHotkey;
              _cancel = defaults.cancelHotkey;
              _dirty = true;
            }),
            onApply: () async {
              final applied = await controller.save(draft);
              if (applied && mounted) setState(() => _dirty = false);
            },
          ),
        ),
      ],
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
