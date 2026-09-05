import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../model/app_status.dart';
import '../state/settings_controller.dart';
import '../widgets/setting_card.dart';

class OverviewPage extends StatelessWidget {
  const OverviewPage({
    required this.controller,
    required this.onOpenInput,
    required this.onOpenShortcuts,
    required this.onOpenSystem,
    required this.onTrigger,
    super.key,
  });

  final SettingsController controller;
  final VoidCallback onOpenInput;
  final VoidCallback onOpenShortcuts;
  final VoidCallback onOpenSystem;
  final VoidCallback onTrigger;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final status = controller.status;
    final ready = controller.readyForInput;
    return PageContent(
      children: [
        PageHeader(
          title: l10n.text('Overview', '概览'),
          description: l10n.text(
            'See whether ClipType is ready, what it will do, and the one next action that matters.',
            '集中查看 ClipType 是否已经可用、当前会怎样输入，以及现在最需要完成的下一步。',
          ),
        ),
        const SizedBox(height: 24),
        Container(
          padding: const EdgeInsets.all(22),
          decoration: BoxDecoration(
            color: ready
                ? Theme.of(context).colorScheme.primaryContainer
                : Theme.of(context).colorScheme.surfaceContainerLow,
            borderRadius: BorderRadius.circular(18),
            border: Border.all(
              color: Theme.of(context).colorScheme.outlineVariant,
            ),
          ),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(
                ready ? Icons.check_circle_rounded : Icons.info_outline_rounded,
                size: 32,
                color: ready
                    ? Theme.of(context).colorScheme.primary
                    : Theme.of(context).colorScheme.onSurfaceVariant,
              ),
              const SizedBox(width: 16),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      ready
                          ? l10n.text('ClipType is ready', 'ClipType 已准备就绪')
                          : _notReadyTitle(l10n),
                      style: Theme.of(context).textTheme.titleLarge
                          ?.copyWith(fontWeight: FontWeight.w700),
                    ),
                    const SizedBox(height: 7),
                    Text(
                      ready
                          ? l10n.text(
                              'Copy text, focus the destination app, then press ${_hotkeyLabel(controller.settings.triggerHotkey)}.',
                              '复制文本，聚焦目标应用，然后按 ${_hotkeyLabel(controller.settings.triggerHotkey)}。',
                            )
                          : _notReadyDescription(l10n),
                      style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                        color: Theme.of(context).colorScheme.onSurfaceVariant,
                        height: 1.45,
                      ),
                    ),
                    const SizedBox(height: 16),
                    _PrimaryNextAction(
                      controller: controller,
                      onOpenShortcuts: onOpenShortcuts,
                      onOpenSystem: onOpenSystem,
                      onTrigger: onTrigger,
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        SettingCard(
          title: l10n.text('Readiness checklist', '准备状态'),
          child: Column(
            children: [
              _ReadinessRow(
                icon: Icons.power_settings_new,
                title: l10n.text('ClipType enabled', 'ClipType 已启用'),
                detail: controller.settings.enabled
                    ? l10n.text('Global trigger is accepted.', '可以响应全局触发快捷键。')
                    : l10n.text(
                        'Turn ClipType on before starting.',
                        '开始输入前需要先启用 ClipType。',
                      ),
                good: controller.settings.enabled,
                onTap: onOpenSystem,
              ),
              const Divider(height: 24),
              _ReadinessRow(
                icon: Icons.security_outlined,
                title: l10n.text('Accessibility', '辅助功能权限'),
                detail: controller.permissionGranted
                    ? l10n.text('Permission granted.', '已授权。')
                    : l10n.text(
                        'Required for cross-application keyboard input.',
                        '跨应用键盘输入需要此权限。',
                      ),
                good: controller.permissionGranted,
                onTap: onOpenSystem,
              ),
              const Divider(height: 24),
              _ReadinessRow(
                icon: Icons.keyboard_alt_outlined,
                title: l10n.text('Global shortcuts', '全局快捷键'),
                detail: status.hotkeysRegistered
                    ? '${_hotkeyLabel(controller.settings.triggerHotkey)}  ·  ${_hotkeyLabel(controller.settings.cancelHotkey)}'
                    : l10n.text(
                        'No working shortcut pair is registered.',
                        '当前没有已成功注册的快捷键组合。',
                      ),
                good: status.hotkeysRegistered,
                onTap: onOpenShortcuts,
              ),
              const Divider(height: 24),
              _ReadinessRow(
                icon: Icons.tune_rounded,
                title: l10n.text('Input mode', '输入模式'),
                detail: l10n.modeLabel(controller.settings.mode),
                good: true,
                onTap: onOpenInput,
              ),
              if (!status.bridgeAvailable) ...[
                const Divider(height: 24),
                _ReadinessRow(
                  icon: Icons.error_outline,
                  title: l10n.text('Native runtime', '原生运行时'),
                  detail: l10n.text(
                    'Unavailable. Restart ClipType or reinstall the current build.',
                    '不可用。请重启 ClipType 或重新安装当前版本。',
                  ),
                  good: false,
                  onTap: null,
                ),
              ],
            ],
          ),
        ),
        const SizedBox(height: 16),
        SettingCard(
          title: l10n.text('Last session', '最近一次输入'),
          child: Row(
            children: [
              StatusPill(
                label: l10n.phaseLabel(status.phase),
                tone: switch (status.phase) {
                  SessionPhase.idle => StatusTone.success,
                  SessionPhase.preparing ||
                  SessionPhase.injecting => StatusTone.active,
                  SessionPhase.cancelling => StatusTone.warning,
                },
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Text(
                  status.active
                      ? l10n.activeSessionStatus
                      : status.completion == null
                      ? l10n.text(
                          'No completed session in this app run yet.',
                          '本次运行中还没有已完成的输入会话。',
                        )
                      : l10n.completionLabel(status.completion!),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  String _notReadyTitle(ClipTypeLocalizations l10n) =>
      switch (controller.readinessReason) {
        'bridge' => l10n.text('Native runtime unavailable', '原生运行时不可用'),
        'disabled' => l10n.text('ClipType is paused', 'ClipType 已暂停'),
        'permission' => l10n.text(
          'Accessibility permission required',
          '需要辅助功能权限',
        ),
        'shortcuts' => l10n.text('Set a working shortcut pair', '需要设置可用的快捷键'),
        'active' => l10n.text('Typing is in progress', '正在输入'),
        _ => l10n.text('One setup step remains', '还需要完成一项设置'),
      };

  String _notReadyDescription(ClipTypeLocalizations l10n) =>
      switch (controller.readinessReason) {
        'bridge' => l10n.text(
          'ClipType cannot reach the native input runtime.',
          'ClipType 当前无法连接原生输入运行时。',
        ),
        'disabled' => l10n.text(
          'Enable ClipType in System settings.',
          '请在“系统”页面启用 ClipType。',
        ),
        'permission' => l10n.text(
          'Grant Accessibility permission in macOS System Settings.',
          '请在 macOS 系统设置中授予辅助功能权限。',
        ),
        'shortcuts' => l10n.text(
          'Record Trigger and Stop shortcuts that macOS accepts.',
          '请录制一组 macOS 可接受的“开始”和“停止”快捷键。',
        ),
        'active' => l10n.text(
          'Use Stop to end the current bounded session.',
          '可以使用“停止输入”结束当前有界会话。',
        ),
        _ => l10n.text('Review the checklist below.', '请检查下面的准备状态。'),
      };
}

class _PrimaryNextAction extends StatelessWidget {
  const _PrimaryNextAction({
    required this.controller,
    required this.onOpenShortcuts,
    required this.onOpenSystem,
    required this.onTrigger,
  });

  final SettingsController controller;
  final VoidCallback onOpenShortcuts;
  final VoidCallback onOpenSystem;
  final VoidCallback onTrigger;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    if (controller.status.active) {
      return FilledButton.icon(
        onPressed: controller.cancel,
        icon: const Icon(Icons.stop_rounded),
        label: Text(l10n.text('Stop typing', '停止输入')),
      );
    }
    final (label, icon, action) = switch (controller.readinessReason) {
      'disabled' => (
        l10n.text('Open System settings', '打开系统设置'),
        Icons.settings_outlined,
        onOpenSystem,
      ),
      'permission' => (
        l10n.text('Grant Accessibility', '完成辅助功能授权'),
        Icons.security_outlined,
        onOpenSystem,
      ),
      'shortcuts' => (
        l10n.text('Set shortcuts', '设置快捷键'),
        Icons.keyboard_alt_outlined,
        onOpenShortcuts,
      ),
      'bridge' => (
        l10n.text('Runtime unavailable', '运行时不可用'),
        Icons.error_outline,
        null,
      ),
      _ => (
        l10n.text('Start typing', '开始输入'),
        Icons.play_arrow_rounded,
        onTrigger,
      ),
    };
    return FilledButton.icon(
      onPressed: action,
      icon: Icon(icon),
      label: Text(label),
    );
  }
}

class _ReadinessRow extends StatelessWidget {
  const _ReadinessRow({
    required this.icon,
    required this.title,
    required this.detail,
    required this.good,
    required this.onTap,
  });

  final IconData icon;
  final String title;
  final String detail;
  final bool good;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final color = good
        ? Theme.of(context).colorScheme.primary
        : Theme.of(context).colorScheme.error;
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(10),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: Row(
          children: [
            Icon(icon, color: color),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(title, style: Theme.of(context).textTheme.titleSmall),
                  const SizedBox(height: 3),
                  Text(
                    detail,
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: Theme.of(context).colorScheme.onSurfaceVariant,
                    ),
                  ),
                ],
              ),
            ),
            Icon(
              good ? Icons.check_circle_outline : Icons.chevron_right,
              color: good
                  ? color
                  : Theme.of(context).colorScheme.onSurfaceVariant,
              size: 20,
            ),
          ],
        ),
      ),
    );
  }
}

String _hotkeyLabel(String value) {
  if (value.trim().isEmpty) return '—';
  final map = <String, String>{
    'ctrl': '⌃',
    'alt': '⌥',
    'shift': '⇧',
    'meta': '⌘',
  };
  final parts = value.split('+');
  return parts.map((part) => map[part] ?? part.toUpperCase()).join();
}
