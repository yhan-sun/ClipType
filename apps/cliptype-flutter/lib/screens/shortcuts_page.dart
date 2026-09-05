import 'dart:async';

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
  int _applyGeneration = 0;

  SettingsController get controller => widget.controller;

  @override
  void initState() {
    super.initState();
    _syncFromSettings();
  }

  @override
  void didUpdateWidget(covariant ShortcutsPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.controller != controller) _syncFromSettings();
  }

  void _syncFromSettings() {
    _trigger = controller.settings.triggerHotkey;
    _cancel = controller.settings.cancelHotkey;
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final validation = _staticValidation();
    final checking = controller.overallAvailability == 'checking';
    return PageContent(
      children: [
        PageHeader(
          title: l10n.shortcuts,
          description: l10n.text(
            'Record Trigger and Stop shortcuts. A complete candidate is checked and applied transactionally; a conflict never replaces the previous working pair.',
            '录制“开始输入”和“停止输入”快捷键。完整候选组合会自动检查并以事务方式应用；发生冲突时不会覆盖上一组仍可用的快捷键。',
          ),
        ),
        const SizedBox(height: 24),
        SettingCard(
          title: l10n.text('Global shortcuts', '全局快捷键'),
          subtitle: l10n.text(
            'Press a complete modifier + key combination. Esc cancels recording; Delete or Backspace clears only the candidate shown here.',
            '请输入完整的“修饰键 + 按键”组合。Esc 取消录制；Delete 或 Backspace 只清空当前页面中的候选组合。',
          ),
          child: Column(
            children: [
              HotkeyRecorder(
                title: l10n.text('Start typing', '开始输入'),
                value: _trigger,
                onChanged: (value) => _setCandidate(trigger: value),
                onCleared: () => _setCandidate(trigger: ''),
                enabled: !checking,
              ),
              const SizedBox(height: 10),
              HotkeyRecorder(
                title: l10n.text('Stop typing', '停止输入'),
                value: _cancel,
                onChanged: (value) => _setCandidate(cancel: value),
                onCleared: () => _setCandidate(cancel: ''),
                enabled: !checking,
              ),
              if (validation != null) ...[
                const SizedBox(height: 10),
                Align(
                  alignment: Alignment.centerLeft,
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Icon(
                        Icons.warning_amber_rounded,
                        size: 18,
                        color: Theme.of(context).colorScheme.error,
                      ),
                      const SizedBox(width: 8),
                      Expanded(
                        child: Text(
                          l10n.validationMessage(validation),
                          style: TextStyle(
                            color: Theme.of(context).colorScheme.error,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
              if (checking) ...[
                const SizedBox(height: 12),
                const LinearProgressIndicator(),
              ],
            ],
          ),
        ),
        const SizedBox(height: 16),
        SettingCard(
          title: l10n.text('Registration status', '注册状态'),
          subtitle: l10n.text(
            'Available means macOS accepted the global pair. App-local shortcuts and hook-based tools still cannot be fully verified.',
            '“可用”表示 macOS 已接受这组全局快捷键；应用内部快捷键或基于 Hook 的工具仍无法被完全验证。',
          ),
          child: Column(
            children: [
              _AvailabilityRow(
                label: l10n.text('Start typing', '开始输入'),
                value: controller.triggerAvailability,
              ),
              const SizedBox(height: 12),
              _AvailabilityRow(
                label: l10n.text('Stop typing', '停止输入'),
                value: controller.cancelAvailability,
              ),
              const SizedBox(height: 14),
              Align(
                alignment: Alignment.centerLeft,
                child: OutlinedButton.icon(
                  onPressed: validation == null && !checking
                      ? () => controller.probeHotkeys(_trigger, _cancel)
                      : null,
                  icon: const Icon(Icons.refresh_rounded),
                  label: Text(l10n.text('Recheck candidate', '重新检查候选组合')),
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        const _ShortcutBoundaryNote(),
        const SizedBox(height: 16),
        AutoSaveFooter(controller: controller, onReset: _restoreDefaults),
      ],
    );
  }

  void _setCandidate({String? trigger, String? cancel}) {
    setState(() {
      if (trigger != null) _trigger = trigger;
      if (cancel != null) _cancel = cancel;
    });
    controller.resetHotkeyAvailability();
    final validation = _staticValidation();
    if (validation == null) {
      final generation = ++_applyGeneration;
      unawaited(_applyCandidate(generation));
    } else {
      _applyGeneration += 1;
    }
  }

  Future<void> _applyCandidate(int generation) async {
    final trigger = _trigger;
    final cancel = _cancel;
    final applied = await controller.applyHotkeys(trigger, cancel);
    if (!mounted || generation != _applyGeneration) return;
    if (applied) {
      setState(_syncFromSettings);
    }
  }

  void _restoreDefaults() {
    final defaults = AppSettings.defaults();
    setState(() {
      _trigger = defaults.triggerHotkey;
      _cancel = defaults.cancelHotkey;
    });
    controller.resetHotkeyAvailability();
    final generation = ++_applyGeneration;
    unawaited(_applyCandidate(generation));
  }

  String? _staticValidation() {
    if (_trigger.isEmpty || _cancel.isEmpty) return 'shortcuts_required';
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
    final l10n = context.l10n;
    final tone = switch (value) {
      'available' => StatusTone.success,
      'checking' => StatusTone.active,
      'conflict' || 'reserved' || 'unsupported' => StatusTone.error,
      _ => StatusTone.neutral,
    };
    return Row(
      children: [
        Expanded(child: Text(label)),
        if (value == 'checking') ...[
          const SizedBox(
            width: 16,
            height: 16,
            child: CircularProgressIndicator(strokeWidth: 2),
          ),
          const SizedBox(width: 8),
        ],
        StatusPill(
          label: value == 'checking'
              ? l10n.text('Checking…', '检查中…')
              : l10n.availabilityLabel(value),
          tone: tone,
        ),
      ],
    );
  }
}

class _ShortcutBoundaryNote extends StatelessWidget {
  const _ShortcutBoundaryNote();

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            Icons.shield_outlined,
            size: 20,
            color: Theme.of(context).colorScheme.primary,
          ),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              l10n.text(
                'The currently registered pair stays active until a replacement succeeds. Clearing or editing a candidate does not silently unregister the last working shortcuts.',
                '上一组已注册快捷键会一直保持有效，直到新的组合成功替换。清空或编辑候选组合不会悄悄注销最后一组可用快捷键。',
              ),
            ),
          ),
        ],
      ),
    );
  }
}
