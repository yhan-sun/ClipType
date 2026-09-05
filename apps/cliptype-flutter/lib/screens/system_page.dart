import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../model/app_settings.dart';
import '../state/settings_controller.dart';
import '../widgets/setting_card.dart';

class SystemPage extends StatefulWidget {
  const SystemPage({
    required this.controller,
    required this.onLanguageChanged,
    super.key,
  });

  final SettingsController controller;
  final ValueChanged<ClipTypeLanguage> onLanguageChanged;

  @override
  State<SystemPage> createState() => _SystemPageState();
}

class _SystemPageState extends State<SystemPage> {
  late AppSettings _draft;
  bool _hasLocalDraft = false;

  SettingsController get controller => widget.controller;

  @override
  void initState() {
    super.initState();
    _draft = controller.settings;
  }

  @override
  void didUpdateWidget(covariant SystemPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.controller != controller) {
      _draft = controller.settings;
      _hasLocalDraft = false;
    } else if (_hasLocalDraft && _draft == controller.settings) {
      _hasLocalDraft = false;
    } else if (!_hasLocalDraft && _draft != controller.settings) {
      _draft = controller.settings;
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final status = controller.status;
    final granted = controller.permissionGranted;
    return PageContent(
      children: [
        PageHeader(
          title: l10n.text('System', '系统'),
          description: l10n.text(
            'Manage ClipType availability, macOS permission, notifications, startup, and interface language.',
            '管理 ClipType 启用状态、macOS 权限、通知、登录启动和界面语言。',
          ),
        ),
        const SizedBox(height: 24),
        SettingCard(
          title: l10n.text('Application', '应用'),
          child: Column(
            children: [
              SwitchListTile.adaptive(
                contentPadding: EdgeInsets.zero,
                title: Text(l10n.enableClipType),
                subtitle: Text(l10n.enableClipTypeSubtitle),
                value: _draft.enabled,
                onChanged: (value) => _save(_draft.copyWith(enabled: value)),
              ),
              const Divider(height: 12),
              SwitchListTile.adaptive(
                contentPadding: EdgeInsets.zero,
                title: Text(l10n.notifications),
                subtitle: Text(l10n.notificationsSubtitle),
                value: _draft.notifications,
                onChanged: (value) =>
                    _save(_draft.copyWith(notifications: value)),
              ),
              const Divider(height: 12),
              SwitchListTile.adaptive(
                contentPadding: EdgeInsets.zero,
                title: Text(l10n.startAtLogin),
                subtitle: Text(
                  status.startup == 'unsupported'
                      ? l10n.startAtLoginUnsupported
                      : status.startup == 'requires_approval'
                      ? l10n.text(
                          'macOS is waiting for your approval in Login Items.',
                          'macOS 正在等待你在“登录项”中批准。',
                        )
                      : l10n.startAtLoginSubtitle,
                ),
                value: _draft.startAtLogin,
                onChanged: status.startup == 'unsupported'
                    ? null
                    : (value) => _save(_draft.copyWith(startAtLogin: value)),
              ),
              const Divider(height: 12),
              ListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(l10n.interfaceLanguage),
                subtitle: Text(l10n.interfaceLanguageSubtitle),
                trailing: DropdownButton<ClipTypeLanguage>(
                  value: controller.language,
                  onChanged: (value) {
                    if (value != null) widget.onLanguageChanged(value);
                  },
                  items: ClipTypeLanguage.values
                      .map(
                        (language) => DropdownMenuItem(
                          value: language,
                          child: Text(language.label),
                        ),
                      )
                      .toList(),
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        SettingCard(
          title: l10n.accessibility,
          subtitle: l10n.text(
            'Required only for sending synthetic keyboard input across applications.',
            '仅在向其他应用发送模拟键盘输入时需要。',
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Icon(
                    granted ? Icons.verified_outlined : Icons.lock_outline,
                    color: granted
                        ? Theme.of(context).colorScheme.primary
                        : Theme.of(context).colorScheme.error,
                  ),
                  const SizedBox(width: 10),
                  StatusPill(
                    label: l10n.permissionLabel(status.permission),
                    tone: granted ? StatusTone.success : StatusTone.warning,
                  ),
                ],
              ),
              const SizedBox(height: 14),
              Text(switch (status.permission) {
                'granted' => l10n.syntheticInputAvailable,
                'revoked' => l10n.text(
                  'Accessibility was previously available but is now disabled. Re-enable ClipType in System Settings.',
                  '辅助功能权限曾经可用，但现在已被关闭。请在系统设置中重新启用 ClipType。',
                ),
                'not_requested' => l10n.text(
                  'ClipType has not asked for Accessibility yet. Request it only when you are ready to use cross-app input.',
                  'ClipType 尚未请求辅助功能权限。准备使用跨应用输入时再授权即可。',
                ),
                'unknown' => l10n.text(
                  'The permission state could not be confirmed. Open System Settings to review it.',
                  '暂时无法确认权限状态，请打开系统设置检查。',
                ),
                _ => l10n.grantAccess,
              }),
              const SizedBox(height: 16),
              Wrap(
                spacing: 10,
                runSpacing: 10,
                children: [
                  if (!granted)
                    FilledButton.icon(
                      onPressed: controller.requestAccessibility,
                      icon: const Icon(Icons.security_outlined),
                      label: Text(l10n.requestPermission),
                    ),
                  OutlinedButton.icon(
                    onPressed: controller.openAccessibilitySettings,
                    icon: const Icon(Icons.settings_outlined),
                    label: Text(
                      granted
                          ? l10n.text('Manage in System Settings', '在系统设置中管理')
                          : l10n.openSystemSettings,
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        SettingCard(
          title: l10n.safetyBoundary,
          child: Text(
            l10n.text(
              'ClipType needs permission to send keyboard events, but it does not read the destination field, selected text, document contents, or window title.',
              'ClipType 需要权限来发送键盘事件，但不会读取目标输入框、选中文本、文档内容或窗口标题。',
            ),
          ),
        ),
        const SizedBox(height: 16),
        AutoSaveFooter(controller: controller, onReset: _restoreDefaults),
      ],
    );
  }

  void _save(AppSettings value) {
    setState(() {
      _draft = value;
      _hasLocalDraft = true;
    });
    controller.updateSettings(value, debounce: Duration.zero);
  }

  void _restoreDefaults() {
    final defaults = AppSettings.defaults();
    _save(
      _draft.copyWith(
        enabled: defaults.enabled,
        notifications: defaults.notifications,
        startAtLogin: defaults.startAtLogin,
      ),
    );
  }
}
