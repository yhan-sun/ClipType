import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../state/settings_controller.dart';
import '../widgets/setting_card.dart';

class PermissionsPage extends StatelessWidget {
  const PermissionsPage({required this.controller, super.key});

  final SettingsController controller;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final status = controller.status;
    final granted = status.permission == 'granted';
    return PageContent(
      children: [
        PageHeader(
          title: l10n.permissions,
          description: l10n.permissionsDescription,
        ),
        const SizedBox(height: 26),
        SettingCard(
          title: l10n.accessibility,
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
                    good: granted,
                  ),
                ],
              ),
              const SizedBox(height: 14),
              Text(granted ? l10n.syntheticInputAvailable : l10n.grantAccess),
              const SizedBox(height: 18),
              Wrap(
                spacing: 10,
                runSpacing: 10,
                children: [
                  FilledButton.icon(
                    onPressed: granted ? null : controller.requestAccessibility,
                    icon: const Icon(Icons.open_in_new),
                    label: Text(l10n.requestPermission),
                  ),
                  OutlinedButton.icon(
                    onPressed: controller.openAccessibilitySettings,
                    icon: const Icon(Icons.settings_outlined),
                    label: Text(l10n.openSystemSettings),
                  ),
                ],
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        SettingCard(
          title: l10n.safetyBoundary,
          child: Text(l10n.safetyBoundaryText),
        ),
      ],
    );
  }
}
