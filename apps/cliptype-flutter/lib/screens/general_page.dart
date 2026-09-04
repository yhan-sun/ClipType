import 'dart:async';

import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../model/app_settings.dart';
import '../state/settings_controller.dart';
import '../widgets/setting_card.dart';

class GeneralPage extends StatelessWidget {
  const GeneralPage({
    required this.controller,
    required this.onLanguageChanged,
    super.key,
  });

  final SettingsController controller;
  final ValueChanged<ClipTypeLanguage> onLanguageChanged;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final settings = controller.settings;
    final status = controller.status;
    return ListView(
      padding: const EdgeInsets.fromLTRB(32, 30, 32, 32),
      children: [
        PageHeader(title: l10n.general, description: l10n.generalDescription),
        const SizedBox(height: 26),
        SettingCard(
          title: l10n.application,
          subtitle: l10n.futureSessions,
          child: Column(
            children: [
              ListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(l10n.interfaceLanguage),
                subtitle: Text(l10n.interfaceLanguageSubtitle),
                trailing: DropdownButton<ClipTypeLanguage>(
                  value: controller.language,
                  onChanged: (value) {
                    if (value != null) onLanguageChanged(value);
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
              const Divider(height: 12),
              SwitchListTile.adaptive(
                contentPadding: EdgeInsets.zero,
                title: Text(l10n.enableClipType),
                subtitle: Text(l10n.enableClipTypeSubtitle),
                value: settings.enabled,
                onChanged: (value) => _save(settings.copyWith(enabled: value)),
              ),
              SwitchListTile.adaptive(
                contentPadding: EdgeInsets.zero,
                title: Text(l10n.notifications),
                subtitle: Text(l10n.notificationsSubtitle),
                value: settings.notifications,
                onChanged: (value) =>
                    _save(settings.copyWith(notifications: value)),
              ),
              SwitchListTile.adaptive(
                contentPadding: EdgeInsets.zero,
                title: Text(l10n.startAtLogin),
                subtitle: Text(
                  status.startup == 'unsupported'
                      ? l10n.startAtLoginUnsupported
                      : l10n.startAtLoginSubtitle,
                ),
                value: settings.startAtLogin,
                onChanged: status.startup == 'unsupported'
                    ? null
                    : (value) => _save(settings.copyWith(startAtLogin: value)),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        SettingCard(
          title: l10n.currentRuntimeStatus,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  StatusPill(
                    label: l10n.phaseLabel(status.phase),
                    good: !status.active,
                  ),
                  const SizedBox(width: 10),
                  if (status.backend != null)
                    Text(
                      '${l10n.backendPrefix}: ${l10n.backendLabel(status.backend)}',
                    ),
                ],
              ),
              const SizedBox(height: 14),
              Text(
                status.active
                    ? l10n.activeSessionStatus
                    : status.completion == null
                    ? l10n.readyForTrigger
                    : l10n.completionLabel(status.completion!),
              ),
              if (status.active) ...[
                const SizedBox(height: 12),
                OutlinedButton.icon(
                  onPressed: controller.cancel,
                  icon: const Icon(Icons.stop_circle_outlined),
                  label: Text(l10n.cancelActiveSession),
                ),
              ],
            ],
          ),
        ),
      ],
    );
  }

  void _save(AppSettings value) {
    unawaited(controller.save(value));
  }
}
